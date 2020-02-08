use cgmath::{prelude::*, Matrix4, Point3};

use crate::consts::*;

pub struct WorldData {
    circle: f32,
    camera_distance: f32,
    world_mat: Matrix4<f32>,
    center: Point3<f32>,
    eye: Point3<f32>,
    pub shading_enabled: bool,
    pub is_paused: bool,
    pub ao_enabled: bool,
    zoom_level: i32,
    tilt: f32
}

impl WorldData {
    pub fn rotate_delta(&mut self, delta: f32) {
        if !self.is_paused {
            self.circle += delta;
        }
        let x = self.camera_distance * (1.0 - self.tilt.powi(2)).sqrt() * self.circle.sin();
        let z = self.camera_distance * (1.0 - self.tilt.powi(2)).sqrt() * self.circle.cos();
        self.eye = Point3::new(x + self.center.x, self.tilt * self.camera_distance + self.center.y, z + self.center.z);
        self.world_mat = Matrix4::look_at(self.eye, self.center, UP_VECTOR);
    }

    pub fn rotate_manual(&mut self, (delta_x, delta_y): (f64, f64)) {
        self.circle += -(delta_x as f32) / 50.0;
        self.tilt += (delta_y as f32) / 60.0;
        self.tilt = self.tilt.max(-0.999).min(0.999);
        let x = self.camera_distance * (1.0 - self.tilt.powi(2)).sqrt() * self.circle.sin();
        let z = self.camera_distance * (1.0 - self.tilt.powi(2)).sqrt() * self.circle.cos();
        self.eye = Point3::new(x + self.center.x, self.tilt * self.camera_distance + self.center.y, z + self.center.z);
        self.world_mat = Matrix4::look_at(self.eye, self.center, UP_VECTOR);
    }

    pub fn adjust_zoom(&mut self, delta: i32) {
        self.zoom_level += delta;
        self.camera_distance = CAMERA_DIST * 2.0_f32.powf(-self.zoom_level as f32 / 2.0);
    }

    pub fn pan_manual(&mut self, (delta_x, delta_y): (f64, f64)) {
        let camera_dir = (self.center - self.eye).normalize();
        let side = UP_VECTOR.cross(camera_dir);
        let up = camera_dir.cross(side);
        self.eye += side * (delta_x as f32 / 50.0);
        self.eye += up * (delta_y as f32 / 50.0);
        self.center += side * (delta_x as f32 / 50.0);
        self.center += up * (delta_y as f32 / 50.0);
        self.world_mat = Matrix4::look_at(self.eye, self.center, UP_VECTOR);
    }

    pub fn toggle_ao(&mut self) {
        self.ao_enabled = !self.ao_enabled;
    }

    pub fn toggle_paused(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn toggle_shading(&mut self) {
        self.shading_enabled = !self.shading_enabled;
    }

    pub fn world_mat(&self) -> &Matrix4<f32> {
        &self.world_mat
    }
}

impl Default for WorldData {
    fn default() -> Self {
        Self {
            circle: 0.0,
            camera_distance: CAMERA_DIST,
            eye: Point3::new(0.0, 0.0, CAMERA_DIST),
            world_mat: Matrix4::look_at(Point3::new(0.0, 0.0, CAMERA_DIST), CENTER, UP_VECTOR),
            shading_enabled: true,
            is_paused: false,
            ao_enabled: true,
            zoom_level: 0,
            tilt: 0.0,
            center: CENTER
        }
    }
}
