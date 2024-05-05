use miette::Result;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::try_get_row;

#[derive(Debug)]
pub struct Category {
    id: i64,
    name: String,
    producer: String,
}

impl Category {
    #[inline]
    pub fn new(id: i64, name: String, producer: String) -> Self {
        Self { id, name, producer }
    }

    /// The provided row must be `LEFT JOIN`-ed to the `producers` table on the `producer_id`.
    pub fn from_sql_row(row: &SqliteRow) -> Result<Self> {
        let id: i64 = try_get_row!(row, "category_id")?;
        let name: String = try_get_row!(row, "category_id_text")?;
        let producer: String = try_get_row!(row, "producer_id_text")?;

        Ok(Self { id, name, producer })
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn producer(&self) -> &str {
        &self.producer
    }
}
