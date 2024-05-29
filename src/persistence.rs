use lazy_static::lazy_static;
use rusqlite::{Connection, Result};
use rusqlite_migration::{Migrations, M};
use std::error::Error;

use crate::particle::Particle;

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::new(vec![
            M::up("CREATE TABLE state_vectors(
                 mash INTEGER,
                 px INTEGER,
                 py INTEGER,
                 pz INTEGER,
                 vx INTEGER,
                 vy INTEGER,
                 vz INTEGER,
                 count INTEGER
               );")
            .down("DROP TABLE state_vectors;"),
            // In the future, if the need to change the schema arises, put
            // migrations here, like so:
            // M::up("CREATE INDEX UX_friend_email ON friend(email);"),
            // M::up("CREATE INDEX UX_friend_name ON friend(name);"),
        ]);
}

#[derive(Hash, Eq, PartialEq, Debug)]
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

pub fn open_database(path: &str) -> Result<Connection> {
    Connection::open(path)
}

pub fn migrate_to_latest(connection: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    MIGRATIONS.to_latest(connection)
}

pub fn persist_state_count(
    particle: &Particle,
    connection: &Connection,
    bucket_size: f32,
) -> Result<(), Box<dyn Error>> {
    let state_vector = particle.to_state_vector(bucket_size);
    let mut stmt = connection.prepare(
        "INSERT INTO state_vectors (mass, px, py, pz, vx, vy, vz, count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)
         ON CONFLICT(mass, px, py, pz, vx, vy, vz)
         DO UPDATE SET count = count + 1;",
    )?;

    stmt.execute([
        &state_vector.mass,
        &state_vector.position_bucket.0,
        &state_vector.position_bucket.1,
        &state_vector.position_bucket.2,
        &state_vector.velocity_bucket.0,
        &state_vector.velocity_bucket.1,
        &state_vector.velocity_bucket.2,
    ])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }
}
