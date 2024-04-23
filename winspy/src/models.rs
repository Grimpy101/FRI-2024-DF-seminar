use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use miette::{miette, IntoDiagnostic, Result};
use serde_json::Value;
use sqlx::{Connection, Row, SqliteConnection};

#[derive(Debug)]
pub enum MSStoreEvent {
    Launching,
    Launched,
    Activating,
    Activated,
    AuthRequest,
    BeginPurchase,
    FinishPurchase,
    Other,
}

impl MSStoreEvent {
    pub fn display_name(&self) -> String {
        match self {
            MSStoreEvent::Launching => "Launching",
            MSStoreEvent::Launched => "Launched",
            MSStoreEvent::Activating => "Activating",
            MSStoreEvent::Activated => "Activated",
            MSStoreEvent::AuthRequest => "Authentication Request",
            MSStoreEvent::BeginPurchase => "Purchase Begins",
            MSStoreEvent::FinishPurchase => "Purchase Finished",
            MSStoreEvent::Other => "Unknown Event",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub enum WindowsSoftwareClient {
    CheckForUpdates,
    UpdateDetected,
    Installing,
    Downloading,
    Other,
}

impl WindowsSoftwareClient {
    pub fn display_name(&self) -> String {
        match self {
            WindowsSoftwareClient::CheckForUpdates => "Checking for Updates",
            WindowsSoftwareClient::UpdateDetected => "Detected Update",
            WindowsSoftwareClient::Installing => "Installing",
            WindowsSoftwareClient::Downloading => "Downloading",
            WindowsSoftwareClient::Other => "Unknown Event",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub enum EventCategory {
    Device,
    Edge,
    WiFi,
    MSStore(MSStoreEvent),
    WindowsSoftwareClient(WindowsSoftwareClient),
    Other,
}

impl EventCategory {
    pub fn display_name(&self) -> String {
        match self {
            EventCategory::Device => todo!(),
            EventCategory::Edge => todo!(),
            EventCategory::WiFi => todo!(),
            EventCategory::MSStore(store_event) => {
                format!("Microsoft Store: {}", store_event.display_name())
            }
            EventCategory::WindowsSoftwareClient(client_event) => format!(
                "Windows Software Client Telemetry: {}",
                client_event.display_name()
            ),
            EventCategory::Other => "Unknown Event".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct EventPersisted {
    device_id: Option<String>,
    time: Option<DateTime<Utc>>,
    event_name: Option<String>,
    process_name: Option<String>,
    event_category: EventCategory,
    json_payload: Option<Value>,
    raw_payload: Option<String>,
}

impl EventPersisted {
    pub fn new(
        device_id: Option<String>,
        time: Option<DateTime<Utc>>,
        event_name: Option<String>,
        process_name: Option<String>,
        payload: Option<String>,
    ) -> Result<Self> {
        let json_result: Option<Value> = if let Some(payload) = payload.clone() {
            let json_result: Value = serde_json::from_str(&payload).into_diagnostic()?;
            Some(json_result)
        } else {
            None
        };

        let event_category = Self::detect_event_category(&event_name, &json_result);

        Ok(Self {
            device_id,
            time,
            event_name,
            process_name,
            json_payload: json_result,
            raw_payload: payload,
            event_category,
        })
    }

    fn detect_event_category(
        event_name: &Option<String>,
        json_payload: &Option<Value>,
    ) -> EventCategory {
        let Some(event_name) = event_name else {
            return EventCategory::Other;
        };

        if event_name.starts_with("Microsoft-Windows-Store") {
            return match event_name.as_str() {
                "Microsoft-Windows-Store.StoreLaunching" => {
                    EventCategory::MSStore(MSStoreEvent::Launching)
                }
                "Microsoft-Windows-Store.StoreActivating" => {
                    EventCategory::MSStore(MSStoreEvent::Activating)
                }
                "Microsoft-Windows-Store.OutgoingServiceRequest" => {
                    let Some(json_payload) = json_payload else {
                        return EventCategory::MSStore(MSStoreEvent::Other);
                    };
                    if let Some(Value::String(data)) =
                        json_payload.pointer("data/baseData/dependencyType")
                    {
                        if data == "AuthenticationRequest" {
                            return EventCategory::MSStore(MSStoreEvent::AuthRequest);
                        }
                    }
                    EventCategory::MSStore(MSStoreEvent::Other)
                }
                "Microsoft-Windows-Store.StoreActivated" => {
                    EventCategory::MSStore(MSStoreEvent::Activated)
                }
                "Microsoft-Windows-Store.StoreLaunched" => {
                    EventCategory::MSStore(MSStoreEvent::Launched)
                }
                "Microsoft-Windows-Store.PurchaseBegin" => {
                    EventCategory::MSStore(MSStoreEvent::BeginPurchase)
                }
                "Microsoft-Windows-Store.PurchaseOrderFulfillment" => {
                    EventCategory::MSStore(MSStoreEvent::FinishPurchase)
                }
                _ => EventCategory::MSStore(MSStoreEvent::Other),
            };
        }

        if event_name.starts_with("SoftwareUpdateClientTelemetry") {
            return match event_name.as_str() {
                "SoftwareUpdateClientTelemetry.CheckForUpdates" => {
                    EventCategory::WindowsSoftwareClient(WindowsSoftwareClient::CheckForUpdates)
                }
                "SoftwareUpdateClientTelemetry.UpdateDetected" => {
                    EventCategory::WindowsSoftwareClient(WindowsSoftwareClient::UpdateDetected)
                }
                "SoftwareUpdateClientTelemetry.Download" => {
                    EventCategory::WindowsSoftwareClient(WindowsSoftwareClient::Downloading)
                }
                "SoftwareUpdateClientTelemetry.Install" => {
                    EventCategory::WindowsSoftwareClient(WindowsSoftwareClient::Installing)
                }
                _ => EventCategory::WindowsSoftwareClient(WindowsSoftwareClient::Other),
            };
        }

        EventCategory::Other
    }

    pub fn event_category(&self) -> &EventCategory {
        &self.event_category
    }
}

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
    events_persisted: Vec<EventPersisted>,
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
    ) -> Result<Vec<EventPersisted>> {
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

            let entry = EventPersisted::new(device_id, time, event_name, process_name, payload)?;
            entries.push(entry);
        }

        println!("DB: Retrieved {} `events_persisted` records", entries.len());

        Ok(entries)
    }

    pub fn print_detected_events(&self) {
        for persisted_event in self.events_persisted.iter() {
            let time = if let Some(time) = persisted_event.time {
                time.to_string()
            } else {
                "<MISSING>".to_string()
            };
            println!(
                "[{}]  --  {}",
                time,
                persisted_event.event_category.display_name(),
            )
        }
    }
}
