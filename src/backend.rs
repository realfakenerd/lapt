use crate::pkg::Package;
use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, PartialEq)]
pub enum BackendCommand {
    ListInstalled,
    ListUpgradable,
    #[allow(dead_code)]
    Search(String),
    GetDetails(String),
    Install(String),
    Remove(String),
    Reinstall(String),
    RefreshRepos,
    UpgradeSystem,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendEvent {
    InstalledPackagesFound(Vec<Package>),
    UpgradablePackagesFound(Vec<Package>),
    SearchResultsFound(Vec<Package>),
    PackageDetailsFound(Package),
    TaskStarted(String),
    TaskFinished(BackendCommand),
    Error(String),
}

pub struct AptBackend {}

impl AptBackend {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    async fn stream_command_output(
        &self,
        mut child: tokio::process::Child,
        cmd_context: BackendCommand,
        tx: UnboundedSender<BackendEvent>,
    ) {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Some(msg) = parse_apt_signal(&line) {
                    let _ = tx_clone.send(BackendEvent::TaskStarted(msg));
                }
            }
        });

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Some(msg) = parse_apt_signal(&line) {
                    let _ = tx_clone.send(BackendEvent::TaskStarted(format!("Error: {}", msg)));
                }
            }
        });

        match child.wait().await {
            Ok(status) => {
                if !status.success() {
                    let _ = tx.send(BackendEvent::Error(format!(
                        "Command failed with status: {}",
                        status
                    )));
                }
            }
            Err(e) => {
                let _ = tx.send(BackendEvent::Error(format!("Wait failed: {}", e)));
            }
        }
        let _ = tx.send(BackendEvent::TaskFinished(cmd_context));
    }

    pub async fn handle_command(
        &self,
        cmd: BackendCommand,
        tx: UnboundedSender<BackendEvent>,
    ) -> Result<()> {
        let cmd_context = cmd.clone();

        match &cmd {
            BackendCommand::ListInstalled => {
                let _ = tx.send(BackendEvent::TaskStarted(
                    "Listing installed packages...".into(),
                ));
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::task::spawn_blocking(move || {
                    match crate::apt::list_installed() {
                        Ok(pkgs) => {
                            let _ = tx_clone.send(BackendEvent::InstalledPackagesFound(pkgs));
                        }
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!(
                                "Failed to list installed: {}",
                                e
                            )));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::ListUpgradable => {
                let _ = tx.send(BackendEvent::TaskStarted("Checking for updates...".into()));
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::task::spawn_blocking(move || {
                    match crate::apt::list_upgradable() {
                        Ok(pkgs) => {
                            let _ = tx_clone.send(BackendEvent::UpgradablePackagesFound(pkgs));
                        }
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!(
                                "Failed to list upgradable: {}",
                                e
                            )));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::Search(query) => {
                let _ = tx.send(BackendEvent::TaskStarted(format!("Searching '{}'...", query)));
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                let query = query.clone();
                tokio::task::spawn_blocking(move || {
                    match crate::apt::search_packages(&query) {
                        Ok(pkgs) => {
                            let _ = tx_clone.send(BackendEvent::SearchResultsFound(pkgs));
                        }
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!("Search failed: {}", e)));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::GetDetails(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let tx_clone = tx.clone();
                let id_clone = pkg_id.clone();
                let cmd_context = cmd_context.clone();

                tokio::task::spawn_blocking(move || {
                    match crate::apt::get_package_details(&name) {
                        Ok(details) => {
                            let mut pkg = Package::from_packagekit(&id_clone, "", "");
                            pkg.update_details(
                                &details.description,
                                &details.license,
                                details.size,
                                &details.url,
                            );
                            let _ = tx_clone.send(BackendEvent::PackageDetailsFound(pkg));
                        }
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!(
                                "Failed to fetch details for '{}': {}",
                                name, e
                            )));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::Install(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let child = crate::apt::spawn_install(&name)?;
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::spawn(async move {
                    let backend = AptBackend {};
                    backend.stream_command_output(child, cmd_context, tx_clone).await;
                });
            }
            BackendCommand::Remove(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let child = crate::apt::spawn_remove(&name)?;
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::spawn(async move {
                    let backend = AptBackend {};
                    backend.stream_command_output(child, cmd_context, tx_clone).await;
                });
            }
            BackendCommand::Reinstall(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let child = crate::apt::spawn_reinstall(&name)?;
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::spawn(async move {
                    let backend = AptBackend {};
                    backend.stream_command_output(child, cmd_context, tx_clone).await;
                });
            }
            BackendCommand::RefreshRepos => {
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::spawn(async move {
                    let backend = AptBackend {};
                    let _ = tx_clone.send(BackendEvent::TaskStarted("Refreshing repositories...".into()));
                    match crate::apt::spawn_update() {
                        Ok(child) => {
                            backend.stream_command_output(child, cmd_context, tx_clone).await;
                        }
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!("Refresh failed: {}", e)));
                            let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                        }
                    }
                });
            }
            BackendCommand::UpgradeSystem => {
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::spawn(async move {
                    let backend = AptBackend {};
                    let _ = tx_clone.send(BackendEvent::TaskStarted("Updating repositories...".into()));
                    match crate::apt::spawn_update() {
                        Ok(mut child) => {
                            let _ = child.wait().await;
                            let _ = tx_clone.send(BackendEvent::TaskStarted("Upgrading system...".into()));
                            match crate::apt::spawn_upgrade() {
                                Ok(child) => {
                                    backend.stream_command_output(child, cmd_context, tx_clone).await;
                                }
                                Err(e) => {
                                    let _ = tx_clone.send(BackendEvent::Error(format!("Upgrade failed: {}", e)));
                                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!("Update failed: {}", e)));
                            let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                        }
                    }
                });
            }
        }

        Ok(())
    }
}

pub fn parse_apt_signal(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if line.starts_with("Unpacking")
        || line.starts_with("Preparing to unpack")
        || line.starts_with("Setting up")
        || line.starts_with("Removing")
        || line.starts_with("Processing triggers")
        || line.starts_with("Get:")
        || line.starts_with("Hit:")
        || line.starts_with("Err:")
    {
        Some(line.to_string())
    } else {
        None
    }
}
