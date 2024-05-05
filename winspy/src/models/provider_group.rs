use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query, sqlite::SqliteRow, SqliteConnection};

use crate::{require_some, try_get_row};

/// A provider group.
///
///
/// # Example
/// ```no_run
/// # use crate::models::ProviderGroup;
/// ProviderGroup {
///     id: 1,
///     guid: "4F50731A-89CF-4782-B3E0-DCE8C90476BA".to_string()
/// };
/// ```
///
///
/// # Database
/// `provider_groups` table in `EventTrancript.db`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct ProviderGroup {
    /// Source: `group_id` field.
    id: i64,

    /// Source: `group_guid` field.
    guid: String,
}

impl ProviderGroup {
    #[inline]
    pub fn new(id: i64, guid: String) -> Self {
        Self { id, guid }
    }

    pub async fn load_from_database_by_id(
        connection: &mut SqliteConnection,
        group_id: i64,
    ) -> Result<Self> {
        let query_result = query!(
            "SELECT group_id, group_guid \
            FROM provider_groups \
            WHERE group_id = $1",
            group_id
        )
        .fetch_one(connection)
        .await
        .into_diagnostic()
        .wrap_err("Failed to fetch provider group by ID from database.")?;

        Ok(Self {
            id: query_result.group_id,
            guid: require_some!(query_result.group_guid, "group_guid")?,
        })
    }

    #[deprecated]
    pub fn try_from_sqlite_row(row: &SqliteRow) -> Result<Self> {
        let id: i64 = try_get_row!(row, "group_id")?;
        let guid: String = try_get_row!(row, "group_guid")?;

        Ok(Self { id, guid })
    }
}
