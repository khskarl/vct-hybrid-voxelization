use nalgebra_glm as glm;

pub struct Camera {
  pub yaw: f32,
  pub pitch: f32,
  pub position: glm::Vec3,
}

impl Camera {
  const UP: [f32; 3] = [0.0, 1.0, 0.0];

  pub const fn new(position: glm::Vec3, yaw: f32, pitch: f32) -> Camera {
    Camera {
      yaw,
      pitch,
      position,
    }
  }

  pub fn projection(&self) -> glm::Mat4 {
    glm::perspective_rh(16.0 / 9.0, f32::to_radians(65.0), 0.01, 1000.0)
  }

  pub fn view(&self) -> glm::Mat4 {
    glm::look_at_rh(
      &self.position,
      &(self.position + self.forward()),
      &self.up().normalize(),
    )
  }

  pub fn up(&self) -> glm::Vec3 {
    glm::make_vec3(&Self::UP)
  }

  pub fn right(&self) -> glm::Vec3 {
    glm::cross::<f32, glm::U3>(&self.up(), &self.forward()).normalize()
  }

  pub fn forward(&self) -> glm::Vec3 {
    // let pitch_rad = f32::to_radians(self.pitch);
    // let yaw_rad = f32::to_radians(self.yaw);
    let pitch_rad = (self.pitch);
    let yaw_rad = (self.yaw);

    glm::make_vec3(&[
      yaw_rad.sin() * pitch_rad.cos(),
      pitch_rad.sin(),
      yaw_rad.cos() * pitch_rad.cos(),
    ])
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

  pub fn near(&self) -> f32 {
    (self.projection()[(2, 3)] / (self.projection()[(2, 2)] - 1.0))
  }

  pub fn far(&self) -> f32 {
    ((self.projection()[(2, 3)]) / (self.projection()[(2, 2)] + 1.0))
  }
}
