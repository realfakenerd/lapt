#[cfg(test)]
mod tests {
    use crate::app::{App, SelectedTab};
    use crate::backend::BackendCommand;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_tab_expansion() {
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let app = App::new(tx);

        // Verify default tab
        assert_eq!(app.selected_tab, SelectedTab::Installed);

        // Verify next tab (Installed -> Upgradable)
        let tab = SelectedTab::Installed.next();
        assert_eq!(tab, SelectedTab::Upgradable);

        // Verify next tab (Upgradable -> Online)
        let tab = SelectedTab::Upgradable.next();
        assert_eq!(tab, SelectedTab::Online);

        // Verify next tab wrap around (Online -> Installed)
        let tab = tab.next();
        assert_eq!(tab, SelectedTab::Installed);

        // Verify previous tab (Online -> Upgradable)
        assert_eq!(SelectedTab::Online.previous(), SelectedTab::Upgradable);
    }

    #[tokio::test]
    async fn test_online_packages_persistence() {
        use crate::pkg::Package;
        use crate::backend::BackendEvent;
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);

        assert!(app.online_packages.is_empty());
        
        let pkgs = vec![
            Package::from_packagekit("pkg;1.0;all;apt", "Available", "Summary")
        ];
        app.update(crate::action::Action::BackendResponse(BackendEvent::SearchResultsFound(pkgs.clone()))).unwrap();

        assert_eq!(app.online_packages.len(), 1);
        assert_eq!(app.online_packages[0].name, "pkg");

        // Verify it persists after a "tick" or tab switch (simulated)
        app.selected_tab = SelectedTab::Installed;
        app.update(crate::action::Action::Tick).unwrap();
        assert_eq!(app.online_packages.len(), 1);
    }

    #[tokio::test]
    async fn test_trigger_online_search_dispatch() {
        use crate::action::Action;
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);
        app.selected_tab = SelectedTab::Online;
        app.search_query = "vim".to_string();

        // Drain initial commands (ListInstalled, ListUpgradable)
        while let Ok(_) = rx.try_recv() {}

        app.update(Action::TriggerOnlineSearch).unwrap();

        // Check if Search command was sent
        let cmd = rx.try_recv().unwrap();
        if let BackendCommand::Search(q) = cmd {
            assert_eq!(q, "vim");
        } else {
            panic!("Expected Search command, got {:?}", cmd);
        }
    }
}
