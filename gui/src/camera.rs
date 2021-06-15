use itertools::Itertools;
use nalgebra_glm as glm;
use glm::{Vec2, Vec3, Vec4, Mat4};
use winit::event::MouseButton;

use triangulate::mesh::Vertex;

#[derive(Copy, Clone, Debug)]
enum MouseState {
    Unknown,
    Free(Vec2),
    Rotate(Vec2),
    Pan(Vec2),
}

pub struct Camera {
    /// Aspect ratio of the window
    aspect: f32,

    /// Pitch as an Euler angle
    pitch: f32,

    /// Yaw as an Euler angle
    yaw: f32,

    /// Model scale
    scale: f32,

    /// Center of view volume
    center: Vec3,

    mouse: MouseState,
}


impl Camera {
    pub fn new(aspect: f32) -> Self {
        Camera {
            aspect,
            pitch: 0.0,
            yaw: 0.0,
            scale: 1.0,
            center: Vec3::zeros(),
            mouse: MouseState::Unknown,
        }
    }

    pub fn mouse_pressed(&mut self, button: MouseButton) {
        // If we were previously free, then switch to panning or rotating
        if let MouseState::Free(pos) = &self.mouse {
            match button {
                MouseButton::Left => Some(MouseState::Rotate(*pos)),
                MouseButton::Right => Some(MouseState::Pan(*pos)),
                _ => None,
            }.map(|m| self.mouse = m);
        }
    }
    pub fn mouse_released(&mut self, button: MouseButton) {
        match &self.mouse {
            MouseState::Rotate(pos) if button == MouseButton::Left =>
                Some(MouseState::Free(*pos)),
            MouseState::Pan(pos) if button == MouseButton::Right =>
                Some(MouseState::Free(*pos)),
            _ => None,
        }.map(|m| self.mouse = m);
    }

    pub fn mouse_move(&mut self, new_pos: Vec2) {
        // Pan or rotate depending on current mouse state
        match &self.mouse {
            MouseState::Pan(pos) => {
                let delta = new_pos - *pos;
                self.translate_camera(delta.x / 100.0, delta.y / 100.0);
            },
            MouseState::Rotate(pos) => {
                let delta = new_pos - *pos;
                self.spin(delta.x / -10.0, delta.y / 10.0);
            },
            _ => (),
        }

        // Store new mouse position
        match &mut self.mouse {
            MouseState::Free(pos)
            | MouseState::Pan(pos)
            | MouseState::Rotate(pos) => *pos = new_pos,
            MouseState::Unknown => self.mouse = MouseState::Free(new_pos),
        }
    }

    pub fn mouse_scroll(&mut self, delta: f32) {
        if let MouseState::Free(_) = &self.mouse {
            self.scale(1.0 + delta / 10.0);
        }
    }

    pub fn fit_verts(&mut self, verts: &[Vertex]) {
        println!("Got verts {:?}", verts);
        let xb = verts.iter().map(|v| v.pos.x).minmax().into_option().unwrap();
        let yb = verts.iter().map(|v| v.pos.y).minmax().into_option().unwrap();
        let zb = verts.iter().map(|v| v.pos.z).minmax().into_option().unwrap();
        let dx = xb.1 - xb.0;
        let dy = yb.1 - yb.0;
        let dz = zb.1 - zb.0;
        self.scale = (1.0 / dx.max(dy).max(dz)) as f32;
        self.center = Vec3::new((xb.0 + xb.1) as f32 / 2.0,
                                (yb.0 + yb.1) as f32 / 2.0,
                                (zb.0 + zb.1) as f32 / 2.0);
    }

    pub fn set_aspect(&mut self, a: f32) {
        self.aspect = a;
    }

    pub fn model_matrix(&self) -> Mat4 {
        let i = Mat4::identity();
        // The transforms below are applied bottom-to-top when thinking about
        // the model, i.e. it's translated, then scaled, then rotated, etc.

        // Rotation!
        glm::rotate_x(&i, self.yaw) *
        glm::rotate_y(&i, self.pitch) *

        // Scale to compensate for model size
        glm::scale(&i, &Vec3::new(self.scale, self.scale, self.scale)) *

        // Recenter model
        glm::translate(&i, &-self.center)
    }


    /// Returns a matrix which compensates for window aspect ratio and clipping
    pub fn view_matrix(&self) -> Mat4 {
        let i = Mat4::identity();
        // The Z clipping range is 0-1, so push forward
        glm::translate(&i, &Vec3::new(0.0, 0.0, 0.5)) *

        // Scale to compensate for aspect ratio and reduce Z scale to improve
        // clipping
        glm::scale(&i, &Vec3::new(1.0, self.aspect, 0.1))
    }

    pub fn spin(&mut self, dx: f32, dy: f32) {
        self.pitch += dx;
        self.yaw += dy;
    }

    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32){
        self.center.x += dx;
        self.center.y += dy;
        self.center.z += dz;
    }

    pub fn translate_camera(&mut self, dx: f32, dy: f32){
        let i = Mat4::identity();
        let vec = glm::rotate_y(&i, -self.pitch) *
                  glm::rotate_x(&i, -self.yaw) *
                  Vec4::new(dx, dy, 0.0, 1.0);
        self.translate(vec.x, vec.y, vec.z);
    }

    pub fn scale(&mut self, value: f32){
        self.scale *= value;
    }
}
