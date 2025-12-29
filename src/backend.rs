use crate::pkg::Package;
use anyhow::Result;
use futures_util::StreamExt;
use packagekit_zbus::PackageKit::PackageKitProxy;
use packagekit_zbus::Transaction::TransactionProxy;
use packagekit_zbus::zbus::Connection;

const FILTER_NONE: u64 = 0;
const FILTER_INSTALLED: u64 = 1;
const FILTER_NOT_INSTALLED: u64 = 2;
const FILTER_ARCH: u64 = 1 << 2; // 4 - Filtra apenas arquitetura compatível (evita lixo 32bit)
const FILTER_NOT_SOURCE: u64 = 1 << 3; // 8 - Esconde pacotes de código fonte

// --- Constantes de Status (Info Enum) ---
const INFO_INSTALLED: u32 = 1;
const INFO_AVAILABLE: u32 = 2;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum BackendEvent {
    InstalledPackagesFound(Vec<Package>),
    UpgradablePackagesFound(Vec<Package>),
    PackageDetailsFound(Package),
    TaskStarted(String),
    TaskFinished(BackendCommand),
    TaskProgress(u32),
    Error(String),
}

pub struct PackageKitBackend {
    connection: Connection,
}

impl PackageKitBackend {
    pub async fn new() -> Result<Self> {
        let connection = Connection::system().await?;
        Ok(Self { connection })
    }

    pub async fn handle_command(
        &self,
        cmd: BackendCommand,
        tx: std::sync::mpsc::Sender<BackendEvent>,
    ) -> Result<()> {
        if let BackendCommand::GetDetails(pkg_id) = &cmd {
            let name = pkg_id.split(';').next().unwrap_or("").to_string();
            let tx_clone = tx.clone();
            let id_clone = pkg_id.clone();

            tokio::task::spawn_blocking(move || {
                if let Ok(details) = crate::apt::get_package_details(&name) {
                    let mut pkg = Package::from_packagekit(&id_clone, "", "");
                    pkg.update_details(
                        &details.description,
                        &details.license,
                        details.size,
                        &details.url,
                    );
                    let _ = tx_clone.send(BackendEvent::PackageDetailsFound(pkg));
                }
            });
            return Ok(());
        }

        let cmd_context = cmd.clone();
        let pk = PackageKitProxy::new(&self.connection).await?;
        let transaction_path = pk.create_transaction().await?;

        let transaction = TransactionProxy::builder(&self.connection)
            .destination("org.freedesktop.PackageKit")?
            .path(transaction_path)?
            .build()
            .await?;

        // Listeners
        let mut progress_stream = transaction.receive_signal("Percentage").await?;
        let mut package_stream = transaction.receive_signal("Package").await?;
        let mut error_stream = transaction.receive_signal("ErrorCode").await?;
        let mut finished_stream = transaction.receive_signal("Finished").await?;

        // Flags padrão para transações (não confundir com filtros de busca)
        let transaction_flags = 0u64;

        match &cmd {
            BackendCommand::ListInstalled => {
                tx.send(BackendEvent::TaskStarted(
                    "Loading installed packages...".into(),
                ))?;
                // CORREÇÃO DE VELOCIDADE:
                // Usamos INSTALLED | ARCH. Isso força o PK a olhar só o banco local.
                let filter = FILTER_INSTALLED | FILTER_ARCH;
                transaction.get_packages(filter).await?;
            }
            BackendCommand::ListUpgradable => {
                tx.send(BackendEvent::TaskStarted("Checking for updates...".into()))?;
                // GetUpdates usa filtro interno, mas passamos flags de filtro padrão
                transaction.get_updates(FILTER_NONE).await?;
            }
            BackendCommand::Search(query) => {
                tx.send(BackendEvent::TaskStarted(format!(
                    "Searching '{}'...",
                    query
                )))?;
                // Na busca, queremos ver o que NÃO está instalado também
                let filter = FILTER_NOT_INSTALLED | FILTER_ARCH | FILTER_NOT_SOURCE;
                transaction.search_names(filter, &[&query]).await?;
            }
            BackendCommand::Remove(pkg_id) => {
                tx.send(BackendEvent::TaskStarted("Removing...".into()))?;
                transaction
                    .remove_packages(transaction_flags, &[&pkg_id], true, true)
                    .await?;
            }
            BackendCommand::Reinstall(pkg_id) => {
                tx.send(BackendEvent::TaskStarted("Reinstalling...".into()))?;
                transaction
                    .install_packages(transaction_flags, &[&pkg_id])
                    .await?;
            }
            BackendCommand::UpgradeSystem => {
                tx.send(BackendEvent::TaskStarted("System Upgrade...".into()))?;
                transaction.update_packages(transaction_flags, &[]).await?;
            }
            BackendCommand::GetDetails(_) => unreachable!(),
        }

        let mut packages = Vec::new();

        loop {
            tokio::select! {
                Some(msg) = package_stream.next() => {
                    if let Ok((info, id, summary)) = msg.body::<(u32, String, String)>() {

                        let should_add = match cmd {
                            BackendCommand::ListInstalled => {
                                info == INFO_INSTALLED
                            },
                            BackendCommand::ListUpgradable => {
                                info > INFO_AVAILABLE
                            },
                            _ => true
                        };

                        if should_add {
                            let status_str = map_status(info);
                            let pkg = Package::from_packagekit(&id, status_str, &summary);
                            packages.push(pkg);
                        }
                    }
                }

                Some(msg) = error_stream.next() => {
                    if let Ok((_, details)) = msg.body::<(u32, String)>() {
                        tx.send(BackendEvent::Error(details))?;
                        return Ok(());
                    }
                }

                Some(_) = finished_stream.next() => {
                    if !packages.is_empty() {
                        match cmd {
                            BackendCommand::ListInstalled => {
                                tx.send(BackendEvent::InstalledPackagesFound(packages))?;
                            },
                            BackendCommand::ListUpgradable => {
                                tx.send(BackendEvent::UpgradablePackagesFound(packages))?;
                            },
                            BackendCommand::Search(_) => {
                                // Na busca, mandamos como "Installed" temporariamente para popular a lista
                                // ou você pode criar um evento SearchResultsFound
                                tx.send(BackendEvent::InstalledPackagesFound(packages))?;
                            }
                            _ => {}
                        }
                    }
                    tx.send(BackendEvent::TaskFinished(cmd_context))?;
                    break;
                }
            }
        }

        Ok(())
    }
}

// Helper para transformar números mágicos em texto
fn map_status(info: u32) -> &'static str {
    match info {
        1 => "Installed",
        2 => "Available",
        3 => "Trusted",
        4 => "Update",
        5 => "Security",
        6 => "Blocked",
        7 => "Downloading",
        8 => "Cleanup",
        9 => "Obsoleting",
        10 => "Important",
        _ => "Unknown",
    }
}
