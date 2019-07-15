use nalgebra;

pub struct Camera {
  yaw: f32,
  pitch: f32,
  proj: nalgebra::Perspective3<f32>,
}
