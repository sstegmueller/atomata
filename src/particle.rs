use three_d::{vec3, InnerSpace, Vector3};

use crate::parameters::Parameters;
use crate::persistence::StateVector;
use crate::sphere::PositionableRender;

pub struct Particle {
    pub position: Vector3<f32>,
    pub positionable: Box<dyn PositionableRender>,
    pub mass: f32,
    velocity: Vector3<f32>,
    max_velocity: f32,
}

impl Particle {
    pub fn new(
        mut positionable: Box<dyn PositionableRender>,
        border: f32,
        mass: f32,
        max_velocity: f32,
    ) -> Self {
        // generate random position in the range of -1 to +1 times factor
        let x = (rand::random::<f32>() - 0.5) * border;
        let y = (rand::random::<f32>() - 0.5) * border;
        let z = (rand::random::<f32>() - 0.5) * border;
        let position = vec3(x, y, z);
        positionable.set_position(position);

        // initialize random velocity from 0 top max_velocity
        let vx = (rand::random::<f32>() - 0.5) * max_velocity;
        let vy = (rand::random::<f32>() - 0.5) * max_velocity;
        let vz = (rand::random::<f32>() - 0.5) * max_velocity;

        Self {
            position,
            velocity: vec3(vx, vy, vz),
            mass,
            positionable,
            max_velocity,
        }
    }

    pub fn update_velocity(
        &mut self,
        other_position: Vector3<f32>,
        other_mass: f32,
        gravity_constant: f32,
    ) {
        let direction = other_position - self.position;
        let distance = direction.magnitude();
        if distance > 0.0001 {
            let force_magnitude = gravity_constant * self.mass * other_mass / (distance * distance);
            let force = direction.normalize() * force_magnitude;

            self.velocity += force / self.mass;

            if self.velocity.x.abs() > self.max_velocity {
                self.velocity.x = self.velocity.x.signum() * self.max_velocity;
            }

            if self.velocity.y.abs() > self.max_velocity {
                self.velocity.y = self.velocity.y.signum() * self.max_velocity;
            }

            if self.velocity.z.abs() > self.max_velocity {
                self.velocity.z = self.velocity.z.signum() * self.max_velocity;
            }
        }
    }

    pub fn update_position(&mut self, parameters: &Parameters) {
        let mut updated_position = self.compute_updated_position(parameters.timestep);

        let distance_from_center = updated_position.magnitude();

        if distance_from_center.abs() > parameters.border {
            self.velocity = -self.velocity;
            updated_position = self.compute_updated_position(parameters.timestep);
        }

        self.position = updated_position;
        self.positionable.set_position(self.position);
    }

    pub fn apply_friction(&mut self, friction: f32) {
        self.velocity *= 1.0 - friction;
    }

    pub fn to_state_vector(&self, bucket_size: f32) -> StateVector {
        StateVector::new(
            self.mass,
            (self.position.x, self.position.y, self.position.z),
            (self.velocity.x, self.velocity.y, self.velocity.z),
            bucket_size,
        )
    }

    fn compute_updated_position(&self, time_step: f32) -> Vector3<f32> {
        self.position + self.velocity * time_step
    }
}

#[cfg(test)]
mod tests {
    use three_d::{Gm, Mesh, PhysicalMaterial};

    use super::*;

    struct MockPositionableRender;

    impl PositionableRender for MockPositionableRender {
        fn set_position(&mut self, _position: Vector3<f32>) {
            // Do nothing
        }

        fn get_geometry(&self) -> &Gm<Mesh, PhysicalMaterial> {
            todo!()
        }
    }

    #[test]
    fn test_new_particle() {
        let positionable = Box::new(MockPositionableRender);
        let border = 10.0;
        let mass = 1.0;
        let max_velocity = 1000.0;

        let particle = Particle::new(positionable, border, mass, max_velocity);

        assert_eq!(particle.mass, mass);

        // assert position is within the range of -border/2 to +border/2
        assert!(particle.position.x >= -border && particle.position.x <= border);
        assert!(particle.position.y >= -border && particle.position.y <= border);
        assert!(particle.position.z >= -border && particle.position.z <= border);

        // assert velocity is within the range of -max_velocity to +max_velocity
        assert!(particle.velocity.x >= -max_velocity && particle.velocity.x <= max_velocity);
        assert!(particle.velocity.y >= -max_velocity && particle.velocity.y <= max_velocity);
        assert!(particle.velocity.z >= -max_velocity && particle.velocity.z <= max_velocity);
    }

    #[test]
    fn test_update_velocity() {
        let mut particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            max_velocity: 1000.0,
        };

        let other_position = Vector3::new(2.0, 2.0, 2.0);
        let other_mass = 2.0;
        let gravity_constant = 9.8;

        particle.update_velocity(other_position, other_mass, gravity_constant);

        assert_eq!(
            particle.velocity,
            Vector3::new(0.94300544, 0.94300544, 0.94300544)
        );
    }

    #[test]
    fn test_update_position() {
        let mut particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(1.0, 1.0, 1.0),
            max_velocity: 1000.0,
        };

        let parameters = Parameters {
            border: 10.0,
            amount: 30,
            timestep: 0.1,
            gravity_constant: 9.8,
            friction: 0.5,
            mass_red: 1.0,
            mass_green: 1.0,
            mass_blue: 1.0,
            max_velocity: 1000.0,
            bucket_size: 1.0,
            database_path: "particles_states.db".to_string(),
        };

        particle.update_position(&parameters);

        assert_eq!(particle.position, Vector3::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_apply_friction() {
        let mut particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(1.0, 1.0, 1.0),
            max_velocity: 1000.0,
        };

        let friction = 0.5;

        particle.apply_friction(friction);

        assert_eq!(particle.velocity, Vector3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn test_compute_updated_position() {
        let particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(1.0, 1.0, 1.0),
            max_velocity: 1000.0,
        };

        let time_step = 0.1;

        let updated_position = particle.compute_updated_position(time_step);

        assert_eq!(updated_position, Vector3::new(0.1, 0.1, 0.1));
    }
}
