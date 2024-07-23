use three_d::{vec3, InnerSpace, Vector3};

use crate::parameters::{InteractionType, Parameters};
use crate::sphere::PositionableRender;

pub struct Particle {
    pub id: usize,
    pub position: Vector3<f32>,
    pub positionable: Box<dyn PositionableRender>,
    pub mass: f32,
    velocity: Vector3<f32>,
    max_velocity: f32,
}

impl Particle {
    pub fn new(
        id: usize,
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
            id,
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
        interaction_type: InteractionType,
        gravity_constant: f32,
    ) {
        if interaction_type == InteractionType::Neutral {
            return;
        }

        let direction = other_position - self.position;
        let distance = direction.magnitude();
        if distance > 0.0001 {
            let force_magnitude = gravity_constant * self.mass * other_mass / (distance * distance);
            let force = direction.normalize() * force_magnitude;

            if interaction_type == InteractionType::Attraction {
                self.velocity += force / self.mass;
            } else {
                self.velocity -= force / self.mass;
            }

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

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct StateVector {
    pub mass: i32,
    pub position_bucket: (i32, i32, i32),
    pub velocity_bucket: (i32, i32, i32),
}

impl StateVector {
    pub fn new(
        mass: f32,
        position: (f32, f32, f32),
        velocity: (f32, f32, f32),
        bucket_size: f32,
    ) -> Self {
        Self {
            mass: mass as i32,
            position_bucket: (
                (position.0 / bucket_size) as i32,
                (position.1 / bucket_size) as i32,
                (position.2 / bucket_size) as i32,
            ),
            velocity_bucket: (
                (velocity.0 / bucket_size) as i32,
                (velocity.1 / bucket_size) as i32,
                (velocity.2 / bucket_size) as i32,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use three_d::{Gm, Mesh, PhysicalMaterial};

    use crate::parameters::{Mode, ParticleParameters};

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

        let particle = Particle::new(0, positionable, border, mass, max_velocity);

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
            id: 0,
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            max_velocity: 1000.0,
        };

        let other_position = Vector3::new(2.0, 2.0, 2.0);
        let other_mass = 2.0;
        let gravity_constant = 9.8;

        particle.update_velocity(
            other_position,
            other_mass,
            InteractionType::Attraction,
            gravity_constant,
        );

        assert_eq!(
            particle.velocity,
            Vector3::new(0.94300544, 0.94300544, 0.94300544)
        );
    }

    #[test]
    fn test_update_position() {
        let mut particle = Particle {
            id: 0,
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
            max_velocity: 1000.0,
            bucket_size: 1.0,
            particle_parameters: vec![ParticleParameters {
                mass: 1.0,
                index: 0,
            }],
            interactions: vec![InteractionType::Attraction],
            database_path: "particles_states.db".to_string(),
            mode: Mode::Default,
        };

        particle.update_position(&parameters);

        assert_eq!(particle.position, Vector3::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_compute_updated_position() {
        let particle = Particle {
            id: 0,
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
