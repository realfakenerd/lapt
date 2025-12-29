use crate::action::Action;
use crate::backend::{BackendCommand, BackendEvent};
use crate::pkg::Package;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::widgets::ListState;
use std::sync::mpsc::Sender;
use strum::{Display, EnumIter, FromRepr};
use tachyonfx::{Duration as FxDuration, EffectManager, Interpolation, fx};

// Enums auxiliares (Tab, Panel, Popup, etc)
#[derive(Default, PartialEq, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    #[strum(to_string = "Installed")]
    Installed,
    #[strum(to_string = "Upgradable")]
    Upgradable,
}

impl SelectedTab {
    pub fn next(self) -> Self {
        let i = self as usize;
        Self::from_repr(i + 1).unwrap_or(self)
    }
    pub fn previous(self) -> Self {
        let i = self as usize;
        Self::from_repr(i.saturating_sub(1)).unwrap_or(self)
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Panel {
    #[default]
    PackageList,
    Details,
}

#[derive(Default)]
pub struct Popup {
    pub visible: bool,
    pub title: String,
    pub message: String,
    pub command_to_confirm: Option<BackendCommand>, // Guardamos o comando pronto
}

#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub message: String,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationKind {
    Info,
    Error,
}

impl Notification {
    pub fn info(message: String) -> Self {
        Self {
            message,
            kind: NotificationKind::Info,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            message,
            kind: NotificationKind::Error,
        }
    }
}

// O ESTADO DA APLICAÇÃO
pub struct App {
    pub should_quit: bool,
    pub effects: EffectManager<()>,

    // Dados
    pub installed_packages: Vec<Package>,
    pub upgradable_packages: Vec<Package>,
    pub filtered_packages: Vec<Package>,

    // UI State
    pub selected_tab: SelectedTab,
    pub active_panel: Panel,
    pub list_state: ListState,
    pub popup: Popup,
    pub notification_queue: Vec<Notification>,

    // Search
    pub search_query: String,
    pub is_searching: bool,
    pub matcher: SkimMatcherV2,

    // Loading
    pub is_loading: bool,
    pub loading_msg: String,

    // Canal para mandar coisas pro Backend
    pub tx_cmd: Sender<BackendCommand>,
}

impl App {
    pub fn new(tx_cmd: Sender<BackendCommand>) -> Self {
        let mut app = Self {
            should_quit: false,
            effects: EffectManager::default(),
            installed_packages: vec![],
            upgradable_packages: vec![],
            filtered_packages: vec![],
            selected_tab: SelectedTab::Installed,
            active_panel: Panel::PackageList,
            list_state: ListState::default(),
            popup: Popup::default(),
            notification_queue: Vec::new(),
            search_query: String::new(),
            is_searching: false,
            matcher: SkimMatcherV2::default(),
            is_loading: false,
            loading_msg: String::new(),
            tx_cmd,
        };
        // Inicializa buscando dados
        app.dispatch(BackendCommand::ListInstalled);
        app.dispatch(BackendCommand::ListUpgradable);
        app
    }

    // Helper para enviar comandos pro backend
    fn dispatch(&mut self, cmd: BackendCommand) {
        let _ = self.tx_cmd.send(cmd);
        self.is_loading = true;
        self.loading_msg = "Processing...".into();
    }

    pub fn push_notification(&mut self, notification: Notification) {
        self.notification_queue.push(notification);
    }

    pub fn dismiss_notification(&mut self) {
        if !self.notification_queue.is_empty() {
            self.notification_queue.remove(0);
        }
    }

    // O REDUCER: (State, Action) -> New State
    pub fn update(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Tick => {} // Animações são tratadas no draw/loop
            Action::Quit => self.should_quit = true,

            // --- Navegação ---
            Action::SelectNext => self.next_item(),
            Action::SelectPrev => self.prev_item(),
            Action::SwitchTabNext => {
                self.selected_tab = self.selected_tab.next();
                self.perform_search();
                self.list_state.select(Some(0));
                self.trigger_tab_effect();
            }
            Action::SwitchTabPrev => {
                self.selected_tab = self.selected_tab.previous();
                self.perform_search();
                self.list_state.select(Some(0));
                self.trigger_tab_effect();
            }
            Action::ToggleFocus => {
                self.active_panel = match self.active_panel {
                    Panel::PackageList => Panel::Details,
                    Panel::Details => Panel::PackageList,
                };
                self.effects.add_effect(fx::coalesce((
                    FxDuration::from_millis(200),
                    Interpolation::QuadOut,
                )));
            }

            // --- Busca ---
            Action::EnterSearchMode => {
                self.is_searching = true;
                self.search_query.clear();
                self.perform_search();
                self.active_panel = Panel::PackageList;
            }
            Action::ExitSearchMode => {
                self.is_searching = false;
                self.effects.add_effect(fx::fade_to(
                    ratatui::style::Color::Reset,
                    ratatui::style::Color::Reset,
                    (FxDuration::from_millis(150), Interpolation::SineOut),
                ));
            }
            Action::UpdateSearchQuery(c) => {
                self.search_query.push(c);
                self.perform_search();
            }
            Action::DeleteSearchChar => {
                self.search_query.pop();
                self.perform_search();
            }

            // --- Ações de Negócio (Popups) ---
            Action::RequestUninstall => {
                if let Some(pkg) = self.get_selected_pkg() {
                    self.open_popup(
                        "Confirm Uninstall",
                        &format!("Remove {}?", pkg.name),
                        Some(BackendCommand::Remove(pkg.id.clone())),
                    );
                }
            }
            Action::RequestReinstall => {
                if let Some(pkg) = self.get_selected_pkg() {
                    self.open_popup(
                        "Confirm Reinstall",
                        &format!("Reinstall {}?", pkg.name),
                        Some(BackendCommand::Reinstall(pkg.id.clone())),
                    );
                }
            }
            Action::RequestUpgradeSystem => {
                self.open_popup(
                    "System Upgrade",
                    "Update full system?",
                    Some(BackendCommand::UpgradeSystem),
                );
            }
            Action::ConfirmAction => {
                if let Some(cmd) = self.popup.command_to_confirm.take() {
                    self.dispatch(cmd);
                }
                self.popup.visible = false;
            }
            Action::CancelAction => {
                self.popup.visible = false;
                self.popup.command_to_confirm = None;
            }

            Action::BackendResponse(event) => self.handle_backend_event(event),

            _ => {}
        }
        Ok(())
    }

    fn handle_backend_event(&mut self, event: BackendEvent) {
        match event {
            BackendEvent::TaskStarted(msg) => {
                self.is_loading = true;
                self.loading_msg = msg;
            }
            BackendEvent::InstalledPackagesFound(pkgs) => {
                self.installed_packages = pkgs;
                if self.selected_tab == SelectedTab::Installed {
                    self.perform_search();
                }
            }
            BackendEvent::UpgradablePackagesFound(pkgs) => {
                self.upgradable_packages = pkgs;
                if self.selected_tab == SelectedTab::Upgradable {
                    self.perform_search();
                }
            }
            BackendEvent::PackageDetailsFound(details) => {
                // Update in all lists
                for p in self.installed_packages.iter_mut() {
                    if p.id == details.id {
                        p.update_details(&details.description, &details.license, details.size, &details.url);
                    }
                }
                for p in self.upgradable_packages.iter_mut() {
                    if p.id == details.id {
                        p.update_details(&details.description, &details.license, details.size, &details.url);
                    }
                }
                for p in self.filtered_packages.iter_mut() {
                    if p.id == details.id {
                        p.update_details(&details.description, &details.license, details.size, &details.url);
                    }
                }
            }
            BackendEvent::TaskFinished(cmd) => {
                self.is_loading = false;
                match cmd {
                    BackendCommand::Remove(_)
                    | BackendCommand::Reinstall(_)
                    | BackendCommand::UpgradeSystem => {
                        self.dispatch(BackendCommand::ListInstalled);
                        self.dispatch(BackendCommand::ListUpgradable);
                    }
                    _ => {}
                }
            }
            BackendEvent::Error(err) => {
                self.is_loading = false;
                self.open_popup("Error", &err, None);
            }
            _ => {}
        }
    }

    fn perform_search(&mut self) {
        let source = match self.selected_tab {
            SelectedTab::Installed => &self.installed_packages,
            SelectedTab::Upgradable => &self.upgradable_packages,
        };
        if self.search_query.is_empty() {
            self.filtered_packages = source.clone();
        } else {
            let mut matches: Vec<(&Package, i64)> = source
                .iter()
                .filter_map(|pkg| {
                    self.matcher
                        .fuzzy_match(&pkg.name, &self.search_query)
                        .map(|score| (pkg, score))
                })
                .collect();
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_packages = matches.into_iter().map(|(p, _)| p.clone()).collect();
        }
        self.list_state.select(Some(0));
        self.request_details_for_selected();
    }

    fn next_item(&mut self) {
        if self.filtered_packages.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_packages.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.request_details_for_selected();
    }

    fn prev_item(&mut self) {
        if self.filtered_packages.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_packages.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.request_details_for_selected();
    }

    fn request_details_for_selected(&mut self) {
        if let Some(pkg) = self.get_selected_pkg() {
            let _ = self.tx_cmd.send(BackendCommand::GetDetails(pkg.id.clone()));
        }
    }

    fn get_selected_pkg(&self) -> Option<&Package> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_packages.get(i))
    }

    fn open_popup(&mut self, title: &str, msg: &str, cmd: Option<BackendCommand>) {
        self.popup.visible = true;
        self.popup.title = title.into();
        self.popup.message = msg.into();
        self.popup.command_to_confirm = cmd;
    }

    fn trigger_tab_effect(&mut self) {
        self.effects
            .add_effect(fx::coalesce(FxDuration::from_millis(300)));
    }
}
