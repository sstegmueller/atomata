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
        let direction = other_position - self.position;
        let distance = direction.magnitude();
        if distance > 0.0001 {
            let force_magnitude = gravity_constant * self.mass * other_mass / (distance * distance);
            let force = direction.normalize() * force_magnitude;

            self.velocity += force / self.mass;
        }
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
        
        let particle = Particle::new(positionable, border, mass);
        
        assert_eq!(particle.mass, mass);
        assert_eq!(particle.velocity, Vector3::new(0.0, 0.0, 0.0));
    }
    
    #[test]
    fn test_update_velocity() {
        let mut particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(0.0, 0.0, 0.0),
        };
        
        let other_position = Vector3::new(2.0, 2.0, 2.0);
        let other_mass = 2.0;
        let gravity_constant = 9.8;
        
        particle.update_velocity(other_position, other_mass, gravity_constant);
        
        assert_eq!(particle.velocity, Vector3::new(0.94300544, 0.94300544, 0.94300544));
    }
    
    #[test]
    fn test_update_position() {
        let mut particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(1.0, 1.0, 1.0),
        };
        
        let time_step = 0.1;
        let parameters = Parameters { border: 10.0, amount: 30, mass_red: 1.0, mass_green: 1.0, mass_blue: 1.0 };
        
        particle.update_position(time_step, &parameters);
        
        assert_eq!(particle.position, Vector3::new(0.1, 0.1, 0.1));
    }
    
    #[test]
    fn test_apply_friction() {
        let mut particle = Particle {
            position: Vector3::new(0.0, 0.0, 0.0),
            positionable: Box::new(MockPositionableRender),
            mass: 1.0,
            velocity: Vector3::new(1.0, 1.0, 1.0),
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
        };
        
        let time_step = 0.1;
        
        let updated_position = particle.compute_updated_position(time_step);
        
        assert_eq!(updated_position, Vector3::new(0.1, 0.1, 0.1));
    }
}

