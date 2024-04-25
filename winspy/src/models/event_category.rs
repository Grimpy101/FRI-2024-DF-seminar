use super::{ms_store_event::MSStoreEvent, windows_software_client::WindowsSoftwareClient};

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
