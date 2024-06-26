use std::fmt::Debug;

use chrono::{DateTime, Utc};

use super::category::CategoryId;
use super::producer::ProducerId;
use super::provider_group::ProviderGroup;
use super::tag_description::TagDescriptionId;

pub enum PersistedEventPayload {
    None,
    Invalid { raw_payload: String },
    Parsed { payload: serde_json::Value },
}

pub struct LoggingBinary {
    pub name: String,
    pub friendly_name: String,
}

/// Event captured by the database.
#[allow(dead_code)]
pub struct PersistedEvent {
    pub(super) device_id: String,

    pub(super) timestamp: DateTime<Utc>,

    pub(super) payload: PersistedEventPayload,

    /// `full_event_name` in the database
    pub(super) event_name: String,

    pub(super) event_name_hash: i64,

    pub(super) is_core: bool,

    pub(super) provider_group: ProviderGroup,

    /// Binary that logged the event,
    pub(super) logging_binary: LoggingBinary,

    // Producer of the event (e.g. Windows, Edge, ...)
    pub(super) producer_id: ProducerId,

    pub(super) categories: Vec<CategoryId>,

    pub(super) tags: Vec<TagDescriptionId>,
}

impl Debug for PersistedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_timestamp = self.timestamp.to_rfc3339();

        write!(
            f,
            "[{}] {} (device ID: {}, logging binary: {})",
            formatted_timestamp, self.event_name, self.device_id, self.logging_binary.name
        )
    }
}

#[allow(dead_code)]
impl PersistedEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device_id: String,
        timestamp: DateTime<Utc>,
        payload: PersistedEventPayload,
        event_name: String,
        event_name_hash: i64,
        is_core: bool,
        provider_group: ProviderGroup,
        logging_binary: LoggingBinary,
        producer_id: ProducerId,
        categories: Vec<CategoryId>,
        tags: Vec<TagDescriptionId>,
    ) -> Self {
        Self {
            device_id,
            timestamp,
            payload,
            event_name,
            event_name_hash,
            is_core,
            provider_group,
            logging_binary,
            producer_id,
            categories,
            tags,
        }
    }

    /// Unique SID of the device.
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Timestamp of the event.
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn payload(&self) -> &PersistedEventPayload {
        &self.payload
    }

    /// Name of the event
    pub fn event_name(&self) -> &str {
        &self.event_name
    }

    /// Checks if event name includes given keyword
    pub fn event_name_contains(&self, keyword: &str) -> bool {
        self.event_name.contains(keyword)
    }

    pub fn event_name_hash(&self) -> i64 {
        self.event_name_hash
    }

    pub fn is_core(&self) -> bool {
        self.is_core
    }

    pub fn provider_group(&self) -> &ProviderGroup {
        &self.provider_group
    }

    /// The binary of the process that logged the event
    pub fn logging_binary(&self) -> &LoggingBinary {
        &self.logging_binary
    }

    /// Producer of the event (e.g. Windows, Edge, ...)
    pub fn producer_id(&self) -> ProducerId {
        self.producer_id
    }

    pub fn category_ids(&self) -> &[CategoryId] {
        &self.categories
    }

    pub fn tag_description_ids(&self) -> &[TagDescriptionId] {
        &self.tags
    }
}
