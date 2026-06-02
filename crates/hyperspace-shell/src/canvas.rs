//! Canvas rendering and input handling for the Hyperspace shell prototype.
//!
//! This module owns the infinite zoomable 2D world:
//! - Pan/zoom via Viewport math (see hyperspace-core).
//! - Object hit testing, move, and (new) resize via corner handles.
//! - Special interactions: double-click spawn (Notes), Agent clicks, Link activation.
//! - Drawing: grid, objects (with accent headers), selected resize handles, minimap, status overlay.
//!
//! Key recent additions (2026-06-03):
//! - Resize support: ResizeCorner, resizing state, hit_resize_handle, compute_resized,
//!   draw_resize_handles, Resized event.
//! - Link activation: LinkActivate event emitted on click for Link-kind objects.
//! - Shared helpers (object_screen_rect) to keep screen<->world consistent.
//!
//! The app (app.rs) drives the state machine: it passes `selected`, receives events,
//! applies snaps/dirty/persistence, and renders via draw_canvas.
//!
//! See docs/DEVELOPMENT-LOG.md and docs/smart-objects.md for full rationale + usage.

use eframe::egui;
use hyperspace_core::{
    Dimension, ObjectKind, SmartObject, SmartObjectId, Viewport, WorldPoint, WorldSize,
};

#[derive(Default)]
pub struct CanvasInteraction {
    dragging_object: Option<SmartObjectId>,
    drag_offset: Option<WorldPoint>,
    space_pan: bool,
    resizing: Option<(SmartObjectId, ResizeCorner, WorldPoint, WorldPoint, WorldSize)>, // (id, corner, start_pointer_world, start_pos, start_size)
}

/// Which corner is being dragged for resize. Opposite corner (or edge) stays fixed.
/// Used internally by CanvasInteraction for resize state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResizeCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

const MIN_OBJECT_DIM: f32 = 50.0;

pub enum CanvasEvent {
    Created {
        kind: ObjectKind,
        at: WorldPoint,
    },
    Moved {
        id: SmartObjectId,
        to: WorldPoint,
    },
    Resized {
        id: SmartObjectId,
        position: WorldPoint,
        size: WorldSize,
    },
    Selected(SmartObjectId),
    Deselected,
    AgentInvoke(SmartObjectId),
    LinkActivate(SmartObjectId),
}

impl CanvasInteraction {
    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        viewport: &mut Viewport,
        objects: &mut [SmartObject],
        screen_size: WorldSize,
        selected: Option<SmartObjectId>,
    ) -> Vec<CanvasEvent> {
        let mut events = Vec::new();
        let rect = ui.max_rect();
        let response = ui.interact(rect, ui.id().with("canvas"), egui::Sense::click_and_drag());

        self.space_pan = ui.input(|i| i.key_down(egui::Key::Space));

        if response.hovered() || response.dragged() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                    let anchor = WorldPoint::new(pointer.x - rect.min.x, pointer.y - rect.min.y);
                    let factor = (scroll * 0.002).exp();
                    viewport.zoom_at(anchor, screen_size, factor);
                }
            }
        }

        let middle_drag = ui.input(|i| i.pointer.middle_down() && i.pointer.is_moving());
        let space_drag = self.space_pan && response.dragged_by(egui::PointerButton::Primary);
        let pan_delta = if middle_drag || space_drag {
            ui.input(|i| i.pointer.delta())
        } else {
            egui::Vec2::ZERO
        };

        if pan_delta != egui::Vec2::ZERO {
            viewport.pan_by_screen_delta(pan_delta.x, pan_delta.y);
        }

        if response.double_clicked() && !self.space_pan {
            if let Some(pointer) = response.interact_pointer_pos() {
                let screen = WorldPoint::new(pointer.x - rect.min.x, pointer.y - rect.min.y);
                let world = viewport.screen_to_world(screen, screen_size);
                events.push(CanvasEvent::Created {
                    kind: ObjectKind::Note,
                    at: world,
                });
            }
        }

        if response.clicked() && !self.space_pan {
            if let Some(pointer) = response.interact_pointer_pos() {
                let screen = WorldPoint::new(pointer.x - rect.min.x, pointer.y - rect.min.y);
                let world = viewport.screen_to_world(screen, screen_size);

                if let Some(object) = hit_test(objects, world) {
                    events.push(CanvasEvent::Selected(object.id));
                    if object.kind == ObjectKind::Agent {
                        events.push(CanvasEvent::AgentInvoke(object.id));
                    } else if object.kind == ObjectKind::Link {
                        events.push(CanvasEvent::LinkActivate(object.id));
                    }
                } else {
                    events.push(CanvasEvent::Deselected);
                }
            }
        }

        if response.drag_started_by(egui::PointerButton::Primary) && !self.space_pan {
            if let Some(pointer) = response.interact_pointer_pos() {
                let screen = WorldPoint::new(pointer.x - rect.min.x, pointer.y - rect.min.y);
                let world = viewport.screen_to_world(screen, screen_size);

                // Priority: resize handles on the currently selected object
                if let Some(sel_id) = selected {
                    if let Some(obj) = objects.iter().find(|o| o.id == sel_id) {
                        if let Some(corner) = hit_resize_handle(obj, screen, viewport, screen_size, rect.min) {
                            self.resizing = Some((
                                sel_id,
                                corner,
                                world,
                                obj.position,
                                obj.size,
                            ));
                            events.push(CanvasEvent::Selected(sel_id));
                            // do not fall through to move
                            // continue to next input handling
                        } else if let Some(object) = hit_test(objects, world) {
                            self.dragging_object = Some(object.id);
                            self.drag_offset = Some(WorldPoint::new(
                                world.x - object.position.x,
                                world.y - object.position.y,
                            ));
                            events.push(CanvasEvent::Selected(object.id));
                        }
                    }
                } else if let Some(object) = hit_test(objects, world) {
                    self.dragging_object = Some(object.id);
                    self.drag_offset = Some(WorldPoint::new(
                        world.x - object.position.x,
                        world.y - object.position.y,
                    ));
                    events.push(CanvasEvent::Selected(object.id));
                }
            }
        }

        if response.dragged_by(egui::PointerButton::Primary) && !self.space_pan {
            if let Some((id, corner, start_ptr, start_pos, start_sz)) = self.resizing {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let screen = WorldPoint::new(pointer.x - rect.min.x, pointer.y - rect.min.y);
                    let cur_world = viewport.screen_to_world(screen, screen_size);
                    let (new_pos, new_size) =
                        compute_resized(corner, start_ptr, start_pos, start_sz, cur_world);

                    // Emit; app applies with snap + marks dirty (matches move event style)
                    events.push(CanvasEvent::Resized {
                        id,
                        position: new_pos,
                        size: new_size,
                    });
                }
            } else if let Some(id) = self.dragging_object {
                if let Some(offset) = self.drag_offset {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let screen = WorldPoint::new(pointer.x - rect.min.x, pointer.y - rect.min.y);
                        let world = viewport.screen_to_world(screen, screen_size);
                        let to = WorldPoint::new(
                            world.x - offset.x,
                            world.y - offset.y,
                        );
                        events.push(CanvasEvent::Moved { id, to });
                    }
                }
            }
        }

        if response.drag_stopped() {
            self.dragging_object = None;
            self.drag_offset = None;
            self.resizing = None;
        }

        events
    }
}

fn hit_test(objects: &[SmartObject], point: WorldPoint) -> Option<&SmartObject> {
    objects
        .iter()
        .rev()
        .find(|object| object.contains(point))
}

pub fn draw_canvas(
    ui: &mut egui::Ui,
    dimension: &Dimension,
    screen_size: WorldSize,
    selected: Option<SmartObjectId>,
) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    let viewport = dimension.viewport;

    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(12, 16, 28));

    draw_grid(&painter, rect, viewport, screen_size);

    for object in &dimension.objects {
        let is_selected = selected == Some(object.id);
        draw_object(&painter, object, viewport, screen_size, rect.min, is_selected);
    }

    if let Some(sel_id) = selected {
        if let Some(obj) = dimension.objects.iter().find(|o| o.id == sel_id) {
            draw_resize_handles(&painter, obj, &viewport, screen_size, rect.min);
        }
    }

    draw_minimap(&painter, rect, dimension, selected);
    draw_overlay(&painter, rect, viewport, &dimension.name);
}

fn draw_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    viewport: Viewport,
    screen_size: WorldSize,
) {
    let base_spacing = 80.0;
    let spacing = base_spacing * viewport.zoom.max(0.25);
    if spacing < 8.0 {
        return;
    }

    let top_left = viewport.screen_to_world(WorldPoint::new(0.0, 0.0), screen_size);
    let bottom_right = viewport.screen_to_world(
        WorldPoint::new(screen_size.width, screen_size.height),
        screen_size,
    );

    let start_x = (top_left.x / base_spacing).floor() * base_spacing;
    let end_x = (bottom_right.x / base_spacing).ceil() * base_spacing;
    let start_y = (top_left.y / base_spacing).floor() * base_spacing;
    let end_y = (bottom_right.y / base_spacing).ceil() * base_spacing;

    let grid_color = egui::Color32::from_rgba_unmultiplied(120, 140, 180, 28);
    let mut x = start_x;
    while x <= end_x {
        let screen = viewport.world_to_screen(WorldPoint::new(x, 0.0), screen_size);
        let from = egui::pos2(rect.min.x + screen.x, rect.min.y);
        let to = egui::pos2(rect.min.x + screen.x, rect.max.y);
        painter.line_segment([from, to], egui::Stroke::new(1.0, grid_color));
        x += base_spacing;
    }

    let mut y = start_y;
    while y <= end_y {
        let screen = viewport.world_to_screen(WorldPoint::new(0.0, y), screen_size);
        let from = egui::pos2(rect.min.x, rect.min.y + screen.y);
        let to = egui::pos2(rect.max.x, rect.min.y + screen.y);
        painter.line_segment([from, to], egui::Stroke::new(1.0, grid_color));
        y += base_spacing;
    }
}

fn draw_object(
    painter: &egui::Painter,
    object: &SmartObject,
    viewport: Viewport,
    screen_size: WorldSize,
    origin: egui::Pos2,
    selected: bool,
) {
    let top_left = viewport.world_to_screen(object.position, screen_size);
    let bottom_right = viewport.world_to_screen(
        WorldPoint::new(
            object.position.x + object.size.width,
            object.position.y + object.size.height,
        ),
        screen_size,
    );

    let rect = egui::Rect::from_min_max(
        origin + egui::vec2(top_left.x, top_left.y),
        origin + egui::vec2(bottom_right.x, bottom_right.y),
    );

    if rect.width() < 4.0 || rect.height() < 4.0 {
        return;
    }

    let accent = object.kind.accent();
    let fill = egui::Color32::from_rgba_unmultiplied(accent[0], accent[1], accent[2], if selected { 64 } else { 36 });
    let stroke_color = if selected {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(accent[0], accent[1], accent[2])
    };
    let stroke_width = if selected { 2.5 } else { 1.5 };

    painter.rect(
        rect,
        10.0,
        fill,
        egui::Stroke::new(stroke_width, stroke_color),
        egui::StrokeKind::Inside,
    );

    // Futuristic accent header strip
    let accent_col = egui::Color32::from_rgb(accent[0], accent[1], accent[2]);
    let header_bar = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 18.0));
    painter.rect_filled(header_bar, 10.0, egui::Color32::from_rgba_unmultiplied(accent[0], accent[1], accent[2], 70));
    // thin top line for depth
    painter.line_segment(
        [header_bar.min, egui::pos2(header_bar.max.x, header_bar.min.y)],
        egui::Stroke::new(1.0, accent_col),
    );

    let header = rect.shrink2(egui::vec2(12.0, 10.0));
    painter.text(
        header.min + egui::vec2(0.0, 0.0),
        egui::Align2::LEFT_TOP,
        format!("{} · {}", object.kind.label(), object.title),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );

    if rect.height() > 48.0 {
        painter.text(
            header.min + egui::vec2(0.0, 22.0),
            egui::Align2::LEFT_TOP,
            &object.body,
            egui::FontId::proportional(12.0),
            egui::Color32::from_gray(200),
        );
    }
}

/// Draw 4 corner resize handles (small white squares with stroke) when object is selected.
/// Called from draw_canvas only for the selected object. Handles are fixed screen size (~7px)
/// so they remain usable at any zoom level. Early-returns for tiny objects.
fn draw_resize_handles(
    painter: &egui::Painter,
    object: &SmartObject,
    viewport: &Viewport,
    screen_size: WorldSize,
    origin: egui::Pos2,
) {
    let screen_rect = object_screen_rect(object, viewport, screen_size, origin);
    if screen_rect.width() < 12.0 || screen_rect.height() < 12.0 {
        return;
    }
    let h = 7.0_f32;
    let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    let fill = egui::Color32::from_rgb(255, 255, 255);
    let corners = [
        screen_rect.min,
        screen_rect.min + egui::vec2(screen_rect.width(), 0.0),
        screen_rect.min + egui::vec2(0.0, screen_rect.height()),
        screen_rect.min + egui::vec2(screen_rect.width(), screen_rect.height()),
    ];
    for c in corners {
        let r = egui::Rect::from_center_size(c, egui::vec2(h, h));
        painter.rect_filled(r, 1.0, fill);
        painter.rect_stroke(r, 1.0, stroke, egui::StrokeKind::Inside);
    }
}

fn draw_minimap(
    painter: &egui::Painter,
    rect: egui::Rect,
    dimension: &Dimension,
    selected: Option<SmartObjectId>,
) {
    if dimension.objects.is_empty() {
        return;
    }

    let map_size = egui::vec2(140.0, 90.0);
    let map_rect = egui::Rect::from_min_size(
        rect.max - egui::vec2(map_size.x + 16.0, map_size.y + 16.0),
        map_size,
    );

    painter.rect_filled(
        map_rect,
        6.0,
        egui::Color32::from_rgba_unmultiplied(8, 12, 22, 200),
    );
    painter.rect_stroke(
        map_rect,
        6.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
        egui::StrokeKind::Inside,
    );

    let (min_x, min_y, max_x, max_y) = dimension.objects.iter().fold(
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
        |(min_x, min_y, max_x, max_y), object| {
            (
                min_x.min(object.position.x),
                min_y.min(object.position.y),
                max_x.max(object.position.x + object.size.width),
                max_y.max(object.position.y + object.size.height),
            )
        },
    );

    let pad = 40.0;
    let world_w = (max_x - min_x + pad * 2.0).max(1.0);
    let world_h = (max_y - min_y + pad * 2.0).max(1.0);
    let scale = (map_size.x / world_w).min(map_size.y / world_h);

    let to_map = |world: WorldPoint| {
        egui::pos2(
            map_rect.min.x + (world.x - min_x + pad) * scale,
            map_rect.min.y + (world.y - min_y + pad) * scale,
        )
    };

    for object in &dimension.objects {
        let top_left = to_map(object.position);
        let bottom_right = to_map(WorldPoint::new(
            object.position.x + object.size.width,
            object.position.y + object.size.height,
        ));
        let accent = object.kind.accent();
        let color = egui::Color32::from_rgb(accent[0], accent[1], accent[2]);
        let object_rect = egui::Rect::from_min_max(top_left, bottom_right);
        painter.rect_filled(object_rect, 2.0, color);
        if selected == Some(object.id) {
            painter.rect_stroke(
                object_rect.expand(1.0),
                2.0,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
                egui::StrokeKind::Inside,
            );
        }
    }
}

fn draw_overlay(painter: &egui::Painter, rect: egui::Rect, viewport: Viewport, name: &str) {
    let overlay = egui::Rect::from_min_size(
        rect.min + egui::vec2(16.0, rect.height() - 40.0),
        egui::vec2(320.0, 24.0),
    );
    painter.text(
        overlay.min,
        egui::Align2::LEFT_TOP,
        format!("Dimension: {name}   ·   Zoom: {:.0}%", viewport.zoom * 100.0),
        egui::FontId::monospace(12.0),
        egui::Color32::from_gray(170),
    );
}

/// Compute screen rect (in painter local coords) for an object.
/// Shared helper used by draw_object, draw_resize_handles, hit_resize_handle, and input logic.
/// Keeps screen math in one place so zoom/pan changes are consistent.
fn object_screen_rect(
    object: &SmartObject,
    viewport: &Viewport,
    screen_size: WorldSize,
    origin: egui::Pos2,
) -> egui::Rect {
    let tl = viewport.world_to_screen(object.position, screen_size);
    let br = viewport.world_to_screen(
        WorldPoint::new(
            object.position.x + object.size.width,
            object.position.y + object.size.height,
        ),
        screen_size,
    );
    egui::Rect::from_min_max(
        origin + egui::vec2(tl.x, tl.y),
        origin + egui::vec2(br.x, br.y),
    )
}

/// Hit test for resize handles on a selected object's screen rect.
/// Returns the corner if pointer (in screen local coords, relative to canvas origin) hits a handle.
/// Uses generous 12px screen-pixel handles (zoom-independent) for usability.
/// Called only for the currently selected object (passed from app).
fn hit_resize_handle(
    object: &SmartObject,
    pointer_screen: WorldPoint,
    viewport: &Viewport,
    screen_size: WorldSize,
    origin: egui::Pos2,
) -> Option<ResizeCorner> {
    let screen_rect = object_screen_rect(object, viewport, screen_size, origin);
    if screen_rect.width() < 20.0 || screen_rect.height() < 20.0 {
        return None;
    }
    let h = 12.0_f32; // screen pixels, generous hit area
    let corners = [
        (ResizeCorner::TopLeft, screen_rect.min),
        (ResizeCorner::TopRight, screen_rect.min + egui::vec2(screen_rect.width(), 0.0)),
        (ResizeCorner::BottomLeft, screen_rect.min + egui::vec2(0.0, screen_rect.height())),
        (
            ResizeCorner::BottomRight,
            screen_rect.min + egui::vec2(screen_rect.width(), screen_rect.height()),
        ),
    ];
    let p = egui::pos2(pointer_screen.x, pointer_screen.y);
    for (corner, c) in corners {
        let hr = egui::Rect::from_center_size(c, egui::vec2(h, h));
        if hr.contains(p) {
            return Some(corner);
        }
    }
    None
}

/// Given a drag, compute new (position, size) for the corner being dragged.
/// Clamps resulting dimensions to MIN_OBJECT_DIM. Does *not* snap here
/// (snapping + dirty marking happens in app.rs when processing the emitted Resized event).
/// Supports all four corners by adjusting the "fixed" corner implicitly via deltas.
fn compute_resized(
    corner: ResizeCorner,
    start_pointer: WorldPoint,
    start_pos: WorldPoint,
    start_size: WorldSize,
    cur_pointer: WorldPoint,
) -> (WorldPoint, WorldSize) {
    let dx = cur_pointer.x - start_pointer.x;
    let dy = cur_pointer.y - start_pointer.y;
    let mut np = start_pos;
    let mut ns = start_size;

    match corner {
        ResizeCorner::TopLeft => {
            np.x = start_pos.x + dx;
            np.y = start_pos.y + dy;
            ns.width = (start_size.width - dx).max(MIN_OBJECT_DIM);
            ns.height = (start_size.height - dy).max(MIN_OBJECT_DIM);
        }
        ResizeCorner::TopRight => {
            np.y = start_pos.y + dy;
            ns.width = (start_size.width + dx).max(MIN_OBJECT_DIM);
            ns.height = (start_size.height - dy).max(MIN_OBJECT_DIM);
        }
        ResizeCorner::BottomLeft => {
            np.x = start_pos.x + dx;
            ns.width = (start_size.width - dx).max(MIN_OBJECT_DIM);
            ns.height = (start_size.height + dy).max(MIN_OBJECT_DIM);
        }
        ResizeCorner::BottomRight => {
            ns.width = (start_size.width + dx).max(MIN_OBJECT_DIM);
            ns.height = (start_size.height + dy).max(MIN_OBJECT_DIM);
        }
    }
    (np, ns)
}
