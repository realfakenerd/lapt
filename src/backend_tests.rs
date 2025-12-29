#[cfg(test)]
mod tests {
    use crate::backend::{AptBackend, BackendCommand, BackendEvent};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_backend_initialization() {
        let backend = AptBackend::new().await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_backend_get_details_dispatch() {
        let backend = AptBackend::new().await.unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();
        
        // Use a known package like 'adduser'
        let cmd = BackendCommand::GetDetails("adduser;3.137ubuntu1;all;installed:ubuntu-noble-main".to_string());
        backend.handle_command(cmd, tx).await.unwrap();
        
        let mut found = false;
        for _ in 0..20 {
            if let Ok(event) = rx.try_recv() {
                if let BackendEvent::PackageDetailsFound(pkg) = event {
                    assert_eq!(pkg.name, "adduser");
                    found = true;
                    break;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        assert!(found, "PackageDetailsFound event not received");
    }

    #[tokio::test]
    async fn test_backend_list_installed() {
        let backend = AptBackend::new().await.unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();

        backend.handle_command(BackendCommand::ListInstalled, tx).await.unwrap();

        let mut found_packages = false;
        let mut found_finished = false;

        for _ in 0..20 {
            while let Ok(event) = rx.try_recv() {
                match event {
                    BackendEvent::InstalledPackagesFound(pkgs) => {
                        assert!(!pkgs.is_empty());
                        found_packages = true;
                    }
                    BackendEvent::TaskFinished(BackendCommand::ListInstalled) => {
                        found_finished = true;
                    }
                    _ => {}
                }
            }
            if found_packages && found_finished { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        assert!(found_packages, "InstalledPackagesFound event not received");
        assert!(found_finished, "TaskFinished event not received");
    }

    #[tokio::test]
    async fn test_backend_search() {
        let backend = AptBackend::new().await.unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();

        backend.handle_command(BackendCommand::Search("vim".to_string()), tx).await.unwrap();

        let mut found_packages = false;
        let mut found_finished = false;

        for _ in 0..20 {
            while let Ok(event) = rx.try_recv() {
                match event {
                    BackendEvent::InstalledPackagesFound(pkgs) => {
                        assert!(!pkgs.is_empty());
                        found_packages = true;
                    }
                    BackendEvent::TaskFinished(BackendCommand::Search(_)) => {
                        found_finished = true;
                    }
                    _ => {}
                }
            }
            if found_packages && found_finished { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        assert!(found_packages, "Search results not received");
        assert!(found_finished, "TaskFinished event not received");
    }
}