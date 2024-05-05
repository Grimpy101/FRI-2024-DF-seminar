use miette::Result;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::try_get_row;

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
pub struct Producer {
    id: i64,
    name: String,
}

impl Producer {
    #[inline]
    pub fn new(id: i64, name: String) -> Self {
        Self { id, name }
    }

    pub fn try_from_sqlite_row(row: &SqliteRow) -> Result<Self> {
        let id: i64 = try_get_row!(row, "producer_id")?;
        let name: String = try_get_row!(row, "producer_id_text")?;

        Ok(Self { id, name })
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
