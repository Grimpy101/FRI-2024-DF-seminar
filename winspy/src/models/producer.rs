use miette::{IntoDiagnostic, Result};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

pub struct Producer {
    id: Option<i64>,
    name: Option<String>,
}

impl Producer {
    pub fn new(id: Option<i64>, name: Option<String>) -> Self {
        Self { id, name }
    }

    pub fn from_sql_row(row: SqliteRow) -> Result<Self> {
        let id: Option<i64> = row.try_get("producer_id").into_diagnostic()?;
        let name: Option<String> = row.try_get("producer_id_name").into_diagnostic()?;

        Ok(Self { id, name })
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
}
