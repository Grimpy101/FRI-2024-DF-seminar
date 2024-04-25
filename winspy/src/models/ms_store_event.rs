#[derive(Debug)]
pub enum MSStoreEvent {
    Launching,
    Launched,
    Activating,
    Activated,
    AuthRequest,
    BeginPurchase,
    FinishPurchase,
    Other,
}

impl MSStoreEvent {
    pub fn display_name(&self) -> String {
        match self {
            MSStoreEvent::Launching => "Launching",
            MSStoreEvent::Launched => "Launched",
            MSStoreEvent::Activating => "Activating",
            MSStoreEvent::Activated => "Activated",
            MSStoreEvent::AuthRequest => "Authentication Request",
            MSStoreEvent::BeginPurchase => "Purchase Begins",
            MSStoreEvent::FinishPurchase => "Purchase Finished",
            MSStoreEvent::Other => "Unknown Event",
        }
        .to_string()
    }
}
