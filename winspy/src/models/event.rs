use chrono::{DateTime, Utc};
use miette::{IntoDiagnostic, Result};
use serde_json::Value;

use super::{
    event_category::EventCategory, ms_store_event::MSStoreEvent,
    windows_software_client::WindowsSoftwareClient,
};

#[derive(Debug)]
pub struct Event {
    device_id: Option<String>,
    time: Option<DateTime<Utc>>,
    event_name: Option<String>,
    process_name: Option<String>,
    json_payload: Option<Value>,
    event_category: EventCategory,
    raw_payload: Option<String>,
}

impl Event {
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
            event_category,
            json_payload: json_result,
            raw_payload: payload,
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

    pub fn time(&self) -> Option<DateTime<Utc>> {
        self.time
    }
}
