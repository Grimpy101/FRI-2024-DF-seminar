use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use miette::Result;

pub struct Category {
    id: Option<i64>,
    name: Option<String>,
    producer: Option<String>,
}

impl Category {
    pub fn new(id: Option<i64>, name: Option<String>, producer: Option<String>) -> Self {
        Self { id, name, producer }
    }

    /*pub fn from_sql_row(row: SqliteRow) -> Result<Self> {
        let id = row.try_get("category_id")
    }*/

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
