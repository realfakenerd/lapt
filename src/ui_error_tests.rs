#[cfg(test)]
mod tests {
    use crate::app::{App, Notification};
    use crate::ui::render_error_popup;
    use ratatui::{backend::TestBackend, Terminal, buffer::Buffer, layout::Rect};
    use std::sync::mpsc;
    use crate::backend::BackendCommand;

    #[test]
    fn test_render_error_popup() {
        let (tx, _) = mpsc::channel::<BackendCommand>();
        let mut app = App::new(tx);
        
        let notif = Notification::error("Critical Failure".to_string());
        app.push_notification(notif);

        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let area = Rect::new(0, 0, 100, 50);
            render_error_popup(f, area, &app);
        }).unwrap();

        let buffer_string = format!("{:?}", terminal.backend().buffer());
        assert!(buffer_string.contains("Critical Failure"));
        assert!(buffer_string.contains("Error"));
    }
}
