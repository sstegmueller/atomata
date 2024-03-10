use three_d::{
    vec3, InnerSpace, Vector3
};

use crate::parameters::Parameters;
use crate::sphere::PositionableRender;

pub struct Particle {
    pub position: Vector3<f32>,
    pub positionable: Box<dyn PositionableRender>,
    pub mass: f32,
    velocity: Vector3<f32>,
}

impl Particle {
    pub fn new(mut positionable: Box<dyn PositionableRender>, border: f32, mass: f32) -> Self {
        let factor = border / 2.0;
        // generate random position in the range of -1 to +1 times factor
        let x = (rand::random::<f32>() - 0.5) * factor;
        let y = (rand::random::<f32>() - 0.5) * factor;
        let z = (rand::random::<f32>() - 0.5) * factor;
        let position = vec3(x, y, z);

        positionable.set_position(position);

        Self {
            position,
            velocity: vec3(0.0, 0.0, 0.0),
            mass,
            positionable,
        }
    }

    pub fn update_velocity(
        &mut self,
        other_position: Vector3<f32>,
        other_mass: f32,
        gravity_constant: f32,
    ) {
        let distance = self.position - other_position;
        let distance_squared = distance.dot(distance);
        let mut directed_acceleration = vec3(0.0, 0.0, 0.0);
        if distance_squared > 0.0001 {
            let acceleration = gravity_constant * other_mass / distance_squared;
            directed_acceleration = distance.normalize() * acceleration;
        }

        self.velocity += directed_acceleration;
    }

    pub fn update_position(&mut self, time_step: f32, parameters: &Parameters) {
        let mut updated_position = self.compute_updated_position(time_step);

        let distance_from_center = updated_position.magnitude();

        if distance_from_center.abs() > parameters.border {
            self.velocity = -self.velocity;
            updated_position = self.compute_updated_position(time_step);
        }

        self.position = updated_position;
        self.positionable.set_position(self.position);
    }

    pub fn apply_friction(&mut self, friction: f32) {
        self.velocity *= 1.0 - friction;
    }

    fn compute_updated_position(&self, time_step: f32) -> Vector3<f32> {
        self.position + self.velocity * time_step
    }
}
