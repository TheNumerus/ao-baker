use cgmath::{Point3, Vector3, vec3};

use crate::geo::VertexUV;

/// consts for `WorldData`
pub const CENTER: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
pub const CAMERA_DIST: f32 = 3.0;
pub const UP_VECTOR: Vector3<f32> = vec3(0.0, 1.0, 0.0);

/// consts for computations
pub const ANGLE_SPREAD: f32 = 178.0;
pub const SAMPLES: u32 = 512;
pub const MAP: [usize; 8] = [2, 1, 2, 1, 2, 2, 0, 0];

/// consts for window management
pub const SIZE_X: u32 = 1280;
pub const SIZE_Y: u32 = 720;
pub const ASPECT_RATIO: f32 = 1280.0_f32 / 720.0;
pub const APP_NAME: &str = "AO Baker";

/// consts for rendering
pub const FONT_BYTES: &[u8] = include_bytes!("../fonts/Roboto.ttf");
pub const QUAD: [VertexUV; 6] = [
    VertexUV{pos: [0.0, 0.0, 0.0], uv: [0.0, 0.0]},
    VertexUV{pos: [1.0, 0.0, 0.0], uv: [1.0, 0.0]},
    VertexUV{pos: [1.0, 1.0, 0.0], uv: [1.0, 1.0]},
    VertexUV{pos: [0.0, 1.0, 0.0], uv: [0.0, 1.0]},
    VertexUV{pos: [0.0, 0.0, 0.0], uv: [0.0, 0.0]},
    VertexUV{pos: [1.0, 1.0, 0.0], uv: [1.0, 1.0]}
];
pub const TOOLTIPS: [&str; 3] = [
    "P - toggle animation",
    "D - toggle shading",
    "F - toggle AO"
];