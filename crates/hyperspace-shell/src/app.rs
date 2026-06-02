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
    DimensionId, HyperspaceState, ObjectKind, SmartObject, SmartObjectId, WorldPoint, WorldSize,
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
        }
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

            if input.modifiers.command && input.key_pressed(egui::Key::S) {
                self.save_workspace();
            }
        });
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("OS/3 Hyperspace");
                if self.dirty {
                    ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(255, 196, 86)));
                }
                ui.separator();

                let tabs: Vec<(DimensionId, String)> = self
                    .state
                    .dimensions
                    .iter()
                    .map(|dimension| (dimension.id, dimension.name.clone()))
                    .collect();
                let active_id = self.active_dimension_id();
                for (id, name) in tabs {
                    let selected = id == active_id;
                    if ui.selectable_label(selected, name).clicked() {
                        self.switch_dimension(id);
                    }
                }

                ui.separator();
                ui.label("New:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_dimension_name)
                        .desired_width(100.0)
                        .hint_text("Name"),
                );
                if ui.button("+ Dimension").clicked() {
                    let name = if self.new_dimension_name.trim().is_empty() {
                        format!("Dimension {}", self.state.dimensions.len() + 1)
                    } else {
                        self.new_dimension_name.trim().to_string()
                    };
                    self.state.add_dimension(name);
                    self.new_dimension_name.clear();
                    self.selected_object = None;
                    self.status = "Created dimension".into();
                    self.mark_dirty();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save").clicked() {
                        self.save_workspace();
                    }
                    if ui.button("Toggle HUD").clicked() {
                        self.show_hud = !self.show_hud;
                    }
                    ui.label(&self.status);
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
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.label("Scroll — zoom at cursor");
                ui.label("Middle-drag — pan canvas");
                ui.label("Space + drag — pan canvas");
                ui.label("Double-click — new note");
                ui.label("Drag object — move (snaps to grid)");
                ui.label("Drag corner handles (on selection) — resize");
                ui.label("Delete — remove selection");
                ui.label("Ctrl+S — save workspace");
                ui.separator();

                ui.heading("Spawn");
                ui.horizontal_wrapped(|ui| {
                    for kind in [
                        ObjectKind::Note,
                        ObjectKind::App,
                        ObjectKind::Folder,
                        ObjectKind::Agent,
                        ObjectKind::Link,
                    ] {
                        if ui.button(kind.label()).clicked() {
                            self.spawn_object(kind, WorldPoint::new(0.0, 0.0));
                        }
                    }
                });

                ui.separator();
                ui.heading("AI Runtime");
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

                ui.separator();
                ui.heading("Hyperspace FS");
                ui.label(format!(
                    "Store: {}",
                    self.workspace_store.path().display()
                ));
                if ui.button("Sync to in-memory store").clicked() {
                    self.sync_active_dimension();
                    if let Ok(entries) = self.store.list_active(self.active_dimension_id()) {
                        self.status = format!("Synced {} objects", entries.len());
                    }
                }

                if let Ok(entries) = self.store.list_active(self.active_dimension_id()) {
                    ui.label(format!("Stored objects: {}", entries.len()));
                }
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
            .show(ctx, |ui| {
                ui.heading("Inspector");

                let Some(object) = self.state.find_object_mut(dimension_id, selected_id) else {
                    ui.label("Object not found.");
                    return;
                };

                ui.label(format!("Type: {}", object.kind.label()));
                ui.separator();

                if object.kind == ObjectKind::Link {
                    ui.label("Link Target (click to set):");
                    for (tid, tname) in &other_dims {
                        let is_set = object.link_target == Some(*tid);
                        if ui.selectable_label(is_set, tname).clicked() {
                            object.link_target = Some(*tid);
                            changed = true;
                        }
                    }
                    if ui.button("Clear link target").clicked() {
                        object.link_target = None;
                        changed = true;
                    }
                    ui.separator();
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

                // Editable size (for resize + direct entry)
                ui.horizontal(|ui| {
                    ui.label("Size:");
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

                if ui.button("Delete object").clicked() {
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

        egui::CentralPanel::default().show(ctx, |ui| {
            let screen_size = WorldSize {
                width: ui.available_width(),
                height: ui.available_height(),
            };

            let mut pending_agent_prompt: Option<String> = None;
            let mut pending_link_nav: Option<DimensionId> = None;
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
