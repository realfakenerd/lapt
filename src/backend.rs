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
    UpgradeSystem,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendEvent {
    InstalledPackagesFound(Vec<Package>),
    UpgradablePackagesFound(Vec<Package>),
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
                            let _ = tx_clone.send(BackendEvent::InstalledPackagesFound(pkgs));
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
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = tx_clone.send(BackendEvent::TaskStarted(format!("Installing {}...", name)));
                    match crate::apt::install_package(&name) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!("Installation failed: {}", e)));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::Remove(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = tx_clone.send(BackendEvent::TaskStarted(format!("Removing {}...", name)));
                    match crate::apt::remove_package(&name) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!("Removal failed: {}", e)));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::Reinstall(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = tx_clone.send(BackendEvent::TaskStarted(format!("Reinstalling {}...", name)));
                    match crate::apt::reinstall_package(&name) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = tx_clone.send(BackendEvent::Error(format!("Reinstall failed: {}", e)));
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
            BackendCommand::UpgradeSystem => {
                let tx_clone = tx.clone();
                let cmd_context = cmd_context.clone();
                tokio::task::spawn_blocking(move || {
                    let _ = tx_clone.send(BackendEvent::TaskStarted("Updating repositories...".into()));
                    if let Err(e) = crate::apt::update_repos() {
                        let _ = tx_clone.send(BackendEvent::Error(format!("Update failed: {}", e)));
                    } else {
                        let _ = tx_clone.send(BackendEvent::TaskStarted("Upgrading system...".into()));
                        match crate::apt::upgrade_system() {
                            Ok(_) => {}
                            Err(e) => {
                                let _ = tx_clone.send(BackendEvent::Error(format!("Upgrade failed: {}", e)));
                            }
                        }
                    }
                    let _ = tx_clone.send(BackendEvent::TaskFinished(cmd_context));
                });
            }
        }

        Ok(())
    }
}
