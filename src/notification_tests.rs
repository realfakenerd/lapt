#[cfg(test)]
mod tests {
    use crate::app::{App, Notification};
    use crate::backend::BackendCommand;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_notification_queue() {
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);

        assert!(app.notification_queue.is_empty());

        let notif1 = Notification::error("Error 1".to_string());
        app.push_notification(notif1.clone());

        assert_eq!(app.notification_queue.len(), 1);
        assert_eq!(app.notification_queue[0], notif1);

        let notif2 = Notification::info("Info 1".to_string());
        app.push_notification(notif2.clone());

        assert_eq!(app.notification_queue.len(), 2);
        assert_eq!(app.notification_queue[1], notif2);

        app.dismiss_notification();
        assert_eq!(app.notification_queue.len(), 1);
        assert_eq!(app.notification_queue[0], notif2);
    }

    #[tokio::test]
    async fn test_handle_backend_error_event() {
        use crate::backend::BackendEvent;
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);

        app.update(crate::action::Action::BackendResponse(BackendEvent::Error("Test Error".to_string()))).unwrap();

        assert_eq!(app.notification_queue.len(), 1);
        assert_eq!(app.notification_queue[0].message, "Test Error");
        assert_eq!(app.notification_queue[0].kind, crate::app::NotificationKind::Error);
    }

    #[tokio::test]
    async fn test_action_dismiss_notification() {
        let (tx, _) = mpsc::unbounded_channel::<BackendCommand>();
        let mut app = App::new(tx);
        
        app.push_notification(Notification::info("Test".to_string()));
        assert_eq!(app.notification_queue.len(), 1);

        app.update(crate::action::Action::DismissNotification).unwrap();
        assert_eq!(app.notification_queue.len(), 0);
    }
}