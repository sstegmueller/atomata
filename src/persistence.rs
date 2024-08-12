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
                index INTEGER NOT NULL,
                FOREIGN KEY (run_id) REFERENCES run_parameters(run_id) ON DELETE CASCADE
            );"
        )
        .down("DROP TABLE particle_parameters;"),
        M::up(
            "CREATE TABLE interactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                iteraction_type TEXT NOT NULL,
                FOREIGN KEY (parameter_id_0) REFERENCES particle_parameters(id) ON DELETE CASCADE
                FOREIGN KEY (parameter_id_1) REFERENCES particle_parameters(id) ON DELETE CASCADE
            );"
        )
        .down("DROP TABLE interactions;"),
        M::up(
            "CREATE TABLE state_vectors(
                 mass INTEGER NOT NULL,
                 px INTEGER NOT NULL,
                 py INTEGER NOT NULL,
                 pz INTEGER NOT NULL,
                 vx INTEGER NOT NULL,
                 vy INTEGER NOT NULL,
                 vz INTEGER NOT NULL,
                 count INTEGER,
                 PRIMARY KEY (mass, px, py, pz, vx, vy, vz)
                 FOREIGN KEY (run_id) REFERENCES run_parameters(run_id) ON DELETE CASCADE
               );
               CREATE INDEX idx_state_vectors_run_id ON state_vectors(run_id);
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

pub fn persist_state_count<T: TransactionProvider>(
    state_vector: &StateVector,
    tx: &T,
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

pub fn persist_parameters<T: TransactionProvider>(
    parameters: &Parameters,
    tx: &T,
) -> Result<usize, Box<dyn Error>> {
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

    let mut last_particle_id: Option<usize> = None;
    let mut last_particle_ix: Option<usize> = None;
    for particle in parameters.particle_parameters.iter() {
        let mut stmt = tx.prepare(
            "INSERT INTO particle_parameters (mass, index, run_id)
             VALUES (?1, ?2, ?3);",
        )?;
        stmt.execute(params![particle.mass, particle.index, parameters_id])?;

        let particle_id = tx.get_last_insert_rowid() as usize;
        if let Some(last_particle_id) = last_particle_id {
            if let Some(last_particle_ix) = last_particle_ix {
                let mut stmt = tx.prepare(
                    "INSERT INTO interactions (interaction_type, parameter_id_0, parameter_id_1)
                    VALUES (?1, ?2, ?3);",
                )?;
                let interaction =
                    parameters.interaction_by_indices(last_particle_ix, particle.index)?;
                stmt.execute(params![interaction.to_string(), last_particle_id, particle_id])?;
            }
        }

        last_particle_ix = Some(particle.index);
        last_particle_id = Some(particle_id);
    }
    Ok(parameters_id as usize)
}

#[cfg(test)]
mod tests {
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
    fn test_persist_state_count() {
        let mut connection_provider = open_memory_database();
        migrate_to_latest(&mut connection_provider).unwrap();
        let tx_provider = create_transaction_provider(&mut connection_provider).unwrap();
        let state_vector = StateVector::new(1.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 10.0, 1);
        persist_state_count(&state_vector, &tx_provider).unwrap();
        commit_transaction(tx_provider).unwrap();

        let mut stmt = connection_provider
            .connection
            .prepare(
                "SELECT count FROM state_vectors
             WHERE mass = 1 AND px = 0 AND py = 0 AND pz = 0 AND vx = 0 AND vy = 0 AND vz = 0;",
            )
            .unwrap();

        let count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
