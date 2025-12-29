mod action;
mod app;
mod apt;
mod backend;
mod backend_tests;
mod notification_tests;
mod pkg;
mod pkg_tests;
mod tab_tests;
mod ui;
mod ui_tab_tests;
mod ui_details;
mod ui_error_tests;
mod ui_tests;

use crate::app::App;
use crate::{
    action::Action,
    backend::{AptBackend, BackendCommand, BackendEvent},
};
use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();

    // channels
    let (tx_action, mut rx_action) = mpsc::unbounded_channel::<Action>();
    let (tx_backend_cmd, mut rx_backend_cmd) = mpsc::unbounded_channel::<BackendCommand>();

    // Backend worker
    let tx_action_backend = tx_action.clone();
    tokio::spawn(async move {
        let backend = match AptBackend::new().await {
            Ok(b) => b,
            Err(e) => {
                let _ = tx_action_backend
                    .send(Action::BackendResponse(BackendEvent::Error(e.to_string())));
                return;
            }
        };

        while let Some(cmd) = rx_backend_cmd.recv().await {
            if let Err(e) = backend.handle_command(cmd, tx_action_backend.clone().downgrade_to_backend()).await {
                let _ = tx_action_backend
                    .send(Action::BackendResponse(BackendEvent::Error(e.to_string())));
            }
        }
    });

    // Input loop
    let tx_action_input = tx_action.clone();
    tokio::spawn(async move {
        let tick_rate = Duration::from_millis(16);
        loop {
            if event::poll(tick_rate).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.kind == KeyEventKind::Press {
                        let _ = tx_action_input.send(Action::Key(key));
                    }
                }
            } else {
                let _ = tx_action_input.send(Action::Tick);
            }
        }
    });

    let mut app = App::new(tx_backend_cmd);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Some(action) = rx_action.recv().await {
            let effective_action = match action {
                Action::Key(key) => map_key_to_action(key, &app),
                _ => Some(action),
            };

            if let Some(act) = effective_action {
                app.update(act)?;
            }
        }

        if app.should_quit {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}

// Extension trait to convert Action sender to BackendEvent sender
trait ActionSenderExt {
    fn downgrade_to_backend(self) -> mpsc::UnboundedSender<BackendEvent>;
}

impl ActionSenderExt for mpsc::UnboundedSender<Action> {
    fn downgrade_to_backend(self) -> mpsc::UnboundedSender<BackendEvent> {
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();
        let tx_action = self.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let _ = tx_action.send(Action::BackendResponse(event));
            }
        });
        tx
    }
}

fn map_key_to_action(key: KeyEvent, app: &App) -> Option<Action> {
    if !app.notification_queue.is_empty() {
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Action::DismissNotification),
            _ => None,
        };
    }

    if app.popup.visible {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Enter => Some(Action::ConfirmAction),
            KeyCode::Char('n') | KeyCode::Esc => Some(Action::CancelAction),
            _ => None,
        };
    }

    if app.is_searching {
        return match key.code {
            KeyCode::Esc => Some(Action::ExitSearchMode),
            KeyCode::Enter => {
                if app.selected_tab == crate::app::SelectedTab::Online {
                    Some(Action::TriggerOnlineSearch)
                } else {
                    Some(Action::ExitSearchMode)
                }
            }
            KeyCode::Backspace => Some(Action::DeleteSearchChar),
            KeyCode::Char(c) => Some(Action::UpdateSearchQuery(c)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNext),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPrev),
        KeyCode::Char('h') | KeyCode::Left => Some(Action::SwitchTabPrev),
        KeyCode::Char('l') | KeyCode::Right => Some(Action::SwitchTabNext),
        KeyCode::Char('/') => Some(Action::EnterSearchMode),
        KeyCode::Tab => Some(Action::ToggleFocus),
        KeyCode::Char('i') => Some(Action::RequestInstall),
        KeyCode::Char('d') => Some(Action::RequestUninstall),
        KeyCode::Char('r') => Some(Action::RequestReinstall),
        KeyCode::Char('U') => Some(Action::RequestUpgradeSystem),
        KeyCode::Char('f') => Some(Action::RefreshRepos),
        _ => None,
    }
}