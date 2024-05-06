use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TabEvent {
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EdgeInstance {
    opened_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    tabs: Vec<TabEvent>,
    session_guid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EdgeDefaultSearchEngineChange {
    changed_at: DateTime<Utc>,
    changed_to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EdgeDefaultSearchEngine {
    history: Vec<EdgeDefaultSearchEngineChange>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EdgeHomePageChange {
    changed_at: DateTime<Utc>,
    changed_to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EdgeHomePage {
    history: Vec<EdgeHomePageChange>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EdgeEvent {
    EdgeInstance(EdgeInstance),
    DefaultSearchEngine(EdgeDefaultSearchEngine),
    HomePage(EdgeHomePage),
}
