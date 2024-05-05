use std::fmt::Debug;

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use miette::{IntoDiagnostic, Result};
use serde_json::Value;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

use super::{category::Category, tag::Tag};

/// Used to pass event data for [PersistedEvent] creation
pub struct PersistedEventDescriptor {
    /// Date and time of event
    pub time: Option<DateTime<Utc>>,
    /// Unique SID of the device
    pub device_id: Option<String>,
    /// Name of the event, separated by full-stops
    pub event_name: Option<String>,
    pub event_name_hash: Option<i64>,
    /// Not sure but we parse it anyway
    pub is_core: Option<bool>,
    /// The process that logged the event
    pub logging_binary: Option<String>,
    /// The name of the OS component associated with event
    pub producer: Option<String>,
    /// Raw associated data of the event
    pub raw_payload: Option<String>,
    pub provider_group: Option<String>,
    pub extra1: Option<String>,
    pub extra2: Option<String>,
    pub extra3: Option<String>,
}

/// Event captured by the database
///
/// NOTE: This structure provides `json_payload` and `raw_payload`.
/// `json_payload` is used if event payload is valid JSON.
/// Otherwise `raw_payload` is used and contains original event payload.
/// Both cannot be Some at the same time.
pub struct PersistedEvent {
    time: Option<DateTime<Utc>>,
    device_id: Option<String>,
    event_name: Option<String>, // `full_event_name` in the database
    event_name_hash: Option<i64>,
    is_core: Option<bool>,
    logging_binary: Option<String>, // Binary that logged the event
    producer: Option<String>,       // Producer of the event (e.g. Windows, Edge, ...)
    provider_group: Option<String>,

    json_payload: Option<Value>, // Parsed JSON; is None if event data was not valid JSON - see raw_payload
    raw_payload: Option<String>, // Raw event data; is None if is valid JSON - see json_payload

    extra1: Option<String>, // ????
    extra2: Option<String>, // ????
    extra3: Option<String>, // ????

    categories: Vec<Category>,
    tags: Vec<Tag>,
}

impl Debug for PersistedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time = self.time.map_or("/".to_string(), |t| t.to_string());
        let device_id = self.device_id.clone().map_or("/".to_string(), |d| d);
        let event_name = self.event_name.clone().map_or("/".to_string(), |e| e);
        let logging_binary = self.logging_binary.clone().map_or("/".to_string(), |l| l);

        write!(
            f,
            "[{}] {} (device ID: {}, logging binary: {})",
            time, event_name, device_id, logging_binary
        )
    }
}

impl PersistedEvent {
    /// Creates a new [PersistedEvent]
    pub fn new(descriptor: PersistedEventDescriptor) -> Self {
        // Parse JSON payload. If not successful, set `json_payload` to None and provide
        // `raw_payload`. Otherwise set `raw_payload` to None.
        let json_payload: Option<Value> = if let Some(payload) = &descriptor.raw_payload {
            if let Ok(json_result) = serde_json::from_str(payload) {
                Some(json_result)
            } else {
                None
            }
        } else {
            None
        };
        let raw_payload = if json_payload.is_some() {
            None
        } else {
            descriptor.raw_payload
        };

        Self {
            time: descriptor.time,
            device_id: descriptor.device_id,
            event_name: descriptor.event_name,
            event_name_hash: descriptor.event_name_hash,
            is_core: descriptor.is_core,
            logging_binary: descriptor.logging_binary,
            producer: descriptor.producer,
            provider_group: descriptor.provider_group,
            json_payload,
            raw_payload,
            extra1: descriptor.extra1,
            extra2: descriptor.extra2,
            extra3: descriptor.extra3,
            categories: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn from_sql_row(row: &SqliteRow) -> Result<Self> {
        let sid: Option<String> = row.try_get("sid").into_diagnostic()?;
        let ldap_timestamp: Option<i64> = row.try_get("timestamp").into_diagnostic()?;
        let payload: Option<String> = row.try_get("payload").into_diagnostic()?;
        let event_name: Option<String> = row.try_get("full_event_name").into_diagnostic()?;
        let event_name_hash: Option<i64> = row.try_get("full_event_name_hash").into_diagnostic()?;
        let is_core: Option<bool> = row.try_get("is_core").into_diagnostic()?;
        let provider_group_guid: Option<String> = row.try_get("group_guid").into_diagnostic()?;
        let logging_binary: Option<String> = row
            .try_get("friendly_logging_binary_name")
            .into_diagnostic()?;
        let producer: Option<String> = row.try_get("producer_id_name").into_diagnostic()?;
        let extra1: Option<String> = row.try_get("extra1").into_diagnostic()?;
        let extra2: Option<String> = row.try_get("extra2").into_diagnostic()?;
        let extra3: Option<String> = row.try_get("extra3").into_diagnostic()?;

        // Time is measured in 100-ns intervals since 1. 1. 1601
        let mut time = None;
        if let Some(ldap_timestamp) = ldap_timestamp {
            if let Some(start_datetime) = Utc.with_ymd_and_hms(1601, 1, 1, 0, 0, 0).single() {
                let time_delta = TimeDelta::seconds(ldap_timestamp / 10000000);
                if let Some(final_time) = start_datetime.checked_add_signed(time_delta) {
                    time = Some(final_time);
                };
            };
        }

        // Parse JSON payload. If not successful, set `json_payload` to None and provide
        // `raw_payload`. Otherwise set `raw_payload` to None.
        let json_payload: Option<Value> = if let Some(payload) = &payload {
            if let Ok(json_result) = serde_json::from_str(payload) {
                Some(json_result)
            } else {
                None
            }
        } else {
            None
        };
        let raw_payload = if json_payload.is_some() {
            None
        } else {
            payload
        };

        Ok(Self {
            time,
            device_id: sid,
            event_name,
            event_name_hash,
            is_core,
            logging_binary,
            producer,
            provider_group: provider_group_guid,
            json_payload,
            raw_payload,
            extra1,
            extra2,
            extra3,
            categories: Vec::new(),
            tags: Vec::new(),
        })
    }

    /// Time of the event
    pub fn time(&self) -> Option<DateTime<Utc>> {
        self.time.clone()
    }

    /// Unique SID of the device
    pub fn device_id(&self) -> Option<String> {
        self.device_id.clone()
    }

    /// Name of the event
    pub fn event_name(&self) -> Option<String> {
        self.event_name.clone()
    }

    pub fn is_core(&self) -> Option<bool> {
        self.is_core
    }

    /// The binary of the process that logged the event
    pub fn logging_binary(&self) -> Option<String> {
        self.logging_binary.clone()
    }

    /// Producer of the event (e.g. Windows, Edge, ...)
    pub fn producer(&self) -> Option<String> {
        self.producer.clone()
    }

    pub fn provider_group(&self) -> Option<String> {
        self.provider_group.clone()
    }

    /// Parsed JSON payload. If None, check [Self::raw_payload]
    pub fn json_payload(&self) -> &Option<Value> {
        &self.json_payload
    }

    /// Raw string payload. If None, check [Self::json_payload]
    pub fn raw_payload(&self) -> &Option<String> {
        &self.raw_payload
    }

    pub fn extra1(&self) -> &Option<String> {
        &self.extra1
    }

    pub fn extra2(&self) -> &Option<String> {
        &self.extra2
    }

    pub fn extra3(&self) -> &Option<String> {
        &self.extra3
    }

    pub fn add_category(&mut self, category: Category) {
        self.categories.push(category);
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    /// Checks if event name includes given keyword
    pub fn event_name_includes<'a, P>(&self, keyword: &str) -> bool {
        if let Some(event_name) = &self.event_name {
            return event_name.contains(keyword);
        }
        false
    }
}
