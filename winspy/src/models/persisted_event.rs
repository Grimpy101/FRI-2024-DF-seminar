use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{category::Category, tag::Tag};

/// Used to pass event data for [PersistedEvent] creation
pub struct PersistedEventDescriptor {
    /// Date and time of event
    pub time: Option<DateTime<Utc>>,
    /// Unique SID of the device
    pub device_id: Option<String>,
    /// Name of the event, separated by full-stops
    pub event_name: Option<String>,
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
