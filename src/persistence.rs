use serde::{Deserialize, Serialize};
use sled::Db;
use std::error::Error;

use crate::particle::Particle;

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Debug)]
pub struct StateVector {
    mass: i32,
    position_bucket: (i32, i32, i32),
    velocity_bucket: (i32, i32, i32),
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

pub fn open_database(path: &str) -> Result<Db, Box<dyn Error>> {
    let db = sled::open(path)?;
    Ok(db)
}

pub fn persist_state_count(
    particle: &Particle,
    db: Db,
    bucket_size: f32,
) -> Result<(), Box<dyn Error>> {
    let state_vector = particle.to_state_vector(bucket_size);
    let key = bincode::serialize(&state_vector)?;
    let counter = db
        .get(&key)?
        .map(|v| bincode::deserialize::<u64>(&v).unwrap())
        .unwrap_or(0)
        + 1;
    db.insert(&key, bincode::serialize(&counter)?)?;
    Ok(())
}

pub fn map_state_count<F>(db: Db, op: F) -> Result<u64, Box<dyn Error>>
where
    F: Fn(u64, u64) -> u64,
{
    let mut sum = 0;
    for item in db.iter() {
        let (_, value) = item?;
        let count: u64 = bincode::deserialize(&value)?;
        sum = sum + op(count, sum);
    }
    Ok(sum)
}
