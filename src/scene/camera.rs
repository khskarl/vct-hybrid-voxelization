use nalgebra::*;

pub struct Camera {
  yaw: f32,
  pitch: f32,
  position: Vector3<f32>,
  up: Vector3<f32>,
}

impl Camera {
  pub fn new(position: Vector3<f32>, yaw: f32, pitch: f32) -> Camera {
    Camera {
      yaw,
      pitch,
      position,
      up: Vector3::y(),
    }
  }

  pub fn projection(&self) -> Perspective3<f32> {
    Perspective3::new(16.0 / 9.0, 3.1415 / 4.0, 1.0, 1000.0)
  }

  pub fn view(&self) -> Isometry3<f32> {
    let rotation = UnitQuaternion::from_euler_angles(0.0, self.pitch, self.yaw);
    let translation = Translation3::from(self.position);

    Isometry3::from_parts(translation, rotation)
  }

  pub fn up(&self) -> Vector3<f32> {
    Vector3::y()
  }

  pub fn right(&self) -> Vector3<f32> {
    Vector3::x()
  }

  pub fn forward(&self) -> Vector3<f32> {
    Vector3::z()
  }

  pub fn move_right(&mut self, amount: f32) {
    self.position += self.right() * amount
  }

  pub fn move_up(&mut self, amount: f32) {
    self.position += self.up() * amount
  }

  pub fn move_forward(&mut self, amount: f32) {
    self.position += self.forward() * amount
  }

  pub fn rotate_up(&mut self, amount: f32) {
    self.pitch += amount
  }

  pub fn rotate_right(&mut self, amount: f32) {
    self.yaw += amount
  }
}
