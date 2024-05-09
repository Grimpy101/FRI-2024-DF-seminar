// pub enum USBEvent {
//     Attached(PersistedEvent),
//     Detached(PersistedEvent),
// }

// enum USBState {
//     DeviceGuidGenerated,
//     DeviceStarted,
//     DeviceDescriptorData,
//     DeviceAccessAlignment,
//     DeviceSeekPenaltyProperty,
//     BasicVolumeDeviceCreation,
//     DiskDiscovery,
//     FatMount,
//     NTFSMount,
//     NTFSVolumeInfo,
//     NTFSVolumeInfoSizes,
//     VolumeAttach,
//     InitInstance,
//     UsbDiskArrival,
//     SdCardStatus,
// }

// pub fn detect_usb_events(events: &Vec<PersistedEvent>) -> Vec<USBEvent> {
//     let mut usb_events = Vec::new();

//     for event in events.iter() {}

//     usb_events
// }

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    extract_value_from_json_object, models::persisted_event::PersistedEventPayload,
    require_json_object, require_json_object_value,
};

use super::{DetectedEvent, EventDetector, ProcessedEvent};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct USBAddedEvent {
    device_id: String,
    service: String,
    description: String,
}

impl USBAddedEvent {
    pub fn new(device_id: String, service: String, description: String) -> Self {
        Self {
            device_id,
            service,
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum USBEvent {
    #[serde(rename = "added")]
    Added(USBAddedEvent),
}

impl From<USBEvent> for DetectedEvent {
    fn from(value: USBEvent) -> Self {
        Self::UsbEvent(value)
    }
}

pub struct USBEventDetector;

impl USBEventDetector {
    pub fn new() -> Self {
        Self
    }
}

impl EventDetector for USBEventDetector {
    fn process_event(
        &mut self,
        event: &crate::models::persisted_event::PersistedEvent,
        context: &super::EventTranscriptReadOnlyView,
    ) -> Option<Vec<super::ProcessedEvent>> {
        if !event.event_name_contains("Microsoft.Windows.Inventory.Core.InventoryDevicePnpAdd") {
            return None;
        }

        let PersistedEventPayload::Parsed { payload } = event.payload() else {
            return None;
        };

        let payload_object = require_json_object!(payload);
        let data = extract_value_from_json_object!(payload_object, "data" => object);
        let class = extract_value_from_json_object!(data, "Class" => str);
        if !class.contains("usb") {
            warn!("Device is of type {}", class);
            return None;
        }

        let description = extract_value_from_json_object!(data, "Description" => str);
        let service = extract_value_from_json_object!(data, "Service" => str);
        let device_id = extract_value_from_json_object!(data, "MatchingID" => str);

        let usb_event = USBAddedEvent::new(
            device_id.to_string(),
            service.to_string(),
            description.to_string(),
        );

        Some(vec![ProcessedEvent::new_with_random_id(
            event.timestamp().to_utc(),
            USBEvent::Added(usb_event),
        )])
    }
}
