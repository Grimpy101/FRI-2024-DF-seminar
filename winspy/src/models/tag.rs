use miette::Result;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use crate::try_get_row;

/// Describes an event tag (category in a sense).
///
/// # Example
/// ```no_run
/// # use crate::models::Tag;
/// Tag {
///     id: 1,
///     name: "Browsing History".to_string(),
///     description: "Records of the web browsing history ...".to_string(),
///     locale: "en-US".to_string()
/// }
/// ```
///
/// # Source
/// `tag_descriptions` table in `EventTrancript.db`.
#[derive(Debug, PartialEq, Eq)]
pub struct Tag {
    id: i64,
    name: String,
    description: String,
    locale: String,
}

impl Tag {
    #[inline]
    pub fn new(id: i64, name: String, description: String, locale: String) -> Self {
        Self {
            id,
            name,
            description,
            locale,
        }
    }

    pub fn try_from_sqlite_row(row: SqliteRow) -> Result<Self> {
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
