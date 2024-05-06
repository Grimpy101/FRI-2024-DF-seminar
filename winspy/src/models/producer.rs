use std::borrow::BorrowMut;

use miette::{Context, IntoDiagnostic, Result};
use sqlx::SqliteConnection;

use crate::require_some;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ProducerId(i64);

impl ProducerId {
    #[inline]
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
struct ProducersTableRecord {
    pub producer_id: i64,
    pub producer_id_name: Option<String>,
}

/// Describes an event producer (program or executable).
///
/// # Example
/// ```no_run
/// # use crate::models::Producer;
/// Tag {
///     id: 1,
///     name: "Windows".to_string()
/// }
/// ```
///
/// # Source
/// `producers` table in `EventTrancript.db`.
#[allow(dead_code)]
pub struct Producer {
    pub(super) id: ProducerId,
    pub(super) name: String,
}

#[allow(dead_code)]
impl Producer {
    #[inline]
    pub fn new(id: ProducerId, name: String) -> Self {
        Self { id, name }
    }

    pub async fn load_all_from_database(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let mut query_results: Result<Vec<ProducersTableRecord>> =
            sqlx::query_as("SELECT producer_id, producer_id_name FROM producers")
                .fetch_all(connection.borrow_mut())
                .await
                .into_diagnostic()
                .wrap_err("Failed to load all producers from database.");

        if query_results.is_err() {
            query_results = sqlx::query_as(
                "SELECT producer_id, producer_id_text as producer_id_name FROM producers",
            )
            .fetch_all(connection)
            .await
            .into_diagnostic()
            .wrap_err("Failed to load all producers from database.")
        }

        let query_results = query_results?;

        let mut parsed_producers = Vec::with_capacity(query_results.len());

        for query_result in query_results {
            let parsed_producer = Self {
                id: ProducerId::new(query_result.producer_id),
                name: require_some!(query_result.producer_id_name, "producer_id_text")?,
            };

            parsed_producers.push(parsed_producer);
        }

        Ok(parsed_producers)
    }

    pub fn id(&self) -> ProducerId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
