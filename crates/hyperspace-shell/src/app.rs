//! Main application state and egui UI wiring for the Hyperspace shell.
//!
//! HyperspaceApp owns:
//! - HyperspaceState (dimensions + active + objects) + persistence (JsonWorkspaceStore).
//! - In-memory ObjectStore sync.
//! - Agent runtime (stub today).
//! - CanvasInteraction (input + events for pan/zoom/move/resize/Link).
//! - UI panels: top bar (dims + save), left HUD (controls + spawn + AI + FS), right inspector, central canvas.
//!
//! Recent 2026-06-03 changes documented here and in canvas.rs + DEVELOPMENT-LOG.md:
//! - Size editing + resize event handling (with GRID_SNAP).
//! - Link target setting UI + deferred navigation (pending_link_nav to dodge active_dimension_mut borrow).
//! - Spawn now includes Link; Link click handling.
//! - Pre-collection of other_dims before mutable borrows in inspector.
//!
//! See docs/ for full feature docs. Run with `cargo run -p hyperspace-shell`.

use eframe::egui;
use hyperspace_core::{
    DimensionId, HyperspaceState, ObjectKind, SmartObject, SmartObjectId, Viewport, WorldPoint,
    WorldSize,
};
use hyperspace_ai::{AgentMessage, AgentRuntime, LocalAgentRuntime};

use hyperspace_fs::{InMemoryObjectStore, JsonWorkspaceStore, ObjectStore};

use crate::canvas::{self, CanvasEvent, CanvasInteraction};
use crate::theme;

const GRID_SNAP: f32 = 20.0;

pub struct HyperspaceApp {
    state: HyperspaceState,
    store: InMemoryObjectStore,
    workspace_store: JsonWorkspaceStore,
    agent_runtime: LocalAgentRuntime,
    canvas: CanvasInteraction,
    selected_object: Option<SmartObjectId>,
    new_dimension_name: String,
    status: String,
    show_hud: bool,
    dirty: bool,
    /// Deferred "fit canvas to content" request — applied in the CentralPanel where the
    /// live canvas size is known (set from the dock button or the `F` shortcut).
    pending_fit: bool,
}

impl HyperspaceApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        let workspace_store = JsonWorkspaceStore::new(JsonWorkspaceStore::default_path());
        let state = workspace_store.load_or_default();
        let store = InMemoryObjectStore::from_state(&state);
        let status = format!(
            "Loaded workspace · {}",
            workspace_store.path().display()
        );

        Self {
            state,
            store,
            workspace_store,
            agent_runtime: LocalAgentRuntime::new(),
            canvas: CanvasInteraction::default(),
            selected_object: None,
            new_dimension_name: String::new(),
            status,
            show_hud: true,
            dirty: false,
            pending_fit: false,
        }
    }

    /// Create a new Dimension (workspace) from `new_dimension_name`, falling back to an
    /// auto-generated name. `add_dimension` makes it the active dimension.
    fn create_dimension(&mut self) {
        let name = {
            let trimmed = self.new_dimension_name.trim();
            if trimmed.is_empty() {
                format!("Dimension {}", self.state.dimensions.len() + 1)
            } else {
                trimmed.to_string()
            }
        };
        self.state.add_dimension(name.clone());
        self.new_dimension_name.clear();
        self.selected_object = None;
        self.sync_active_dimension();
        self.mark_dirty();
        self.status = format!("Created workspace '{name}'");
    }

    fn active_dimension_id(&self) -> DimensionId {
        self.state.active_dimension
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn save_workspace(&mut self) {
        match self.workspace_store.save(&self.state) {
            Ok(()) => {
                self.dirty = false;
                self.status = format!("Saved · {}", self.workspace_store.path().display());
            }
            Err(err) => self.status = format!("Save failed: {err}"),
        }
    }

    fn sync_active_dimension(&mut self) {
        if let Some(dimension) = self.state.active_dimension() {
            let _ = self.store.sync_dimension(dimension);
        }
    }

    fn switch_dimension(&mut self, id: DimensionId) {
        if self.state.dimension_by_id(id).is_some() {
            self.state.active_dimension = id;
            self.selected_object = None;
            self.status = format!(
                "Switched to {}",
                self.state
                    .active_dimension()
                    .map(|d| d.name.as_str())
                    .unwrap_or("unknown")
            );
        }
    }

    fn spawn_object(&mut self, kind: ObjectKind, world: WorldPoint) {
        let title = match kind {
            ObjectKind::Note => "New Note",
            ObjectKind::App => "New App",
            ObjectKind::Folder => "New Folder",
            ObjectKind::Agent => "New Agent",
            ObjectKind::Link => "New Link",
        };

        let object = SmartObject::new(kind, title, snap_point(world))
            .with_body("Smart Object placeholder");

        if let Some(dimension) = self.state.active_dimension_mut() {
            self.selected_object = Some(object.id);
            dimension.objects.push(object);
            self.status = format!("Created {title}");
            self.mark_dirty();
            self.sync_active_dimension();
        }
    }

    fn delete_selected(&mut self) {
        let Some(id) = self.selected_object.take() else {
            return;
        };
        let dimension_id = self.active_dimension_id();
        if self.state.remove_object(dimension_id, id) {
            self.status = "Deleted object".into();
            self.mark_dirty();
            self.sync_active_dimension();
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let typing = ctx.wants_keyboard_input();

        ctx.input(|input| {
            if input.key_pressed(egui::Key::Escape) {
                self.selected_object = None;
                self.status = "Selection cleared".into();
            }

            if !typing && input.key_pressed(egui::Key::Delete) {
                self.delete_selected();
            }

            // F — fit the canvas to the dimension's content (applied next frame in CentralPanel).
            if !typing && input.key_pressed(egui::Key::F) {
                self.pending_fit = true;
            }

            if input.modifiers.command && input.key_pressed(egui::Key::S) {
                self.save_workspace();
            }
        });
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(6, 8, 16))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 90)))
        ).show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Premium top menubar inspired by the reference mockup (Workspaces, Apps/Objects, System, AI)
                ui.label(egui::RichText::new("OS/3 HYPERSPACE").strong().color(egui::Color32::from_rgb(200, 160, 255)));
                ui.add_space(12.0);

                // "Workspaces" section (our Dimensions)
                ui.label(egui::RichText::new("Workspaces").small().color(egui::Color32::from_rgb(170, 180, 210)));
                let tabs: Vec<(DimensionId, String)> = self
                    .state
                    .dimensions
                    .iter()
                    .map(|dimension| (dimension.id, dimension.name.clone()))
                    .collect();
                let active_id = self.active_dimension_id();
                for (id, name) in tabs {
                    let selected = id == active_id;
                    let text = egui::RichText::new(name).color(if selected { egui::Color32::from_rgb(220, 200, 255) } else { egui::Color32::from_rgb(180, 190, 220) });
                    if ui.selectable_label(selected, text).clicked() {
                        self.switch_dimension(id);
                    }
                }

                // Create a new workspace (Dimension). Uses `new_dimension_name`; Enter or ＋ submits.
                let name_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.new_dimension_name)
                        .hint_text("New workspace…")
                        .desired_width(110.0),
                );
                let submit_via_enter =
                    name_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui
                    .button("＋")
                    .on_hover_text("Create workspace (Dimension)")
                    .clicked()
                    || submit_via_enter
                {
                    self.create_dimension();
                }

                ui.separator();

                // "Objects" / spawn quick access (like Apps in the mockup)
                ui.label(egui::RichText::new("Objects").small().color(egui::Color32::from_rgb(170, 180, 210)));
                if ui.button("Spawn").clicked() {
                    // Could open a popup in future; for now just hint
                    self.status = "Use left HUD Spawn buttons (or double-click canvas for Note)".into();
                }

                ui.separator();

                ui.label(egui::RichText::new("System").small().color(egui::Color32::from_rgb(170, 180, 210)));
                if ui.button("About").clicked() {
                    self.status = "OS/3 Hyperspace — AI-native multidimensional prototype (egui desktop)".into();
                }

                ui.separator();

                ui.label(egui::RichText::new("AI").small().color(egui::Color32::from_rgb(170, 180, 210)));
                if ui.button("Copilot").clicked() {
                    self.status = "AI Runtime available in left HUD. Local-first, always yours.".into();
                }

                // Dirty indicator + right side (status like the mockup's top-right icons/time)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.dirty {
                        ui.label(egui::RichText::new("● UNSAVED").color(egui::Color32::from_rgb(255, 196, 86)).small());
                    }
                    ui.label(egui::RichText::new(&self.status).small().color(egui::Color32::from_rgb(140, 150, 180)));
                    ui.add_space(8.0);
                    if ui.button("Save").clicked() {
                        self.save_workspace();
                    }
                    if ui.button("⛶").clicked() { // toggle HUD like "full" button
                        self.show_hud = !self.show_hud;
                    }
                });
            });
        });
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        if !self.show_hud {
            return;
        }

        egui::SidePanel::left("hud")
            .resizable(true)
            .default_width(260.0)
            .frame(egui::Frame::default()
                .fill(egui::Color32::from_rgb(8, 10, 18))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 45, 75))))
            .show(ctx, |ui| {
                // Modern section header like the left sidebar in the reference
                ui.add_space(4.0);
                ui.label(egui::RichText::new("HYPERDRIVE").strong().color(egui::Color32::from_rgb(180, 150, 255)).small());
                ui.add_space(2.0);

                ui.label(egui::RichText::new("Controls").color(egui::Color32::from_rgb(140, 150, 190)).small());
                ui.label(egui::RichText::new("Scroll — zoom • Middle/Space-drag — pan • Double-click — note").small().color(egui::Color32::from_rgb(160, 165, 190)));
                ui.label(egui::RichText::new("Drag corners — resize • Delete — remove • Ctrl+S — save").small().color(egui::Color32::from_rgb(160, 165, 190)));
                ui.add_space(6.0);

                // Spawn section styled like quick access in the mockup
                ui.separator();
                ui.label(egui::RichText::new("SPAWN OBJECTS").color(egui::Color32::from_rgb(180, 150, 255)).small());
                ui.horizontal_wrapped(|ui| {
                    for kind in [
                        ObjectKind::Note,
                        ObjectKind::App,
                        ObjectKind::Folder,
                        ObjectKind::Agent,
                        ObjectKind::Link,
                    ] {
                        let btn = ui.button(format!("{} {}", kind.symbol(), kind.label()));
                        if btn.clicked() {
                            self.spawn_object(kind, WorldPoint::new(0.0, 0.0));
                        }
                    }
                });

                ui.add_space(6.0);
                ui.separator();

                // AI section (dashboard style)
                ui.label(egui::RichText::new("AI RUNTIME").color(egui::Color32::from_rgb(180, 150, 255)).small());
                if ui.button("Ping local agent").clicked() {
                    match self.agent_runtime.ping() {
                        Ok(reply) => self.status = reply,
                        Err(err) => self.status = err.to_string(),
                    }
                }
                if ui.button("Ask about this dimension").clicked() {
                    let prompt = self
                        .state
                        .active_dimension()
                        .map(|d| format!("Describe dimension '{}'", d.name))
                        .unwrap_or_else(|| "Describe the workspace".into());

                    match self.agent_runtime.complete(AgentMessage::user(prompt)) {
                        Ok(reply) => self.status = reply.text,
                        Err(err) => self.status = err.to_string(),
                    }
                }

                ui.add_space(6.0);
                ui.separator();

                // FS / System info (clean like the mockup's system panels)
                ui.label(egui::RichText::new("HYPERSPACE FS").color(egui::Color32::from_rgb(180, 150, 255)).small());
                ui.label(egui::RichText::new(format!("Store: {}", self.workspace_store.path().display())).small().color(egui::Color32::from_rgb(150, 160, 190)));
                if ui.button("Sync store").clicked() {
                    self.sync_active_dimension();
                    if let Ok(entries) = self.store.list_active(self.active_dimension_id()) {
                        self.status = format!("Synced {} objects", entries.len());
                    }
                }
                if let Ok(entries) = self.store.list_active(self.active_dimension_id()) {
                    ui.label(egui::RichText::new(format!("{} objects in active dimension", entries.len())).small());
                }
            });
    }

    fn bottom_dock(&mut self, ctx: &egui::Context) {
        // Bottom dock / taskbar — modern OS feel (quick actions + active dimension indicator).
        // Declared as a sibling panel BEFORE the CentralPanel so it reserves its own strip and
        // does NOT paint over the canvas (previously it occluded the minimap + zoom overlay).
        egui::TopBottomPanel::bottom("dock")
            .frame(egui::Frame::default()
                .fill(egui::Color32::from_rgb(6, 8, 16))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 90))))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new("⌘").small());
                    ui.label(egui::RichText::new("Quick Dock").small().color(egui::Color32::from_rgb(150, 160, 190)));
                    ui.add_space(12.0);

                    if ui.button("New Note").clicked() {
                        if let Some(dim) = self.state.active_dimension_mut() {
                            let pos = WorldPoint::new(-dim.viewport.pan_x, -dim.viewport.pan_y);
                            let obj = SmartObject::note("Quick Note", snap_point(pos)).with_body("Created from dock");
                            self.selected_object = Some(obj.id);
                            dim.objects.push(obj);
                            self.mark_dirty();
                            self.sync_active_dimension();
                        }
                    }
                    if ui.button("Fit to content").clicked() {
                        self.pending_fit = true;
                    }
                    if ui.button("AI Chat").clicked() {
                        self.status = "Use left HUD 'Ask about this dimension' for now.".into();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("Dim: {}", self.state.active_dimension().map(|d| d.name.as_str()).unwrap_or("?"))).small().color(egui::Color32::from_rgb(140, 150, 180)));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("OS/3 Hyperspace • Local-first").small().color(egui::Color32::from_rgb(100, 110, 140)));
                    });
                });
            });
    }

    fn inspector_panel(&mut self, ctx: &egui::Context) {
        let Some(selected_id) = self.selected_object else {
            return;
        };

        let dimension_id = self.active_dimension_id();
        let mut delete_requested = false;
        let mut changed = false;

        // Collect other dims for Link targeting (before mutable borrow of object)
        let other_dims: Vec<(DimensionId, String)> = self
            .state
            .dimensions
            .iter()
            .filter(|d| d.id != self.state.active_dimension)
            .map(|d| (d.id, d.name.clone()))
            .collect();

        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(280.0)
            .frame(egui::Frame::default()
                .fill(egui::Color32::from_rgb(8, 10, 18))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 45, 75))))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("INSPECTOR").strong().color(egui::Color32::from_rgb(180, 150, 255)).small());
                ui.add_space(4.0);

                let Some(object) = self.state.find_object_mut(dimension_id, selected_id) else {
                    ui.label("Object not found.");
                    return;
                };

                // Kind + icon header like modern property panels
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(object.kind.symbol()).size(18.0));
                    ui.label(egui::RichText::new(object.kind.label()).strong().color(egui::Color32::from_rgb(220, 210, 255)));
                });
                ui.separator();

                if object.kind == ObjectKind::Link {
                    ui.label(egui::RichText::new("LINK TARGET").small().color(egui::Color32::from_rgb(160, 170, 200)));
                    for (tid, tname) in &other_dims {
                        let is_set = object.link_target == Some(*tid);
                        let label = if is_set { format!("● {}", tname) } else { tname.clone() };
                        if ui.selectable_label(is_set, label).clicked() {
                            object.link_target = Some(*tid);
                            changed = true;
                        }
                    }
                    if ui.button(egui::RichText::new("Clear Target").small()).clicked() {
                        object.link_target = None;
                        changed = true;
                    }
                    ui.add_space(4.0);
                }

                changed |= ui
                    .text_edit_singleline(&mut object.title)
                    .changed();
                changed |= ui
                    .add(
                        egui::TextEdit::multiline(&mut object.body)
                            .desired_rows(8)
                            .hint_text("Body"),
                    )
                    .changed();

                ui.separator();
                ui.label(format!(
                    "Position: ({:.0}, {:.0})",
                    object.position.x, object.position.y
                ));

                // Editable size (for resize + direct entry) — clean modern controls
                ui.label(egui::RichText::new("SIZE").small().color(egui::Color32::from_rgb(160, 170, 200)));
                ui.horizontal(|ui| {
                    let mut w = object.size.width;
                    let mut h = object.size.height;
                    let w_changed = ui
                        .add(
                            egui::DragValue::new(&mut w)
                                .speed(2.0)
                                .range(50.0..=4000.0)
                                .suffix(" w"),
                        )
                        .changed();
                    let h_changed = ui
                        .add(
                            egui::DragValue::new(&mut h)
                                .speed(2.0)
                                .range(50.0..=4000.0)
                                .suffix(" h"),
                        )
                        .changed();
                    if w_changed || h_changed {
                        object.size.width = w.max(50.0);
                        object.size.height = h.max(50.0);
                        changed = true;
                    }
                });

                ui.add_space(6.0);

                if ui.button(egui::RichText::new("Delete Object").color(egui::Color32::from_rgb(255, 140, 140))).clicked() {
                    delete_requested = true;
                }
            });

        if changed {
            self.mark_dirty();
            self.sync_active_dimension();
        }

        if delete_requested {
            self.delete_selected();
        }
    }
}

impl eframe::App for HyperspaceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.top_bar(ctx);
        self.side_panel(ctx);
        self.inspector_panel(ctx);
        self.bottom_dock(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let screen_size = WorldSize {
                width: ui.available_width(),
                height: ui.available_height(),
            };

            // Apply a deferred "fit to content" now that we know the live canvas size.
            if self.pending_fit {
                self.pending_fit = false;
                if let Some(dimension) = self.state.active_dimension_mut() {
                    fit_viewport_to_content(&mut dimension.viewport, &dimension.objects, screen_size);
                    self.status = "Fit view to content".into();
                }
            }

            let mut pending_agent_prompt: Option<String> = None;
            let mut pending_link_nav: Option<DimensionId> = None;
            let mut pending_close: Option<SmartObjectId> = None;
            let mut moved_ids: Vec<(SmartObjectId, WorldPoint)> = Vec::new();
            let mut new_objects: Vec<SmartObject> = Vec::new();
            let mut selected_object = self.selected_object;
            let mut status_update: Option<String> = None;
            let mut dirty = false;

            if let Some(dimension) = self.state.active_dimension_mut() {
                let events = self.canvas.handle_input(
                    ui,
                    &mut dimension.viewport,
                    &mut dimension.objects,
                    screen_size,
                    selected_object, // pass for resize handle priority on selected
                );

                for event in events {
                    match event {
                        CanvasEvent::Created { kind, at } => {
                            let title = match kind {
                                ObjectKind::Note => "New Note",
                                ObjectKind::App => "New App",
                                ObjectKind::Folder => "New Folder",
                                ObjectKind::Agent => "New Agent",
                                ObjectKind::Link => "New Link",
                            };
                            let object = SmartObject::new(kind, title, snap_point(at))
                                .with_body("Smart Object placeholder");
                            selected_object = Some(object.id);
                            new_objects.push(object);
                            status_update = Some(format!("Created {title}"));
                            dirty = true;
                        }
                        CanvasEvent::Moved { id, to } => {
                            moved_ids.push((id, snap_point(to)));
                        }
                        CanvasEvent::Resized { id, position, size } => {
                            // App will apply snap + update; we may have live mutated in canvas for feel
                            // Collect or apply similar to move. For simplicity apply here with snap.
                            if let Some(obj) = dimension.objects.iter_mut().find(|o| o.id == id) {
                                obj.position = snap_point(position);
                                // Snap size lightly to grid for nice alignment (optional)
                                obj.size.width = ((size.width / GRID_SNAP).round() * GRID_SNAP).max(50.0);
                                obj.size.height = ((size.height / GRID_SNAP).round() * GRID_SNAP).max(50.0);
                                status_update = Some(format!("Resized {}", obj.title));
                                dirty = true;
                            }
                        }
                        CanvasEvent::Selected(id) => {
                            selected_object = Some(id);
                            if let Some(object) = dimension.objects.iter().find(|o| o.id == id) {
                                status_update = Some(format!("Selected: {}", object.title));
                            }
                        }
                        CanvasEvent::Deselected => {
                            selected_object = None;
                        }
                        CanvasEvent::AgentInvoke(id) => {
                            selected_object = Some(id);
                            if let Some(object) = dimension.objects.iter().find(|o| o.id == id) {
                                pending_agent_prompt = Some(format!(
                                    "You are a local-first agent inside OS/3 Hyperspace. Respond briefly about object '{}'.",
                                    object.title
                                ));
                            }
                        }
                        CanvasEvent::LinkActivate(id) => {
                            selected_object = Some(id);
                            if let Some(object) = dimension.objects.iter().find(|o| o.id == id) {
                                if let Some(target) = object.link_target {
                                    // Defer to after the dimension borrow ends (avoid E0499/E0502)
                                    pending_link_nav = Some(target);
                                    status_update = Some(format!("Navigating via Link: {}", object.title));
                                } else {
                                    status_update = Some("Link has no target set (use Inspector)".into());
                                }
                            }
                        }
                        CanvasEvent::CloseObject(id) => {
                            // Close button on selected object's titlebar chrome was clicked.
                            // Defer to avoid borrow conflict with active_dimension_mut (like LinkActivate).
                            pending_close = Some(id);
                            status_update = Some("Closing object...".into());
                        }
                    }
                }

                dimension.objects.extend(new_objects);

                for (id, to) in moved_ids {
                    if let Some(object) = dimension.objects.iter_mut().find(|o| o.id == id) {
                        object.position = to;
                        status_update = Some(format!("Moved {}", object.title));
                        dirty = true;
                    }
                }

                canvas::draw_canvas(ui, dimension, screen_size, selected_object);
            }

            self.selected_object = selected_object;
            if let Some(status) = status_update {
                self.status = status;
            }
            if dirty {
                self.mark_dirty();
                self.sync_active_dimension();
            }

            if let Some(prompt) = pending_agent_prompt {
                match self.agent_runtime.complete(AgentMessage::user(prompt)) {
                    Ok(reply) => self.status = reply.text,
                    Err(err) => self.status = err.to_string(),
                }
            }

            if let Some(target) = pending_link_nav {
                // Now safe to call switch (no active dim mut borrow)
                self.switch_dimension(target);
            }

            if let Some(id) = pending_close {
                // Safe to mutate state now
                if self.state.remove_object(self.active_dimension_id(), id) {
                    self.selected_object = None;
                    self.status = "Closed object".into();
                    self.mark_dirty();
                    self.sync_active_dimension();
                }
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.dirty {
            self.save_workspace();
        }
    }
}

fn snap_point(point: WorldPoint) -> WorldPoint {
    WorldPoint::new(
        (point.x / GRID_SNAP).round() * GRID_SNAP,
        (point.y / GRID_SNAP).round() * GRID_SNAP,
    )
}

/// Center + zoom a viewport so every object in the dimension fits on screen, with margin.
/// No-op for empty dimensions or a zero-sized canvas. Zoom is clamped to the viewport limits.
fn fit_viewport_to_content(viewport: &mut Viewport, objects: &[SmartObject], screen: WorldSize) {
    if objects.is_empty() || screen.width <= 0.0 || screen.height <= 0.0 {
        return;
    }

    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for o in objects {
        min_x = min_x.min(o.position.x);
        min_y = min_y.min(o.position.y);
        max_x = max_x.max(o.position.x + o.size.width);
        max_y = max_y.max(o.position.y + o.size.height);
    }

    let margin = 80.0;
    let content_w = (max_x - min_x).max(1.0) + margin * 2.0;
    let content_h = (max_y - min_y).max(1.0) + margin * 2.0;
    let zoom = (screen.width / content_w)
        .min(screen.height / content_h)
        .clamp(Viewport::MIN_ZOOM, Viewport::MAX_ZOOM);

    // world_to_screen(center) = (center + pan) * zoom + screen_center; set pan = -center so the
    // content centroid lands at the screen centre.
    viewport.zoom = zoom;
    viewport.pan_x = -(min_x + max_x) * 0.5;
    viewport.pan_y = -(min_y + max_y) * 0.5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_centers_content_on_screen() {
        let screen = WorldSize { width: 1000.0, height: 800.0 };
        let objects = vec![
            SmartObject::note("a", WorldPoint::new(100.0, 200.0)),
            SmartObject::note("b", WorldPoint::new(600.0, 500.0)),
        ];
        let mut viewport = Viewport::default();
        fit_viewport_to_content(&mut viewport, &objects, screen);

        // Content centroid should map to the screen centre.
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for o in &objects {
            min_x = min_x.min(o.position.x);
            min_y = min_y.min(o.position.y);
            max_x = max_x.max(o.position.x + o.size.width);
            max_y = max_y.max(o.position.y + o.size.height);
        }
        let centroid = WorldPoint::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
        let mapped = viewport.world_to_screen(centroid, screen);
        assert!((mapped.x - screen.width * 0.5).abs() < 0.001);
        assert!((mapped.y - screen.height * 0.5).abs() < 0.001);
        assert!(viewport.zoom > 0.0 && viewport.zoom <= Viewport::MAX_ZOOM);
    }

    #[test]
    fn fit_is_noop_when_empty() {
        let mut viewport = Viewport { pan_x: 5.0, pan_y: 6.0, zoom: 1.5 };
        fit_viewport_to_content(&mut viewport, &[], WorldSize { width: 800.0, height: 600.0 });
        assert_eq!(viewport.pan_x, 5.0);
        assert_eq!(viewport.zoom, 1.5);
    }
}
