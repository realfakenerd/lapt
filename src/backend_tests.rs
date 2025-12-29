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
                    BackendEvent::SearchResultsFound(pkgs) => {
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

    #[tokio::test]
    async fn test_backend_upgrade_system_dispatch() {
        let backend = AptBackend::new().await.unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();

        backend.handle_command(BackendCommand::UpgradeSystem, tx).await.unwrap();

        let mut found_started = false;
        let mut found_finished = false;

        for _ in 0..50 { // System upgrade might take longer even for just update
            while let Ok(event) = rx.try_recv() {
                match event {
                    BackendEvent::TaskStarted(_) => {
                        found_started = true;
                    }
                    BackendEvent::TaskFinished(BackendCommand::UpgradeSystem) => {
                        found_finished = true;
                    }
                    _ => {}
                }
            }
            if found_started && found_finished { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        assert!(found_started, "TaskStarted event not received");
        assert!(found_finished, "TaskFinished event not received");
    }

    #[tokio::test]
    async fn test_backend_refresh_repos_dispatch() {
        let backend = AptBackend::new().await.unwrap();
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();

        backend.handle_command(BackendCommand::RefreshRepos, tx).await.unwrap();

        let mut found_started = false;
        let mut found_finished = false;

        for _ in 0..20 {
            while let Ok(event) = rx.try_recv() {
                match event {
                    BackendEvent::TaskStarted(_) => {
                        found_started = true;
                    }
                    BackendEvent::TaskFinished(BackendCommand::RefreshRepos) => {
                        found_finished = true;
                    }
                    _ => {}
                }
            }
            if found_started && found_finished { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        assert!(found_started, "TaskStarted event not received");
        assert!(found_finished, "TaskFinished event not received");
    }

    #[test]
    fn test_parse_apt_signal() {
        use crate::backend::parse_apt_signal;
        
        assert_eq!(parse_apt_signal("Unpacking vim (2:9.1)..."), Some("Unpacking vim (2:9.1)...".to_string()));
        assert_eq!(parse_apt_signal("Setting up vim..."), Some("Setting up vim...".to_string()));
        assert_eq!(parse_apt_signal("Hit:1 http://archive.ubuntu.com/ubuntu noble InRelease"), Some("Hit:1 http://archive.ubuntu.com/ubuntu noble InRelease".to_string()));
        assert_eq!(parse_apt_signal("Not interesting output"), None);
    }
}