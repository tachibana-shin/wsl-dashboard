use tokio::task;
use tracing::{info, warn, error};
use crate::wsl::executor::WslCommandExecutor;
use crate::wsl::models::WslCommandResult;

pub async fn start_distro(executor: &WslCommandExecutor, distro_name: &str) -> WslCommandResult<String> {
    // Option 1: First try to start and verify by executing a simple command
    // Use --exec to run a simple echo, which will trigger subsystem startup
    let probe_result = executor.execute_command(&["-d", distro_name, "--", "sh", "-c", "echo 'starting'"]).await;
    
    if !probe_result.success {
        warn!("Failed to start WSL distro {}: {:?}", distro_name, probe_result.error);
        return probe_result;
    }

    // After successful detection, we need to maintain the subsystem's running state.
    // WSL automatically stops the subsystem when there are no active processes or terminal connections.
    // We keep it active by running a non-exiting, windowless 'sleep infinity' process in the background.
    let distro_name_owned = distro_name.to_string();
    task::spawn_blocking(move || {
        info!("Starting background keep-alive process for WSL distro: {}", distro_name_owned);
        
        // Start wsl.exe running sleep infinity with CREATE_NO_WINDOW flag to avoid console window popping up
        let mut cmd = std::process::Command::new("wsl.exe");
        cmd.args(&["-d", &distro_name_owned, "--", "sleep", "infinity"]);
        
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        
        match cmd.spawn() {
            Ok(_child) => {
                info!("Successfully spawned keep-alive process for {}", distro_name_owned);
                // Don't wait for the child process to end
            }
            Err(e) => {
                error!("Failed to spawn keep-alive process for {}: {}", distro_name_owned, e);
            }
        }
    });

    WslCommandResult::success(format!("Distro '{}' started and keep-alive process initiated", distro_name), None)
}

pub async fn stop_distro(executor: &WslCommandExecutor, distro_name: &str) -> WslCommandResult<String> {
    executor.execute_command(&["--terminate", distro_name]).await
}

pub async fn shutdown_wsl(executor: &WslCommandExecutor) -> WslCommandResult<String> {
    executor.execute_command(&["--shutdown"]).await
}

pub async fn delete_distro(executor: &WslCommandExecutor, distro_name: &str) -> WslCommandResult<String> {
    executor.execute_command(&["--unregister", distro_name]).await
}
