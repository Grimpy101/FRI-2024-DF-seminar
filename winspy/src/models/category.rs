use miette::{Context, IntoDiagnostic, Result};
use sqlx::SqliteConnection;

use super::producer::ProducerId;
use crate::require_some;


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CategoryId(i64);

impl CategoryId {
    #[inline]
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub async fn load_all_from_database_for_event(
        connection: &mut SqliteConnection,
        full_event_name_hash: i64,
    ) -> Result<Vec<Self>> {
        let query_results = sqlx::query!(
            "SELECT category_id \
            FROM event_categories as ec \
            WHERE ec.full_event_name_hash = $1",
            full_event_name_hash
        )
        .fetch_all(connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to fetch all event categories by event name hash.")?;

        let mut matching_category_ids = Vec::with_capacity(query_results.len());

        for category_result in query_results {
            let category_id: i64 = require_some!(category_result.category_id, "category_id")?;

            matching_category_ids.push(Self(category_id));
        }

        Ok(matching_category_ids)
    }
}


#[derive(Debug)]
#[allow(dead_code)]
pub struct Category {
    pub(super) id: CategoryId,
    pub(super) name: String,
    pub(super) producer_id: ProducerId,
}

#[allow(dead_code)]
impl Category {
    #[inline]
    pub fn new(id: CategoryId, name: String, producer_id: ProducerId) -> Self {
        Self {
            id,
            name,
            producer_id,
        }
    }

    pub async fn load_all_from_database(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let query_results = sqlx::query!(
            "SELECT category_id, category_id_text, producer_id \
            FROM categories"
        )
        .fetch_all(connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to load all categories from database.")?;


        let mut parsed_categories = Vec::with_capacity(query_results.len());

        for query_result in query_results {
            let parsed_category: Category = Self {
                id: CategoryId::new(require_some!(
                    query_result.producer_id,
                    "producer_id"
                )?),
                name: require_some!(query_result.category_id_text, "category_id_text")?,
                producer_id: ProducerId::new(require_some!(
                    query_result.producer_id,
                    "producer_id"
                )?),
            };

            parsed_categories.push(parsed_category);
        }

        Ok(parsed_categories)
    }

    pub fn id(&self) -> CategoryId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn producer_id(&self) -> ProducerId {
        self.producer_id
    }
}
