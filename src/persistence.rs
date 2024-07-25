use lazy_static::lazy_static;
use rusqlite::{params, Connection, Result, Statement, Transaction};
use rusqlite_migration::{Migrations, M};
use std::error::Error;

use crate::particle::StateVector;

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
        let state_vector = StateVector::new(1.0, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 10.0);
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
