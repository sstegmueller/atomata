use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result, Transaction};
use rusqlite_migration::{Migrations, M};
use std::error::Error;

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> =
        Migrations::new(vec![
            M::up("CREATE TABLE state_vectors(
                 mass INTEGER NOT NULL,
                 px INTEGER NOT NULL,
                 py INTEGER NOT NULL,
                 pz INTEGER NOT NULL,
                 vx INTEGER NOT NULL,
                 vy INTEGER NOT NULL,
                 vz INTEGER NOT NULL,
                 count INTEGER,
                 PRIMARY KEY (mass, px, py, pz, vx, vy, vz)
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

pub fn create_transaction(connection: &mut Connection) -> Result<Transaction> {
    connection.transaction()
}

pub fn commit_transaction(transaction: Transaction) -> Result<()> {
    transaction.commit()
}

pub fn persist_state_count(
    state_vector: &StateVector,
    tx: &Transaction,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = tx.prepare(
        "INSERT INTO state_vectors (mass, px, py, pz, vx, vy, vz, count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)
         ON CONFLICT(mass, px, py, pz, vx, vy, vz)
         DO UPDATE SET count = count + 1;",
    )?;
    stmt.execute(params![
        state_vector.mass,
        state_vector.position_bucket.0,
        state_vector.position_bucket.1,
        state_vector.position_bucket.2,
        state_vector.velocity_bucket.0,
        state_vector.velocity_bucket.1,
        state_vector.velocity_bucket.2
    ])?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    fn open_memory_database() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_migrations() {
        assert!(MIGRATIONS.validate().is_ok());
    }

    #[test]
    fn test_persist_state_count() {
        let mut connection = open_memory_database();
        migrate_to_latest(&mut connection).unwrap();
        let tx = create_transaction(&mut connection).unwrap();
        let state_vector = StateVector::new(1.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 10.0);
        persist_state_count(&state_vector, &tx).unwrap();
        commit_transaction(tx).unwrap();

        let mut stmt = connection
            .prepare(
                "SELECT count FROM state_vectors
             WHERE mass = 1 AND px = 0 AND py = 0 AND pz = 0 AND vx = 0 AND vy = 0 AND vz = 0;",
            )
            .unwrap();

        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
