#[cfg(test)]
mod tests {
    use crate::app::{App, SelectedTab};
    use crate::backend::BackendCommand;
    use crate::ui;
    use ratatui::{backend::TestBackend, Terminal, layout::Rect};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_ui_online_tab_rendering() {
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);
        app.selected_tab = SelectedTab::Online;

        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            ui::draw(f, &mut app);
        }).unwrap();

        let buffer_string = format!("{:?}", terminal.backend().buffer());
        // Verify Online tab is selected/present
        assert!(buffer_string.contains("Online"));
    }

    #[tokio::test]
    async fn test_ui_online_package_list_rendering() {
        use crate::pkg::Package;
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);
        app.selected_tab = SelectedTab::Online;
        app.online_packages = vec![
            Package::from_packagekit("online-pkg;1.0;all;apt", "Available", "A test online package")
        ];
        // Trigger search to populate filtered_packages
        app.update(crate::action::Action::Tick).unwrap(); // Tick doesn't trigger search
        // Manually trigger search for the test
        // Actually perform_search is private but we can trigger it via update or just call it if we were in the same module
        // Let's use a dummy search update
        app.update(crate::action::Action::EnterSearchMode).unwrap();
        app.update(crate::action::Action::ExitSearchMode).unwrap();

        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            ui::draw(f, &mut app);
        }).unwrap();

        let buffer_string = format!("{:?}", terminal.backend().buffer());
        assert!(buffer_string.contains("online-pkg"));
    }
}
