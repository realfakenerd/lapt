use crate::app::{App, SelectedTab};
use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{Block, List, ListItem, Paragraph, Tabs, Wrap},
};
use strum::IntoEnumIterator;
use tachyonfx::Duration as FxDuration;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Layout
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]);
    let [header_area, content_area, footer_area] = vertical.areas(area);

    // Header
    render_header(frame, header_area, app);

    // Content
    render_content(frame, content_area, app);

    // Footer
    render_footer(frame, footer_area, app);

    // Popup
    if app.popup.visible {
        render_popup(frame, area, app);
    }

    // Error/Info Notifications
    if !app.notification_queue.is_empty() {
        render_error_popup(frame, area, app);
    }

    // Loading Spinner
    if app.is_loading {
        render_loading(frame, area, app);
    }

    // Effects
    app.effects
        .process_effects(FxDuration::from_millis(16), frame.buffer_mut(), area);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let horizontal = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [tabs_area, title_area] = horizontal.areas(area);

    let titles = SelectedTab::iter().map(|t| {
        let is_selected = t == app.selected_tab;
        let color = if is_selected {
            tailwind::BLUE.c200
        } else {
            tailwind::SLATE.c500
        };
        Line::from(t.to_string()).fg(color)
    });

    let tabs = Tabs::new(titles)
        .highlight_style(Style::default().fg(tailwind::BLUE.c400))
        .select(app.selected_tab as usize)
        .padding("", "")
        .divider(" ");

    frame.render_widget(tabs, tabs_area);

    let title = Line::from(" LAPT - Flux ")
        .style(
            Style::default()
                .fg(tailwind::SLATE.c200)
                .add_modifier(Modifier::BOLD),
        )
        .right_aligned();
    frame.render_widget(title, title_area);
}

fn render_content(frame: &mut Frame, area: Rect, app: &mut App) {
    let layout = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
    let [list_area, detail_area] = layout.areas(area);

    let border_color = if app.is_searching {
        tailwind::AMBER.c500
    } else {
        tailwind::BLUE.c600
    };

    let items: Vec<ListItem> = app
        .filtered_packages
        .iter()
        .map(|pkg| ListItem::new(format!("📦 {}", pkg.name)))
        .collect();

    let title_top = if app.is_searching {
        format!(" Search: {}_ ", app.search_query)
    } else {
        format!(" {} ", app.selected_tab)
    };
    let title_bottom =
        Line::from(format!(" Total: {} ", app.filtered_packages.len())).right_aligned();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title_top(title_top)
                .title_bottom(title_bottom)
                .border_style(Style::default().fg(border_color))
                .border_set(symbols::border::ROUNDED),
        )
        .highlight_style(
            Style::default()
                .bg(tailwind::SLATE.c800)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, list_area, &mut app.list_state);

    // Details
    let selected = app
        .list_state
        .selected()
        .and_then(|i| app.filtered_packages.get(i));
    
    crate::ui_details::render_details(frame, detail_area, selected);
}

fn render_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let keys = [
        ("q", "Quit"),
        ("/", "Search"),
        ("d", "Uninstall"),
        ("r", "Reinstall"),
        ("U", "Upgrade"),
    ];
    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(k, v)| {
            vec![
                Span::styled(
                    format!(" {} ", k),
                    Style::default()
                        .bg(tailwind::SLATE.c800)
                        .fg(tailwind::SLATE.c200),
                ),
                Span::styled(
                    format!(" {} ", v),
                    Style::default()
                        .bg(tailwind::SLATE.c950)
                        .fg(tailwind::SLATE.c500),
                ),
                Span::raw(" "),
            ]
        })
        .collect();
    frame.render_widget(Line::from(spans).centered(), area);
}

fn render_popup(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(area, 60, 20);
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    let block = Block::bordered()
        .title_top(Line::from(app.popup.title.as_str()).centered())
        .border_style(Style::default().fg(tailwind::RED.c500));
    let p = Paragraph::new(format!("\n{}\n\n[y] Yes   [n] No", app.popup.message))
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(p, popup_area);
}

pub fn render_error_popup(frame: &mut Frame, area: Rect, app: &App) {
    if let Some(notification) = app.notification_queue.first() {
        let popup_area = centered_rect(area, 60, 20);
        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let border_color = match notification.kind {
            crate::app::NotificationKind::Error => tailwind::RED.c500,
            crate::app::NotificationKind::Info => tailwind::BLUE.c500,
        };

        let title = match notification.kind {
            crate::app::NotificationKind::Error => " Error ",
            crate::app::NotificationKind::Info => " Info ",
        };

        let block = Block::bordered()
            .title(Line::from(title).centered())
            .border_style(Style::default().fg(border_color))
            .border_set(symbols::border::ROUNDED);

        let p = Paragraph::new(format!("\n{}\n\n[Esc/Enter] Dismiss", notification.message))
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(p, popup_area);
    }
}

fn render_loading(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(area, 40, 10);
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    let block = Block::bordered()
        .title(" Loading ")
        .style(Style::default().fg(tailwind::YELLOW.c400))
        .border_set(symbols::border::ROUNDED);
    let p = Paragraph::new(format!("\n{}", app.loading_msg))
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(p, popup_area);
}

fn centered_rect(r: Rect, px: u16, py: u16) -> Rect {
    let l = Layout::vertical([
        Constraint::Percentage((100 - py) / 2),
        Constraint::Percentage(py),
        Constraint::Percentage((100 - py) / 2),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - px) / 2),
        Constraint::Percentage(px),
        Constraint::Percentage((100 - px) / 2),
    ])
    .split(l[1])[1]
}
