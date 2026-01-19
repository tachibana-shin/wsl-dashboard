use crate::wsl::dashboard::WslDashboard;
use crate::config::ConfigManager;
use crate::utils::logging::LoggingSystem;

// Define application state
pub struct AppState {
    pub wsl_dashboard: WslDashboard,
    pub config_manager: ConfigManager,
    pub logging_system: Option<LoggingSystem>,
}

impl AppState {
    pub fn new(config_manager: ConfigManager, logging_system: LoggingSystem) -> Self {
        Self {
            wsl_dashboard: WslDashboard::new(),
            config_manager,
            logging_system: Some(logging_system),
        }
    }
}
