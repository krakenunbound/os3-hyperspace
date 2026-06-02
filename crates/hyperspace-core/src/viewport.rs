use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WorldRect {
    pub origin: WorldPoint,
    pub size: WorldSize,
}

/// Camera for the infinite canvas: pan + zoom in world space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    pub const MIN_ZOOM: f32 = 0.08;
    pub const MAX_ZOOM: f32 = 8.0;

    pub fn screen_to_world(&self, screen: WorldPoint, screen_size: WorldSize) -> WorldPoint {
        let cx = screen_size.width * 0.5;
        let cy = screen_size.height * 0.5;
        WorldPoint {
            x: (screen.x - cx) / self.zoom - self.pan_x,
            y: (screen.y - cy) / self.zoom - self.pan_y,
        }
    }

    pub fn world_to_screen(&self, world: WorldPoint, screen_size: WorldSize) -> WorldPoint {
        let cx = screen_size.width * 0.5;
        let cy = screen_size.height * 0.5;
        WorldPoint {
            x: (world.x + self.pan_x) * self.zoom + cx,
            y: (world.y + self.pan_y) * self.zoom + cy,
        }
    }

    pub fn zoom_at(&mut self, screen_anchor: WorldPoint, screen_size: WorldSize, factor: f32) {
        let world_before = self.screen_to_world(screen_anchor, screen_size);
        self.zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let world_after = self.screen_to_world(screen_anchor, screen_size);
        self.pan_x += world_before.x - world_after.x;
        self.pan_y += world_before.y - world_after.y;
    }

    pub fn pan_by_screen_delta(&mut self, dx: f32, dy: f32) {
        self.pan_x += dx / self.zoom;
        self.pan_y += dy / self.zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_world_roundtrip() {
        let viewport = Viewport {
            pan_x: 40.0,
            pan_y: -20.0,
            zoom: 1.5,
        };
        let screen_size = WorldSize {
            width: 800.0,
            height: 600.0,
        };
        let world = WorldPoint::new(100.0, 50.0);
        let screen = viewport.world_to_screen(world, screen_size);
        let back = viewport.screen_to_world(screen, screen_size);
        assert!((back.x - world.x).abs() < 0.001);
        assert!((back.y - world.y).abs() < 0.001);
    }
}
