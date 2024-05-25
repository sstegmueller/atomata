use std::collections::HashMap;
use sled::{Db, IVec};
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Debug)]
pub struct StateVector {
    mass: i32,
    position_bucket: (i32, i32, i32),
    velocity_bucket: (i32, i32, i32),
}

impl StateVector {
    fn new(mass: f32, position: (f32, f32, f32), velocity: (f32, f32, f32), bucket_size: f32) -> Self {
        Self {
            mass: mass as i32,
            position_bucket: (
                (position.0 / bucket_size) as i32,
                (position.1 / bucket_size) as i32,
                (position.2 / bucket_size) as i32
            ),
            velocity_bucket: (
                (velocity.0 / bucket_size) as i32,
                (velocity.1 / bucket_size) as i32,
                (velocity.2 / bucket_size) as i32
            ),
        }
    }
}