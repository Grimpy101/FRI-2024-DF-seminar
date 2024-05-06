use serde::{Deserialize, Serialize};

use super::{DetectedEvent, EventDetector, EventTranscriptReadOnlyView, ProcessedEvent};
use crate::models::persisted_event::{PersistedEvent, PersistedEventPayload};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum BatteryEventType {
    #[serde(rename = "battery_percentage_change")]
    BatteryPercentageChange { battery_percentage: u8 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatteryEvent {
    pub r#type: BatteryEventType,
}

impl BatteryEvent {
    #[inline]
    pub fn battery_percentage_change(battery_percentage: u8) -> Self {
        Self {
            r#type: BatteryEventType::BatteryPercentageChange { battery_percentage },
        }
    }
}

impl From<BatteryEvent> for DetectedEvent {
    fn from(value: BatteryEvent) -> Self {
        Self::BatteryEvent(value)
    }
}



pub struct BatteryEventDetector {}

impl BatteryEventDetector {
    pub fn new() -> Self {
        Self {}
    }
}


const BATTERY_CHANGE_EVENT_NAME: &str =
    "Microsoft.Windows.Kernel.Power.BatteryChargePercentageChange";

impl EventDetector for BatteryEventDetector {
    fn process_event(
        &mut self,
        event: &PersistedEvent,
        _context: &EventTranscriptReadOnlyView,
    ) -> Option<Vec<ProcessedEvent>> {
        if event.event_name().ne(BATTERY_CHANGE_EVENT_NAME) {
            return None;
        }


        let PersistedEventPayload::Parsed { payload } = event.payload() else {
            return None;
        };

        let Some(payload) = payload.as_object() else {
            return None;
        };

        let Some(data_field) = payload.get("data").and_then(|field| field.as_object()) else {
            return None;
        };

        let Some(remaining_percentage) = data_field
            .get("RemainingPercentage")
            .and_then(|field| field.as_i64())
        else {
            return None;
        };

        let Ok(battery_percentage) = u8::try_from(remaining_percentage) else {
            return None;
        };


        Some(vec![ProcessedEvent::new_with_random_id(
            event.timestamp().to_owned(),
            BatteryEvent::battery_percentage_change(battery_percentage),
        )])
    }
}
