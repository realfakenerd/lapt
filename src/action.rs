use crate::backend::BackendEvent;
use ratatui::crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum Action {
    // --- Eventos de Sistema ---
    Tick,
    Key(KeyEvent),
    Quit,

    // --- Ações de UI (User Intent) ---
    SelectNext,
    SelectPrev,
    SwitchTabNext,
    SwitchTabPrev,
    ToggleFocus, // Alternar entre Lista e Detalhes
    EnterSearchMode,
    ExitSearchMode,
    UpdateSearchQuery(char),
    DeleteSearchChar,

    // --- Comandos de Negócio (Disparados pelo usuário) ---
    ConfirmAction, // Enter/y no popup
    CancelAction,  // Esc/n no popup
    RequestUninstall,
    RequestReinstall,
    RequestUpgradeSystem,

    // --- Eventos do Backend (Respostas) ---
    // O Backend manda BackendEvent, que embrulhamos aqui
    BackendResponse(BackendEvent),
}
