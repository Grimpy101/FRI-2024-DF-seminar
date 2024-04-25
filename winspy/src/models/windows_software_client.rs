#[derive(Debug)]
pub enum WindowsSoftwareClient {
    CheckForUpdates,
    UpdateDetected,
    Installing,
    Downloading,
    Other,
}

impl WindowsSoftwareClient {
    pub fn display_name(&self) -> String {
        match self {
            WindowsSoftwareClient::CheckForUpdates => "Checking for Updates",
            WindowsSoftwareClient::UpdateDetected => "Detected Update",
            WindowsSoftwareClient::Installing => "Installing",
            WindowsSoftwareClient::Downloading => "Downloading",
            WindowsSoftwareClient::Other => "Unknown Event",
        }
        .to_string()
    }
}
