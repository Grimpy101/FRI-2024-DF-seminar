use chrono::{DateTime, Utc};

pub enum EventCategory {
    USB,
    WebBrowser,
    WiFi,
    Other,
}

pub struct EventPersisted {
    device_id: i64,
    time: Option<DateTime<Utc>>,
    event_name: Option<String>,
    process_name: Option<String>,
    event_category: EventCategory,
}
