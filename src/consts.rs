use cgmath::{Point3, Vector3, vec3};

/// consts for `WorldData`
pub const CENTER: Point3<f32> = Point3::new(0.0, 0.0, 0.0);
pub const CAMERA_DIST: f32 = 3.0;
pub const UP_VECTOR: Vector3<f32> = vec3(0.0, 1.0, 0.0);

/// consts for computations
pub const ANGLE_SPREAD: f32 = 178.0;
pub const SAMPLES: u32 = 256;

/// consts for window management
pub const SIZE_X: u32 = 1280;
pub const SIZE_Y: u32 = 720;
pub const ASPECT_RATIO: f32 = 1280.0_f32 / 720.0;
