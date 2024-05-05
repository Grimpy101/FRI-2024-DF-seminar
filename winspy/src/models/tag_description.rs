use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query, sqlite::SqliteRow, SqliteConnection};

use crate::{require_some, try_get_row};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TagDescriptionId(i64);

impl TagDescriptionId {
    pub async fn load_all_from_database_for_event(
        connection: &mut SqliteConnection,
        full_event_name_hash: i64,
    ) -> Result<Vec<Self>> {
        let query_results = sqlx::query!(
            "SELECT tag_id \
            FROM event_tags et \
            WHERE et.full_event_name_hash = $1",
            full_event_name_hash
        )
        .fetch_all(connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to fetch all tag IDs for event from database.")?;


        let mut matching_tag_ids = Vec::with_capacity(query_results.len());

        for query_result in query_results {
            let tag_id: i64 = require_some!(query_result.tag_id, "tag_id")?;

            matching_tag_ids.push(Self(tag_id));
        }

        Ok(matching_tag_ids)
    }
}



/// An event tag (a category in a sense).
///
///
/// # Example
/// ```no_run
/// # use crate::models::Tag;
/// Tag {
///     id: 1,
///     name: "Browsing History".to_string(),
///     description: "Records of the web browsing history ...".to_string(),
///     locale: "en-US".to_string()
/// };
/// ```
///
///
/// # Database
/// `tag_descriptions` table in `EventTrancript.db`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagDescription {
    id: i64,
    name: String,
    description: String,
    locale: String,
}

impl TagDescription {
    #[inline]
    pub fn new(id: i64, name: String, description: String, locale: String) -> Self {
        Self {
            id,
            name,
            description,
            locale,
        }
    }

    pub async fn load_all_from_database(connection: &mut SqliteConnection) -> Result<Vec<Self>> {
        let query_results = query!(
            "SELECT tag_id, locale_name, tag_name, description \
            FROM tag_descriptions"
        )
        .fetch_all(connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to fetch all tag descriptions from database.")?;


        let mut tag_descriptions = Vec::with_capacity(query_results.len());

        for query_result in query_results {
            let parsed_tag = Self {
                id: require_some!(query_result.tag_id, "tag_id")?,
                name: require_some!(query_result.tag_name, "tag_name")?,
                locale: require_some!(query_result.locale_name, "locale_name")?,
                description: require_some!(query_result.description, "description")?,
            };

            tag_descriptions.push(parsed_tag);
        }

        Ok(tag_descriptions)
    }

    pub fn try_from_sqlite_row(row: &SqliteRow) -> Result<Self> {
        let id: i64 = try_get_row!(row, "tag_id")?;
        let name: String = try_get_row!(row, "tag_name")?;
        let description: String = try_get_row!(row, "description")?;
        let locale: String = try_get_row!(row, "locale_name")?;

        Ok(Self {
            id,
            name,
            description,
            locale,
        })
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }
}
