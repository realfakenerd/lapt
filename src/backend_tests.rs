#[cfg(test)]
mod tests {
    use crate::backend::{AptBackend, BackendCommand, BackendEvent};
    use std::sync::mpsc;

    #[tokio::test]
    async fn test_backend_initialization() {
        let backend = AptBackend::new().await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_backend_get_details_dispatch() {
        let backend = AptBackend::new().await.unwrap();
        let (tx, rx) = mpsc::channel::<BackendEvent>();
        
        // Use a known package like 'adduser'
        let cmd = BackendCommand::GetDetails("adduser;3.137ubuntu1;all;installed:ubuntu-noble-main".to_string());
        backend.handle_command(cmd, tx).await.unwrap();
        
        // We expect a PackageDetailsFound event eventually
        // Since it's in a spawn_blocking, we might need to wait or loop
        let mut found = false;
        for _ in 0..10 {
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
}
