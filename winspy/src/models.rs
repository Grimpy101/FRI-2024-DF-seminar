use std::path::Path;

use miette::Result;
use miette::{miette, IntoDiagnostic};
use sqlx::{Connection, SqliteConnection};

use self::error::SavedSqliteRow;
use self::{
    category::Category,
    error::EventReaderError,
    persisted_event::PersistedEvent,
    producer::Producer,
    tag::Tag,
};

pub mod category;
pub mod error;
mod macros;
pub mod persisted_event;
pub mod producer;
pub mod tag;



pub struct EventReader {
    connection: SqliteConnection,
    errors: Vec<EventReaderError>,
}

impl EventReader {
    pub async fn new(database_path: &Path) -> Result<Self> {
        if !database_path.exists() || !database_path.is_file() {
            return Err(miette!(
                "Provided file does not exist or is not a file."
            ));
        }

        let Some(path_str) = database_path.to_str() else {
            return Err(miette!(
                "While file does exist, its name is not valid UTF-8."
            ));
        };

        let database_url = format!("sqlite:{}", path_str);
        let connection = SqliteConnection::connect(&database_url)
            .await
            .into_diagnostic()?;

        let errors = Vec::new();

        Ok(Self { connection, errors })
    }

    pub async fn load_all_tags(&mut self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();

        let tag_records = sqlx::query("SELECT * FROM tag_descriptions")
            .fetch_all(&mut self.connection)
            .await
            .into_diagnostic()?;


        for tag_record in tag_records {
            let Ok(tag) = Tag::try_from_sqlite_row(&tag_record) else {
                self.errors.push(EventReaderError::RecordParsingError {
                    table_name: "tag_descriptions".to_string(),
                    saved_row: SavedSqliteRow::from_sqlite_row(&tag_record)?,
                });

                continue;
            };

            tags.push(tag);
        }

        Ok(tags)
    }

    pub async fn load_all_producers(&mut self) -> Result<Vec<Producer>> {
        let mut producers = Vec::new();

        let producer_records = sqlx::query("SELECT * FROM producers")
            .fetch_all(&mut self.connection)
            .await
            .into_diagnostic()?;

        for producer_record in producer_records {
            let Ok(producer) = Producer::try_from_sqlite_row(&producer_record) else {
                self.errors.push(EventReaderError::RecordParsingError {
                    table_name: "producers".to_string(),
                    saved_row: SavedSqliteRow::from_sqlite_row(&producer_record)?,
                });

                continue;
            };

            producers.push(producer);
        }

        Ok(producers)
    }

    pub async fn load_all_categories(&mut self) -> Result<Vec<Category>> {
        let mut categories = Vec::new();

        let category_records = sqlx::query(
            "SELECT * FROM categories \
            LEFT JOIN producers ON categories.producer_id = producers.producer_id",
        )
        .fetch_all(&mut self.connection)
        .await
        .into_diagnostic()?;


        for category_record in category_records {
            let Ok(category) = Category::from_sql_row(&category_record) else {
                self.errors.push(EventReaderError::RecordParsingError {
                    table_name: "categories+producers".to_string(),
                    saved_row: SavedSqliteRow::from_sqlite_row(&category_record)?,
                });

                continue;
            };

            categories.push(category);
        }

        Ok(categories)
    }

    pub async fn load_all_events(&mut self) -> Result<Vec<PersistedEvent>> {
        let mut events = Vec::new();

        let event_records = sqlx::query(
            "SELECT * FROM events_persisted \
            LEFT JOIN provider_groups ON provider_group_id = group_id \
            LEFT JOIN producers ON events_persisted.producer_id = producers.producer_id",
        )
        .fetch_all(&mut self.connection)
        .await
        .into_diagnostic()?;


        for event_record in event_records {
            let Ok(event) = PersistedEvent::from_sql_row(&event_record) else {
                self.errors.push(EventReaderError::RecordParsingError {
                    table_name: "events_persisted+provider_groups+producers".to_string(),
                    saved_row: SavedSqliteRow::from_sqlite_row(&event_record)?,
                });

                continue;
            };

            events.push(event);
        }

        Ok(events)
    }

    pub async fn load_tags_by_event(&mut self, event_name_hash: Option<i64>) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();

        let tag_records = sqlx::query(
            "SELECT * FROM event_tags \
            LEFT JOIN tag_descriptions ON event_tags.tag_id = tag_descriptions.tag_id \
            WHERE full_event_name_hash = ?",
        )
        .bind(event_name_hash)
        .fetch_all(&mut self.connection)
        .await
        .into_diagnostic()?;


        for tag_record in tag_records {
            let Ok(tag) = Tag::try_from_sqlite_row(&tag_record) else {
                self.errors.push(EventReaderError::RecordParsingError {
                    table_name: "event_tags+tag_descriptions".to_string(),
                    saved_row: SavedSqliteRow::from_sqlite_row(&tag_record)?,
                });

                continue;
            };

            tags.push(tag);
        }

        Ok(tags)
    }

    pub async fn load_categories_by_event(
        &mut self,
        event_name_hash: Option<i64>,
    ) -> Result<Vec<Category>> {
        let mut categories = Vec::new();

        let category_records = sqlx::query(
            "SELECT * FROM event_categories \
            LEFT JOIN categories ON event_categories.category_id = categories.category_id \
            LEFT JOIN producers ON categories.producer_id = producers.producer_id \
            WHERE full_event_name_hash = ?",
        )
        .bind(event_name_hash)
        .fetch_all(&mut self.connection)
        .await
        .into_diagnostic()?;


        for category_record in category_records {
            let Ok(category) = Category::from_sql_row(&category_record) else {
                self.errors.push(EventReaderError::RecordParsingError {
                    table_name: "event_categories+categories+producers".to_string(),
                    saved_row: SavedSqliteRow::from_sqlite_row(&category_record)?,
                });

                continue;
            };

            categories.push(category);
        }

        Ok(categories)
    }
}
