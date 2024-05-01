use miette::{IntoDiagnostic, Result};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

pub struct Tag {
    id: Option<i64>,
    name: Option<String>,
    description: Option<String>,
    locale: Option<String>,
}

impl Tag {
    pub fn new(
        id: Option<i64>,
        name: Option<String>,
        description: Option<String>,
        locale: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            locale,
        }
    }

    pub fn from_sql_row(row: SqliteRow) -> Result<Self> {
        let id: Option<i64> = row.try_get("tag_id").into_diagnostic()?;
        let name: Option<String> = row.try_get("tag_name").into_diagnostic()?;
        let description: Option<String> = row.try_get("description").into_diagnostic()?;
        let locale: Option<String> = row.try_get("locale_name").into_diagnostic()?;

        Ok(Self {
            id,
            name,
            description,
            locale,
        })
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn locale(&self) -> Option<String> {
        self.locale.clone()
    }
}
