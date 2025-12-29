use crate::pkg::Package;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum BackendCommand {
    ListInstalled,
    ListUpgradable,
    #[allow(dead_code)]
    Search(String),
    GetDetails(String),
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
    TaskProgress(u32),
    Error(String),
}

pub struct AptBackend {
}

impl AptBackend {
    pub async fn new() -> Result<Self> {
        Ok(Self { })
    }

    pub async fn handle_command(
        &self,
        cmd: BackendCommand,
        tx: std::sync::mpsc::Sender<BackendEvent>,
    ) -> Result<()> {
        let cmd_context = cmd.clone();

        match &cmd {
            BackendCommand::GetDetails(pkg_id) => {
                let name = pkg_id.split(';').next().unwrap_or("").to_string();
                let tx_clone = tx.clone();
                let id_clone = pkg_id.clone();

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
                            eprintln!("Failed to fetch details for '{}': {}", name, e);
                        }
                    }
                });
            }
            _ => {
                // To be implemented in next tasks
                tx.send(BackendEvent::TaskStarted("Initializing Apt Backend...".into()))?;
                tx.send(BackendEvent::TaskFinished(cmd_context))?;
            }
        }

        Ok(())
    }
}