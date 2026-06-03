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
    CloseObject(SmartObjectId),  // For window chrome close button on selected objects
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
                    // Check for close button hit on the currently selected object's titlebar chrome
                    // (makes selected Smart Objects feel like real closable windows from the reference mockup).
                    // NOTE: hit testing happens in canvas-local coords (matching `screen` = pointer - rect.min),
                    // so we build the object rect with a ZERO origin. Using rect.min here was a bug: it put the
                    // hit zone in global screen coords, offset from the pointer by the central panel origin.
                    let is_selected = selected == Some(object.id);
                    if is_selected {
                        let local_rect = object_local_rect(object, *viewport, screen_size);
                        // Tight zone around just the close (×) glyph so minimize/maximize don't also close.
                        let close_rect = egui::Rect::from_center_size(
                            egui::pos2(local_rect.max.x - 8.0, local_rect.min.y + 8.0),
                            egui::vec2(18.0, 18.0),
                        );
                        if close_rect.contains(egui::pos2(screen.x, screen.y)) {
                            events.push(CanvasEvent::CloseObject(object.id));
                            return events; // don't also select
                        }
                    }

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
                        // Hit test in canvas-local coords (ZERO origin) to match `screen`.
                        if let Some(corner) = hit_resize_handle(obj, screen, viewport, screen_size, egui::Pos2::ZERO) {
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

        // Cursor affordance: show directional resize cursors when hovering a selected
        // object's corner handle (mac-like feedback that the handle is grabbable).
        if self.resizing.is_none() && self.dragging_object.is_none() && !self.space_pan {
            if let (Some(sel_id), Some(hover)) = (selected, ui.input(|i| i.pointer.hover_pos())) {
                if let Some(obj) = objects.iter().find(|o| o.id == sel_id) {
                    let local = WorldPoint::new(hover.x - rect.min.x, hover.y - rect.min.y);
                    if let Some(corner) =
                        hit_resize_handle(obj, local, viewport, screen_size, egui::Pos2::ZERO)
                    {
                        let icon = match corner {
                            ResizeCorner::TopLeft | ResizeCorner::BottomRight => {
                                egui::CursorIcon::ResizeNwSe
                            }
                            ResizeCorner::TopRight | ResizeCorner::BottomLeft => {
                                egui::CursorIcon::ResizeNeSw
                            }
                        };
                        ui.output_mut(|o| o.cursor_icon = icon);
                    }
                }
            }
        }

        events
    }
}

/// Object rect in canvas-local screen coords (origin at the canvas top-left).
/// Matches the pointer coords used for hit testing (`pointer - rect.min`), so it must
/// NOT add the global panel origin. Drawing uses [`object_screen_rect`] with `rect.min`.
fn object_local_rect(object: &SmartObject, viewport: Viewport, screen_size: WorldSize) -> egui::Rect {
    object_screen_rect(object, &viewport, screen_size, egui::Pos2::ZERO)
}

fn hit_test(objects: &[SmartObject], point: WorldPoint) -> Option<&SmartObject> {
    objects
        .iter()
        .rev()
        .find(|object| object.contains(point))
}

/// Draw a starfield for "Hyperspace" immersion.
/// Stars are placed in world space (anchored during pan), but we modulate count/size/alpha with zoom to simulate depth layers.
/// This is cheap (no textures) and makes the infinite canvas feel alive and vast — key to "best OS" spatial UI vs traditional desktops.
/// Deterministic-ish using simple math on world coords (no rand for reproducibility across frames).
fn draw_starfield(
    painter: &egui::Painter,
    rect: egui::Rect,
    viewport: Viewport,
    screen_size: WorldSize,
) {
    let top_left = viewport.screen_to_world(WorldPoint::new(0.0, 0.0), screen_size);
    let bottom_right = viewport.screen_to_world(
        WorldPoint::new(screen_size.width, screen_size.height),
        screen_size,
    );

    // Base density; higher zoom = "closer" stars, more visible small ones.
    let zoom = viewport.zoom.clamp(0.1, 5.0);
    let base_step = 120.0 / zoom.max(0.2);
    let layers = 3;

    for layer in 0..layers {
        let layer_factor = 1.0 + layer as f32 * 0.7;
        let step = base_step * layer_factor;
        let size = (1.5 + layer as f32 * 0.8) * (zoom.min(1.5));
        let alpha_base = (80.0 - layer as f32 * 15.0) * if zoom > 1.0 { 1.2 } else { 0.8 };
        let alpha = alpha_base.min(90.0);

        let color = egui::Color32::from_rgba_unmultiplied(200, 210, 255, alpha as u8);

        let start_x = (top_left.x / step).floor() * step - step * 2.0;
        let end_x = (bottom_right.x / step).ceil() * step + step * 2.0;
        let start_y = (top_left.y / step).floor() * step - step * 2.0;
        let end_y = (bottom_right.y / step).ceil() * step + step * 2.0;

        let mut x = start_x;
        while x < end_x {
            let mut y = start_y;
            while y < end_y {
                // Simple "hash" for slight variation per position (no floating rand).
                let hash = ((x * 12.9898 + y * 78.233).sin() * 43758.547).fract();
                let jitter_x = (hash - 0.5) * step * 0.3;
                let jitter_y = ((hash * 1.3).fract() - 0.5) * step * 0.3;

                let world = WorldPoint::new(x + jitter_x, y + jitter_y);
                let screen = viewport.world_to_screen(world, screen_size);
                let pos = egui::pos2(rect.min.x + screen.x, rect.min.y + screen.y);

                if rect.contains(pos) {
                    painter.circle_filled(pos, size, color);
                    // occasional "bright" star
                    if hash > 0.92 {
                        painter.circle_filled(pos, size * 1.8, egui::Color32::from_rgba_unmultiplied(255, 255, 255, (alpha * 0.4) as u8));
                    }
                }
                y += step;
            }
            x += step;
        }
    }

    // Subtle colored nebula "clouds" / cosmic dust for richer hyperspace background
    // (directly inspired by the beautiful nebulae, light streaks, and depth in the reference mockup).
    // Very low alpha so they don't fight the UI; they sell the "vast living space" feeling.
    let nebula_color1 = egui::Color32::from_rgba_unmultiplied(120, 80, 220, 22); // purple nebula
    let nebula_color2 = egui::Color32::from_rgba_unmultiplied(60, 160, 220, 18); // cyan nebula
    for (i, &col) in [nebula_color1, nebula_color2].iter().enumerate() {
        let n_step = 380.0 + i as f32 * 70.0;
        let n_size = 95.0 + i as f32 * 25.0;
        let ox = (viewport.pan_x * 0.15 + i as f32 * 40.0) % n_step;
        let oy = (viewport.pan_y * 0.12 + i as f32 * 55.0) % n_step;

        let mut nx = top_left.x - (top_left.x % n_step) - n_step + ox;
        while nx < bottom_right.x + n_step {
            let mut ny = top_left.y - (top_left.y % n_step) - n_step + oy;
            while ny < bottom_right.y + n_step {
                let npos = viewport.world_to_screen(WorldPoint::new(nx, ny), screen_size);
                let p = egui::pos2(rect.min.x + npos.x, rect.min.y + npos.y);
                if rect.expand(60.0).contains(p) {
                    painter.circle_filled(p, n_size * (0.7 + (i as f32 * 0.15)), col);
                }
                ny += n_step;
            }
            nx += n_step;
        }
    }

    // Additional "glowing energy streaks / light ribbons" for cinematic depth (inspired by the swirling purple energy and light streaks in the reference image).
    // A few long, thin, world-anchored lines with multi-pass glow (brighter core + soft outer).
    // Parallax with pan, intensity with zoom. Very subtle so UI stays readable.
    let energy_color = egui::Color32::from_rgba_unmultiplied(180, 100, 255, 18);
    let streak_step = 520.0;
    let num_streaks = 5;
    for s in 0..num_streaks {
        let phase = (viewport.pan_x * 0.08 + s as f32 * 97.0) % streak_step;
        let y_off = (viewport.pan_y * 0.05 + s as f32 * 73.0) % 180.0 - 90.0;
        let len = 280.0 + (s % 3) as f32 * 40.0;

        // Compute a diagonal streak in world space
        let wx = top_left.x + phase - 100.0;
        let wy = top_left.y + y_off + (s as f32 * 35.0);

        let start = viewport.world_to_screen(WorldPoint::new(wx, wy), screen_size);
        let end = viewport.world_to_screen(WorldPoint::new(wx + len, wy + 18.0), screen_size);

        let p1 = egui::pos2(rect.min.x + start.x, rect.min.y + start.y);
        let p2 = egui::pos2(rect.min.x + end.x, rect.min.y + end.y);

        if rect.expand(50.0).intersects(egui::Rect::from_two_pos(p1, p2)) {
            // Soft outer glow passes
            for g in (0..3).rev() {
                let w = 2.5 + g as f32 * 3.0;
                let a = 8 + g * 5;
                painter.line_segment([p1, p2], egui::Stroke::new(w, egui::Color32::from_rgba_unmultiplied(180, 100, 255, a)));
            }
            // Core bright line
            painter.line_segment([p1, p2], egui::Stroke::new(1.2, energy_color));
        }
    }
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

    // Very dark base for hyperspace atmosphere (stars will show through a bit).
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(8, 10, 18, 220));

    // Hyperspace immersion: dynamic starfield background.
    // Gives depth and "space" feel. Stars are world-anchored but density/brightness reacts to zoom for parallax-like effect.
    // This moves the UI from "flat canvas app" toward "best OS" spatial experience.
    draw_starfield(&painter, rect, viewport, screen_size);

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

    // Premium subtle glowing grid (less "graph paper", more futuristic HUD overlay)
    let grid_color = egui::Color32::from_rgba_unmultiplied(90, 110, 160, 22);
    let mut x = start_x;
    while x <= end_x {
        let screen = viewport.world_to_screen(WorldPoint::new(x, 0.0), screen_size);
        let from = egui::pos2(rect.min.x + screen.x, rect.min.y);
        let to = egui::pos2(rect.min.x + screen.x, rect.max.y);
        painter.line_segment([from, to], egui::Stroke::new(0.75, grid_color));
        x += base_spacing;
    }

    let mut y = start_y;
    while y <= end_y {
        let screen = viewport.world_to_screen(WorldPoint::new(0.0, y), screen_size);
        let from = egui::pos2(rect.min.x, rect.min.y + screen.y);
        let to = egui::pos2(rect.max.x, rect.min.y + screen.y);
        painter.line_segment([from, to], egui::Stroke::new(0.75, grid_color));
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
    let accent_col = egui::Color32::from_rgb(accent[0], accent[1], accent[2]);

    // === PREMIUM "MODERN GLASS CARD / FLOATING WINDOW" LOOK ===
    // Designed to feel like the sleek, glowing, high-end cards and windows in the reference mockup
    // (deep space + neon, subtle glassmorphism via layered fills, soft shadows, rich headers with icons).
    // This is the main visual upgrade to escape "basic" egui appearance while staying true to the
    // infinite-canvas + Smart Objects vision (no traditional overlapping OS windows yet).

    // 1. Soft drop shadow (simulated - egui has limited real shadow support)
    let shadow_offset = egui::vec2(4.0, 6.0);
    let shadow_rect = rect.translate(shadow_offset);
    painter.rect(
        shadow_rect,
        12.0,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 70),
        egui::Stroke::NONE,
        egui::StrokeKind::Inside,
    );

    // 2. Main glassmorphic card body (slightly inset "content" area for depth)
    let body_fill = if selected {
        egui::Color32::from_rgba_unmultiplied(12, 14, 26, 235)
    } else {
        egui::Color32::from_rgba_unmultiplied(10, 12, 22, 225)
    };
    painter.rect(
        rect,
        11.0,
        body_fill,
        egui::Stroke::new(if selected { 2.0 } else { 1.0 }, accent_col),
        egui::StrokeKind::Inside,
    );

    // 3. Subtle inner "glass" highlight / content area (gives the premium layered look from the image)
    let inner_margin = 3.0;
    let inner_rect = rect.shrink(inner_margin);
    painter.rect(
        inner_rect,
        9.0,
        egui::Color32::from_rgba_unmultiplied(18, 20, 34, 160),
        egui::Stroke::NONE,
        egui::StrokeKind::Inside,
    );

    // 4. Stronger outer neon glow when selected (like active windows in the mockup)
    if selected {
        for i in 0..3 {
            let expand = 3.0 + i as f32 * 2.5;
            let alpha = 35 - i * 8;
            painter.rect(
                rect.expand(expand),
                14.0,
                egui::Color32::from_rgba_unmultiplied(accent[0], accent[1], accent[2], alpha as u8),
                egui::Stroke::NONE,
                egui::StrokeKind::Outside,
            );
        }
    }

    // === HEADER (icon + title + kind badge, modern like the reference) ===
    let header_height = 22.0;
    let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), header_height));

    // Subtle header background strip (glass over the card)
    painter.rect_filled(
        header_rect,
        9.0,
        egui::Color32::from_rgba_unmultiplied(accent[0], accent[1], accent[2], if selected { 55 } else { 35 }),
    );

    // Icon + title (premium typography + per-kind symbol for instant recognition)
    let icon = object.kind.symbol();
    let title_text = format!("{}  {}", icon, object.title);
    painter.text(
        header_rect.min + egui::vec2(8.0, 3.0),
        egui::Align2::LEFT_TOP,
        title_text,
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(245, 248, 255),
    );

    // Small kind badge on the right of header (very "modern OS" touch from the mockup)
    let badge = object.kind.label();
    let badge_width = (badge.len() as f32 * 6.5) + 10.0;
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(header_rect.max.x - badge_width - 6.0, header_rect.min.y + 3.0),
        egui::vec2(badge_width, 16.0),
    );
    painter.rect_filled(badge_rect, 6.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25));
    painter.text(
        badge_rect.center() + egui::vec2(0.0, -1.0),
        egui::Align2::CENTER_CENTER,
        badge,
        egui::FontId::proportional(9.0),
        accent_col,
    );

    // === WINDOW CHROME CONTROLS for selected objects (makes them feel like real modern app windows in the reference) ===
    // Draw small titlebar controls on far right of header: [−] [□] [X]
    // Only for selected (active "window"). Visual only for now; interaction added in input handling.
    if selected {
        let ctrl_y = header_rect.min.y + 4.0;
        let ctrl_size = 12.0;
        let spacing = 16.0;
        let right = header_rect.max.x - 8.0;

        // Close button (X) - red-ish accent
        let close_pos = egui::pos2(right, ctrl_y + 4.0);
        painter.rect_filled(
            egui::Rect::from_center_size(close_pos, egui::vec2(ctrl_size, ctrl_size)),
            3.0,
            egui::Color32::from_rgba_unmultiplied(200, 80, 80, 180),
        );
        painter.text(
            close_pos + egui::vec2(-3.5, -4.0),
            egui::Align2::LEFT_TOP,
            "×",
            egui::FontId::proportional(11.0),
            egui::Color32::WHITE,
        );

        // Minimize and Maximize (simple lines / square) - subtle
        let min_pos = egui::pos2(right - spacing, ctrl_y + 4.0);
        painter.rect_filled(
            egui::Rect::from_center_size(min_pos, egui::vec2(ctrl_size, ctrl_size)),
            3.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60),
        );
        painter.line_segment(
            [min_pos + egui::vec2(-3.0, 3.0), min_pos + egui::vec2(3.0, 3.0)],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(200, 210, 230)),
        );

        let max_pos = egui::pos2(right - spacing * 2.0, ctrl_y + 4.0);
        painter.rect_filled(
            egui::Rect::from_center_size(max_pos, egui::vec2(ctrl_size, ctrl_size)),
            3.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60),
        );
        painter.rect_stroke(
            egui::Rect::from_center_size(max_pos, egui::vec2(6.0, 6.0)),
            1.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 210, 230)),
            egui::StrokeKind::Inside,
        );
    }

    // === BODY CONTENT ===
    if rect.height() > 52.0 {
        let body_rect = rect.shrink2(egui::vec2(10.0, 26.0)).translate(egui::vec2(0.0, 4.0));
        painter.text(
            body_rect.min,
            egui::Align2::LEFT_TOP,
            &object.body,
            egui::FontId::proportional(11.5),
            egui::Color32::from_rgb(210, 215, 230),
        );
    }

    // === SPECIAL "LIVING OBJECT" VISUALS (kept + lightly enhanced for best-OS feel) ===
    // Link as glowing hyperspace portal (directly evokes the wormhole/neon portal aesthetics in the reference image)
    if object.kind == ObjectKind::Link {
        let cx = rect.center().x;
        let cy = rect.center().y;
        let max_r = (rect.width().min(rect.height()) * 0.32).min(36.0);
        for i in 0..4 {
            let r = max_r * (0.35 + i as f32 * 0.22);
            let ring_alpha = 110 - i * 18;
            let ring_col = egui::Color32::from_rgba_unmultiplied(248, 113, 113, ring_alpha as u8);
            painter.circle_stroke(egui::pos2(cx, cy), r, egui::Stroke::new(1.8, ring_col));
        }
        painter.circle_filled(egui::pos2(cx, cy), max_r * 0.22, egui::Color32::from_rgb(18, 8, 28));
        painter.circle_filled(egui::pos2(cx, cy), max_r * 0.11, egui::Color32::from_rgb(255, 140, 160));
    }

    // Agent with stronger neural "alive" presence
    if object.kind == ObjectKind::Agent {
        let cx = rect.center().x;
        let cy = rect.center().y;
        let r = (rect.width().min(rect.height()) * 0.16).min(20.0);
        painter.circle_filled(egui::pos2(cx, cy), r * 1.1, egui::Color32::from_rgba_unmultiplied(192, 132, 252, 55));
        painter.circle_filled(egui::pos2(cx, cy), r * 0.55, egui::Color32::from_rgb(235, 200, 255));
        // Small orbiting "thought" dots for extra life
        let t = (rect.min.x * 0.03 + rect.min.y * 0.017).sin(); // cheap time-ish variation
        let ox = (t * 6.0).cos() * r * 0.7;
        let oy = (t * 5.0).sin() * r * 0.7;
        painter.circle_filled(egui::pos2(cx + ox, cy + oy), 2.5, egui::Color32::from_rgb(255, 230, 255));
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

    // Modern glass minimap frame
    painter.rect_filled(
        map_rect,
        7.0,
        egui::Color32::from_rgba_unmultiplied(6, 8, 16, 215),
    );
    painter.rect_stroke(
        map_rect,
        7.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 60, 95)),
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
        rect.min + egui::vec2(14.0, rect.height() - 32.0),
        egui::vec2(380.0, 20.0),
    );
    painter.text(
        overlay.min,
        egui::Align2::LEFT_TOP,
        format!("{}  •  {:.0}%", name, viewport.zoom * 100.0),
        egui::FontId::monospace(11.0),
        egui::Color32::from_rgb(160, 170, 200),
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
