use miette::{Context, IntoDiagnostic, Result};
use sqlx::sqlite::SqliteRow;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use thiserror::Error;

/// Run-time serialized column from [`SqliteRow`] for diagnostics.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SavedSqliteColumn {
    pub name: String,
    pub ordinal: usize,
    pub r#type: String,

    /// `None` if the field was null, `String` is not.
    /// All types, including numeric ones, are serialized
    /// as a string.
    pub value: Option<String>,
}

/// Run-time serialized [`SqliteRow`] for diagnostics.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SavedSqliteRow {
    pub columns: Vec<SavedSqliteColumn>,
}

macro_rules! try_deserialize_some_from_row {
    ($row:expr, $column:expr, $column_type:expr => $type:ty) => {{
        let _value: $type = $row
            .try_get($column.ordinal())
            .into_diagnostic()
            .wrap_err(format!(
                "Column type was {}, but could not store as $type.",
                $column_type
            ))?;

        Some(_value)
    }};
}

#[allow(dead_code)]
impl SavedSqliteRow {
    pub fn from_sqlite_row(sqlite_row: &SqliteRow) -> Result<Self> {
        let mut saved_columns: Vec<SavedSqliteColumn> = Vec::with_capacity(sqlite_row.len());

        for column in sqlite_row.columns() {
            let column_value = sqlite_row
                .try_get_raw(column.ordinal())
                .into_diagnostic()
                .wrap_err("Failed to get raw value from sqlite row.")?;

            let column_type_info = column_value.type_info();
            let column_type_name = column_type_info.name();

            // See also: <https://docs.rs/sqlx-sqlite/0.7.4/src/sqlx_sqlite/type_info.rs.html#35>
            let saved_string_value: Option<String> = match column_type_name {
                "NULL" => None,
                "TEXT" => try_deserialize_some_from_row!(sqlite_row, column, "TEXT" => String),
                "REAL" => try_deserialize_some_from_row!(sqlite_row, column, "REAL" => f64)
                    .map(|value| value.to_string()),
                "BLOB" => try_deserialize_some_from_row!(sqlite_row, column, "BLOB" => Vec<u8>)
                    .map(|value| format!("[{}]", itertools::join(value, ", "))),
                "INTEGER" => try_deserialize_some_from_row!(sqlite_row, column, "INTEGER" => i64)
                    .map(|value| value.to_string()),
                "NUMERIC" => try_deserialize_some_from_row!(sqlite_row, column, "NUMERIC" => f64)
                    .map(|value| value.to_string()),
                "BOOLEAN" => try_deserialize_some_from_row!(sqlite_row, column, "BOOLEAN" => bool)
                    .map(|value| value.to_string()),
                "DATE" => try_deserialize_some_from_row!(sqlite_row, column, "DATE" => String),
                "TIME" => try_deserialize_some_from_row!(sqlite_row, column, "TIME" => String),
                "DATETIME" => {
                    try_deserialize_some_from_row!(sqlite_row, column, "DATETIME" => String)
                }
                _ => panic!(
                    "unexpected SQLite type name: {}",
                    column_type_name
                ),
            };

            saved_columns.push(SavedSqliteColumn {
                name: column.name().to_string(),
                ordinal: column.ordinal(),
                r#type: column_type_name.to_string(),
                value: saved_string_value,
            });
        }

        Ok(Self {
            columns: saved_columns,
        })
    }
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum EventReaderError {
    #[error("failed to parse record from table {table_name}")]
    RecordParsingError {
        table_name: String,
        saved_row: SavedSqliteRow,
    },
}
