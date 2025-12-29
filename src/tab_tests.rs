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
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);

        assert!(app.online_packages.is_empty());
        
        // Simulate adding a package to online results
        // app.online_packages.push(...);
    }
}
