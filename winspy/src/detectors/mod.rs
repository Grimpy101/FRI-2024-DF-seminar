use std::collections::HashMap;

use chrono::prelude::{DateTime, Utc};
use miette::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::{
    application::{ApplicationEvent, ApplicationEventDetector},
    battery::{BatteryEvent, BatteryEventDetector},
};
use crate::{
    models::{
        category::{Category, CategoryId},
        persisted_event::PersistedEvent,
        producer::{Producer, ProducerId},
        tag_description::{TagDescription, TagDescriptionId},
    },
    reader::EventTranscriptReader,
};

mod application;
mod battery;
mod usb;



pub struct EventTranscriptProcessor {
    events: Vec<PersistedEvent>,
    tags: HashMap<TagDescriptionId, TagDescription>,
    producers: HashMap<ProducerId, Producer>,
    categories: HashMap<CategoryId, Category>,
}

impl EventTranscriptProcessor {
    pub async fn new_from_event_transcript_reader(
        mut reader: EventTranscriptReader,
    ) -> Result<Self> {
        let events = reader
            .load_all_events()
            .await
            .wrap_err("Failed to load all persisted events.")?;

        let tags = reader
            .load_all_tags()
            .await
            .wrap_err("Failed to load all tags.")?;

        let producers = reader
            .load_all_producers()
            .await
            .wrap_err("Failed to load all producers.")?;

        let categories = reader
            .load_all_categories()
            .await
            .wrap_err("Failed to load all categories.")?;


        let mut tags_map = HashMap::with_capacity(tags.len());
        for tag in tags {
            tags_map.insert(tag.id(), tag);
        }

        let mut producers_map = HashMap::with_capacity(producers.len());
        for producer in producers {
            producers_map.insert(producer.id(), producer);
        }

        let mut categories_map = HashMap::with_capacity(categories.len());
        for category in categories {
            categories_map.insert(category.id(), category);
        }


        Ok(Self {
            events,
            tags: tags_map,
            producers: producers_map,
            categories: categories_map,
        })
    }

    pub fn process_events(self, mut primary_detector: AllDetectors) -> Vec<ProcessedEvent> {
        let read_only_view = EventTranscriptReadOnlyView {
            tags: &self.tags,
            producers: &self.producers,
            categories: &self.categories,
        };


        let mut aggregated_events = Vec::new();

        for event in self.events {
            let emitted_events = primary_detector.process_event(&event, &read_only_view);

            if let Some(events) = emitted_events {
                aggregated_events.extend(events);
            }
        }

        aggregated_events
    }
}



#[allow(dead_code)]
pub struct EventTranscriptReadOnlyView<'a> {
    tags: &'a HashMap<TagDescriptionId, TagDescription>,
    producers: &'a HashMap<ProducerId, Producer>,
    categories: &'a HashMap<CategoryId, Category>,
}

#[allow(dead_code)]
impl<'a> EventTranscriptReadOnlyView<'a> {
    pub fn tag_by_id(&self, tag_id: TagDescriptionId) -> Option<&TagDescription> {
        self.tags.get(&tag_id)
    }

    pub fn producer_by_id(&self, producer_id: ProducerId) -> Option<&Producer> {
        self.producers.get(&producer_id)
    }

    pub fn category_by_id(&self, category_id: CategoryId) -> Option<&Category> {
        self.categories.get(&category_id)
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum DetectedEvent {
    #[serde(rename = "battery_event")]
    BatteryEvent(BatteryEvent),

    #[serde(rename = "application_event")]
    ApplicationEvent(ApplicationEvent),
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub detected_event: DetectedEvent,
}

impl ProcessedEvent {
    pub fn new_with_random_id<E>(timestamp: DateTime<Utc>, event: E) -> Self
    where
        E: Into<DetectedEvent>,
    {
        let id = Uuid::new_v4();

        Self {
            id,
            timestamp,
            detected_event: event.into(),
        }
    }
}


pub trait EventDetector {
    fn process_event(
        &mut self,
        event: &PersistedEvent,
        context: &EventTranscriptReadOnlyView,
    ) -> Option<Vec<ProcessedEvent>>;
}


pub struct AllDetectors {
    battery: BatteryEventDetector,
    application: ApplicationEventDetector,
}

impl AllDetectors {
    pub fn new() -> Self {
        Self {
            battery: BatteryEventDetector::new(),
            application: ApplicationEventDetector::new(),
        }
    }
}

impl EventDetector for AllDetectors {
    fn process_event(
        &mut self,
        event: &PersistedEvent,
        context: &EventTranscriptReadOnlyView,
    ) -> Option<Vec<ProcessedEvent>> {
        let mut aggregated_events = Vec::new();

        if let Some(emitted_battery_events) = self.battery.process_event(event, context) {
            aggregated_events.extend(emitted_battery_events);
        };

        if let Some(emitted_application_events) = self.application.process_event(event, context) {
            aggregated_events.extend(emitted_application_events);
        };

        if !aggregated_events.is_empty() {
            Some(aggregated_events)
        } else {
            None
        }
    }
}
