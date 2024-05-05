use miette::{IntoDiagnostic, Result};
use sqlx::Row;
use sqlx::{sqlite::SqliteRow, SqliteConnection};

#[derive(Debug)]
pub struct Category {
    id: Option<i64>,
    name: Option<String>,
    producer: Option<String>,
}

impl Category {
    pub fn new(id: Option<i64>, name: Option<String>, producer: Option<String>) -> Self {
        Self { id, name, producer }
    }

    pub fn from_sql_row(row: SqliteRow) -> Result<Self> {
        let id: Option<i64> = row.try_get("category_id").into_diagnostic()?;
        let name: Option<String> = row.try_get("category_id_text").into_diagnostic()?;
        let producer: Option<String> = row.try_get("producer_id_name").into_diagnostic()?;

        Ok(Self { id, name, producer })
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn producer(&self) -> Option<String> {
        self.producer.clone()
    }
}
