use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result, Statement, Transaction};
use rusqlite_migration::{Migrations, M};
use std::error::Error;

use crate::{parameters::Parameters, particle::StateVector};

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![
        M::up(
            "CREATE TABLE run_parameters (
                run_id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount INTEGER NOT NULL,
                border REAL NOT NULL,
                timestep REAL NOT NULL,
                gravity_constant REAL NOT NULL,
                friction REAL NOT NULL,
                max_velocity REAL NOT NULL,
                bucket_size REAL NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        )
        .down("DROP TABLE run_parameters;"),
        M::up(
            "CREATE TABLE particle_parameters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mass REAL NOT NULL,
                ix INTEGER NOT NULL,
                run_id INTEGER NOT NULL,
                FOREIGN KEY (run_id) REFERENCES run_parameters(run_id) ON DELETE CASCADE
            );"
        )
        .down("DROP TABLE particle_parameters;"),
        M::up(
            "CREATE TABLE interactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                interaction_type TEXT NOT NULL,
                parameter_id_0 INTEGER NOT NULL,
                parameter_id_1 INTEGER NOT NULL,
                FOREIGN KEY (parameter_id_0) REFERENCES particle_parameters(id) ON DELETE CASCADE,
                FOREIGN KEY (parameter_id_1) REFERENCES particle_parameters(id) ON DELETE CASCADE
            );"
        )
        .down("DROP TABLE interactions;"),
        M::up(
            "CREATE TABLE state_vectors(
                 px INTEGER NOT NULL,
                 py INTEGER NOT NULL,
                 pz INTEGER NOT NULL,
                 vx INTEGER NOT NULL,
                 vy INTEGER NOT NULL,
                 vz INTEGER NOT NULL,
                 count INTEGER,
                 particle_parameters_id INTEGER NOT NULL,
                 PRIMARY KEY (particle_parameters_id, px, py, pz, vx, vy, vz),
                 FOREIGN KEY (particle_parameters_id) REFERENCES particle_parameters(id) ON DELETE CASCADE
               );
            "
        )
        .down("DROP TABLE state_vectors;"),
    ]);
}

trait ConnectionProvider {
    fn transaction(&mut self) -> Result<Transaction>;
}

pub struct ConnectionProviderImpl {
    connection: Connection,
}

impl ConnectionProvider for ConnectionProviderImpl {
    fn transaction(&mut self) -> Result<Transaction> {
        self.connection.transaction()
    }
}

pub trait TransactionProvider {
    fn prepare(&self, sql: &str) -> Result<Statement>;
    fn commit(self) -> Result<()>;
    fn get_last_insert_rowid(&self) -> i64;
}

pub struct TransactionProviderImpl<'a> {
    transaction: Transaction<'a>,
}

impl<'a> TransactionProvider for TransactionProviderImpl<'a> {
    fn prepare(&self, sql: &str) -> Result<Statement> {
        self.transaction.prepare(sql)
    }

    fn commit(self) -> Result<()> {
        self.transaction.commit()
    }

    fn get_last_insert_rowid(&self) -> i64 {
        self.transaction.last_insert_rowid()
    }
}

pub fn open_database(path: &str) -> Result<ConnectionProviderImpl> {
    Ok(ConnectionProviderImpl {
        connection: Connection::open(path)?,
    })
}

pub fn migrate_to_latest(
    connection_provider: &mut ConnectionProviderImpl,
) -> Result<(), rusqlite_migration::Error> {
    MIGRATIONS.to_latest(&mut connection_provider.connection)
}

pub fn create_transaction_provider(
    connection: &mut ConnectionProviderImpl,
) -> Result<TransactionProviderImpl<'_>, Box<dyn Error>> {
    let transaction = connection.transaction()?;
    Ok(TransactionProviderImpl { transaction })
}

pub fn commit_transaction(transaction: TransactionProviderImpl) -> Result<()> {
    transaction.commit()
}

pub fn increment_state_count<T: TransactionProvider>(
    state_vector: &StateVector,
    tx: &T,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = tx.prepare(
        "INSERT INTO state_vectors (px, py, pz, vx, vy, vz, particle_parameters_id, count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)
         ON CONFLICT(px, py, pz, vx, vy, vz, particle_parameters_id)
         DO UPDATE SET count = count + 1;",
    )?;
    stmt.execute(params![
        state_vector.position_bucket.0,
        state_vector.position_bucket.1,
        state_vector.position_bucket.2,
        state_vector.velocity_bucket.0,
        state_vector.velocity_bucket.1,
        state_vector.velocity_bucket.2,
        state_vector.particle_parameters_id,
    ])?;
    Ok(())
}

pub fn persist_parameters<T: TransactionProvider>(
    parameters: &mut Parameters,
    tx: &T,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = tx.prepare(
        "INSERT INTO run_parameters (amount, border, timestep, gravity_constant, friction, max_velocity, bucket_size)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);",
    )?;
    stmt.execute(params![
        parameters.amount,
        parameters.border,
        parameters.timestep,
        parameters.gravity_constant,
        parameters.friction,
        parameters.max_velocity,
        parameters.bucket_size
    ])?;
    let parameters_id = tx.get_last_insert_rowid();

    for particle in parameters.particle_parameters.iter_mut() {
        let mut stmt = tx.prepare(
            "INSERT INTO particle_parameters (mass, ix, run_id)
             VALUES (?1, ?2, ?3);",
        )?;
        stmt.execute(params![particle.mass, particle.index, parameters_id])?;

        particle.id = Some(tx.get_last_insert_rowid() as usize);
    }

    for i in 0..parameters.particle_parameters.len() {
        for j in i..parameters.particle_parameters.len() {
            let interaction = parameters.interaction_by_indices(i, j)?;
            let mut stmt = tx.prepare(
                "INSERT INTO interactions (interaction_type, parameter_id_0, parameter_id_1)
                 VALUES (?1, ?2, ?3);",
            )?;
            stmt.execute(params![interaction.to_string(), i as i64 + 1, j as i64 + 1])?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parameters::{InteractionType, ParticleParameters};

    use super::*;
    use pretty_assertions_sorted::assert_eq;

    fn open_memory_database() -> ConnectionProviderImpl {
        ConnectionProviderImpl {
            connection: Connection::open_in_memory().unwrap(),
        }
    }

    #[test]
    fn test_migrations() {
        assert!(MIGRATIONS.validate().is_ok());
    }

    #[test]
    fn test_persist_parameters() {
        let mut connection_provider = open_memory_database();
        migrate_to_latest(&mut connection_provider).unwrap();
        let tx_provider = create_transaction_provider(&mut connection_provider).unwrap();
        let mut parameters = Parameters {
            amount: 10,
            border: 200.0,
            friction: 0.0,
            timestep: 0.0002,
            gravity_constant: 1.0,
            particle_parameters: vec![
                ParticleParameters {
                    id: None,
                    mass: 3.0,
                    index: 0,
                },
                ParticleParameters {
                    id: None,
                    mass: 250.0,
                    index: 1,
                },
                ParticleParameters {
                    id: None,
                    mass: 10000.0,
                    index: 2,
                },
                ParticleParameters {
                    id: None,
                    mass: 10000.0,
                    index: 3,
                },
            ],
            interactions: vec![
                InteractionType::Attraction, // 0 <-> 0
                InteractionType::Neutral,    // 1 <-> 0
                InteractionType::Repulsion,  // 2 <-> 0
                InteractionType::Repulsion,  // 3 <-> 0
                InteractionType::Neutral,    // 1 <-> 1
                InteractionType::Attraction, // 1 <-> 2
                InteractionType::Attraction, // 1 <-> 3
                InteractionType::Repulsion,  // 2 <-> 2
                InteractionType::Repulsion,  // 2 <-> 3
                InteractionType::Repulsion,  // 3 <-> 3
            ],
            max_velocity: 20000.0,
            bucket_size: 10.0,
        };
        let _ = persist_parameters(&mut parameters, &tx_provider).unwrap();
        commit_transaction(tx_provider).unwrap();

        let mut stmt = connection_provider
            .connection
            .prepare("SELECT count(*) FROM run_parameters;")
            .unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);

        let mut stmt = connection_provider
            .connection
            .prepare("SELECT count(*) FROM particle_parameters;")
            .unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, parameters.particle_parameters.len() as i32);

        let mut stmt = connection_provider
            .connection
            .prepare("SELECT count(*) FROM interactions;")
            .unwrap();
        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, parameters.interactions.len() as i32);
    }

    #[test]
    fn test_increment_state_count() {
        let mut connection_provider = open_memory_database();
        migrate_to_latest(&mut connection_provider).unwrap();

        let tx_provider = create_transaction_provider(&mut connection_provider).unwrap();
        let mut parameters = Parameters {
            amount: 10,
            border: 200.0,
            friction: 0.0,
            timestep: 0.0002,
            gravity_constant: 1.0,
            particle_parameters: vec![
                ParticleParameters {
                    id: None,
                    mass: 3.0,
                    index: 0,
                },
                ParticleParameters {
                    id: None,
                    mass: 250.0,
                    index: 1,
                },
            ],
            interactions: vec![
                InteractionType::Attraction, // 0 <-> 0
                InteractionType::Neutral,    // 1 <-> 0
                InteractionType::Repulsion,  // 1 <-> 1
            ],
            max_velocity: 20000.0,
            bucket_size: 10.0,
        };

        persist_parameters(&mut parameters, &tx_provider).unwrap();
        let particle_parameter_id = parameters.particle_parameters[0].id.unwrap();

        let state_vector = StateVector::new(
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0),
            10.0,
            particle_parameter_id,
        );
        increment_state_count(&state_vector, &tx_provider).unwrap();
        commit_transaction(tx_provider).unwrap();

        let mut stmt = connection_provider
            .connection
            .prepare(
                "SELECT count FROM state_vectors
             WHERE px = 0 AND py = 0 AND pz = 0 AND vx = 0 AND vy = 0 AND vz = 0;",
            )
            .unwrap();

        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
