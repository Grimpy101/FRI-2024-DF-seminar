use std::collections::HashMap;

use chrono::{TimeDelta, TimeZone, Utc};
use miette::{miette, IntoDiagnostic, Result};
use sqlx::{Connection, Row, SqliteConnection};

use self::event::Event;

mod event;
mod event_category;
mod ms_store_event;
mod windows_software_client;

#[derive(Debug)]
pub struct TableSchema {
    table_name: String,
    attributes: HashMap<String, String>,
}

impl TableSchema {
    pub fn new(table_name: String, attributes: HashMap<String, String>) -> Self {
        Self {
            table_name,
            attributes,
        }
    }

    pub fn table_name(&self) -> String {
        self.table_name.clone()
    }

    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    pub fn get_attribute(&self, attribute: String) -> Option<(String, String)> {
        if let Some(attribute_type) = self.attributes.get(&attribute) {
            return Some((attribute, attribute_type.clone()));
        };
        None
    }
}

#[derive(Debug)]
pub struct EventDatabase {
    events_persisted: Vec<Event>,
    schemas: Vec<TableSchema>,
}

impl EventDatabase {
    pub async fn events_from_file(file_path: &str) -> Result<Self> {
        let mut schemas = Vec::new();

        let url = format!("sqlite:{}", file_path);
        let mut connection = SqliteConnection::connect(&url).await.into_diagnostic()?;

        let tables = Self::retrieve_tables(&mut connection).await?;

        if !tables.contains(&"events_persisted".to_string()) {
            return Err(miette!("No `events_persisted` data table!"));
        }

        for table in tables {
            let attributes =
                Self::retrieve_table_attributes(&mut connection, table.clone()).await?;
            let schema = TableSchema::new(table, attributes);
            schemas.push(schema);
        }

        let events_persisted = Self::get_events_persisted_contents(&mut connection).await?;

        Ok(Self {
            events_persisted,
            schemas,
        })
    }

    async fn retrieve_tables(connection: &mut SqliteConnection) -> Result<Vec<String>> {
        let mut tables = Vec::new();

        let records = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(connection)
            .await
            .into_diagnostic()?;

        for record in records {
            let table_name = record.try_get(0).into_diagnostic()?;
            tables.push(table_name);
        }
        println!("DB: Retrieved {} tables", tables.len());

        Ok(tables)
    }

    async fn retrieve_table_attributes(
        connection: &mut SqliteConnection,
        table_name: String,
    ) -> Result<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        let statement = format!("PRAGMA table_info({})", table_name);
        let records = sqlx::query(&statement)
            .fetch_all(connection)
            .await
            .into_diagnostic()?;

        for record in records {
            let attribute_name: String = record.try_get(1).into_diagnostic()?;
            let attribute_type: String = record.try_get(2).into_diagnostic()?;
            attributes.insert(attribute_name, attribute_type);
        }

        Ok(attributes)
    }

    async fn get_events_persisted_contents(
        connection: &mut SqliteConnection,
    ) -> Result<Vec<Event>> {
        let mut entries = Vec::new();

        let records = sqlx::query("SELECT * FROM events_persisted")
            .fetch_all(connection)
            .await
            .into_diagnostic()?;

        for record in records {
            let ldap_timestamp: Option<i64> = record.try_get("timestamp").into_diagnostic()?;
            let payload: Option<String> = record.try_get("payload").into_diagnostic()?;
            let event_name: Option<String> = record.try_get("full_event_name").into_diagnostic()?;
            let process_name: Option<String> =
                record.try_get("logging_binary_name").into_diagnostic()?;
            let device_id: Option<String> = record.try_get("sid").into_diagnostic()?;

            let mut time = None;

            if let Some(ldap_timestamp) = ldap_timestamp {
                if let Some(start_datetime) = Utc.with_ymd_and_hms(1601, 1, 1, 0, 0, 0).single() {
                    let time_delta = TimeDelta::seconds(ldap_timestamp / 10000000);
                    if let Some(final_time) = start_datetime.checked_add_signed(time_delta) {
                        time = Some(final_time);
                    };
                };
            }

            let entry = Event::new(device_id, time, event_name, process_name, payload)?;
            entries.push(entry);
        }

        println!("DB: Retrieved {} `events_persisted` records", entries.len());

        Ok(entries)
    }

    pub fn print_detected_events(&self) {
        for persisted_event in self.events_persisted.iter() {
            let time = if let Some(time) = persisted_event.time() {
                time.to_string()
            } else {
                "<MISSING>".to_string()
            };
            println!(
                "[{}]  --  {}",
                time,
                persisted_event.event_category().display_name(),
            )
        }
    }
}
