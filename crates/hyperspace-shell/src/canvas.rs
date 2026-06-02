use eframe::egui;
use hyperspace_core::{
    Dimension, ObjectKind, SmartObject, SmartObjectId, Viewport, WorldPoint, WorldSize,
};

#[derive(Default)]
pub struct CanvasInteraction {
    dragging_object: Option<SmartObjectId>,
    drag_offset: Option<WorldPoint>,
    space_pan: bool,
}

pub enum CanvasEvent {
    Created {
        kind: ObjectKind,
        at: WorldPoint,
    },
    Moved {
        id: SmartObjectId,
        to: WorldPoint,
    },
    Selected(SmartObjectId),
    Deselected,
    AgentInvoke(SmartObjectId),
}

impl CanvasInteraction {
    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        viewport: &mut Viewport,
        objects: &mut [SmartObject],
        screen_size: WorldSize,
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
                    if object.kind == ObjectKind::Agent {
                        events.push(CanvasEvent::Selected(object.id));
                        events.push(CanvasEvent::AgentInvoke(object.id));
                    } else {
                        events.push(CanvasEvent::Selected(object.id));
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
                if let Some(object) = hit_test(objects, world) {
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
            if let Some(id) = self.dragging_object {
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
