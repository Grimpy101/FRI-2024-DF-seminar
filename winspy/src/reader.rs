use std::path::Path;

use chrono::{TimeDelta, TimeZone, Utc};
use miette::{miette, IntoDiagnostic, Result};
use sqlx::{Connection, SqliteConnection};

use crate::{
    models::{
        category::{Category, CategoryId},
        persisted_event::{LoggingBinary, PersistedEvent, PersistedEventPayload},
        producer::{Producer, ProducerId},
        provider_group::ProviderGroup,
        tag_description::{TagDescription, TagDescriptionId},
    },
    require_some,
};

pub struct EventTranscriptReader {
    connection: SqliteConnection,
}

impl EventTranscriptReader {
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

        Ok(Self { connection })
    }

    pub async fn load_all_tags(&mut self) -> Result<Vec<TagDescription>> {
        TagDescription::load_all_from_database(&mut self.connection).await
    }

    pub async fn load_all_producers(&mut self) -> Result<Vec<Producer>> {
        Producer::load_all_from_database(&mut self.connection).await
    }

    pub async fn load_all_categories(&mut self) -> Result<Vec<Category>> {
        Category::load_all_from_database(&mut self.connection).await
    }

    pub async fn load_all_events(&mut self) -> Result<Vec<PersistedEvent>> {
        let mut persisted_events = Vec::new();


        let events_query = sqlx::query!(
            "SELECT sid, timestamp, payload, full_event_name, full_event_name_hash, is_core, \
                provider_group_id, group_guid as provider_group_guid, logging_binary_name, \
                friendly_logging_binary_name, p.producer_id as producer_id, producer_id_text \
            FROM events_persisted as e \
            LEFT JOIN provider_groups as g \
                ON e.provider_group_id = g.group_id \
            LEFT JOIN producers as p \
                ON e.producer_id = p.producer_id",
        )
        .fetch_all(&mut self.connection)
        .await
        .into_diagnostic()?;

        for row in events_query {
            // Unpack columns and ensure that most of them are `Some`.
            let device_id: String = require_some!(row.sid, "sid")?;
            let raw_ldap_event_timestamp: i64 = require_some!(row.timestamp, "timestamp")?;
            let raw_payload: Option<String> = row.payload;
            let event_name: String = require_some!(row.full_event_name, "full_event_name")?;
            let event_name_hash: i64 =
                require_some!(row.full_event_name_hash, "full_event_name_hash")?;
            let is_core: bool = {
                let is_core_integer: i64 = require_some!(row.is_core, "is_core")?;
                is_core_integer != 0
            };
            let provider_group_id: i64 = require_some!(row.provider_group_id, "provider_group_id")?;
            let provider_group_guid: String =
                require_some!(row.provider_group_guid, "provider_group_guid")?;
            let logging_binary_name: String =
                require_some!(row.logging_binary_name, "logging_binary_name")?;
            let logging_binary_friendly_name: String = require_some!(
                row.friendly_logging_binary_name,
                "friendly_logging_binary_name"
            )?;
            let producer_id: i64 = row.producer_id;


            // Parse a LDAP timestamp into a UTC one.
            let event_timestamp = {
                let ldap_starting_offset = Utc
                    .with_ymd_and_hms(1601, 1, 1, 0, 0, 0)
                    .single()
                    .ok_or_else(|| miette!("Failed to construct initial LDAP timestamp offset."))?;

                // Time is measured in 100-ns intervals since 1. 1. 1601
                let time_delta_since_offset =
                    TimeDelta::seconds(raw_ldap_event_timestamp / 10000000);

                ldap_starting_offset
                    .checked_add_signed(time_delta_since_offset)
                    .ok_or_else(|| miette!("Failed to construct UTC evnt timestamp."))?
            };


            // Load all related categories (but only their IDs).
            let category_ids =
                CategoryId::load_all_from_database_for_event(&mut self.connection, event_name_hash)
                    .await?;

            // Load all related tag descriptions (but only their IDs).
            let tag_ids = TagDescriptionId::load_all_from_database_for_event(
                &mut self.connection,
                event_name_hash,
            )
            .await?;


            // Parse the event payload, if any, as JSON.
            let event_payload = if let Some(payload) = raw_payload {
                match serde_json::from_str(&payload) {
                    Ok(parsed_payload) => PersistedEventPayload::Parsed {
                        payload: parsed_payload,
                    },
                    Err(_) => PersistedEventPayload::Invalid {
                        raw_payload: payload,
                    },
                }
            } else {
                PersistedEventPayload::None
            };


            // Parse event provider, logging binary and producer ID.
            let event_provider_group = ProviderGroup::new(provider_group_id, provider_group_guid);

            let logging_binary = LoggingBinary {
                name: logging_binary_name,
                friendly_name: logging_binary_friendly_name,
            };

            let producer = ProducerId::new(producer_id);


            // Structure the entire event into a [`PersistedEvent`] for future use.
            let persisted_event = PersistedEvent {
                device_id,
                timestamp: event_timestamp,
                payload: event_payload,
                event_name,
                event_name_hash,
                is_core,
                provider_group: event_provider_group,
                logging_binary,
                producer_id: producer,
                categories: category_ids,
                tags: tag_ids,
            };

            persisted_events.push(persisted_event);
        }


        Ok(persisted_events)
    }
}
