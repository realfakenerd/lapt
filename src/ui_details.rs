use ratatui::{prelude::*, widgets::*, style::palette::tailwind};
use crate::pkg::Package;

pub fn render_details(frame: &mut Frame, area: Rect, pkg: Option<&Package>) {
    let block = Block::bordered()
        .title(" Details ")
        .border_style(Style::default().fg(tailwind::SLATE.c700))
        .border_set(symbols::border::ROUNDED);

    let text = if let Some(pkg) = pkg {
        let size_str = if pkg.size > 0 {
            format!("{:.1} MB", pkg.size as f64 / 1024.0 / 1024.0)
        } else {
            "Unknown".to_string()
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(tailwind::SLATE.c500)),
                Span::styled(&pkg.name, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(tailwind::SLATE.c500)),
                Span::raw(&pkg.version),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(tailwind::SLATE.c500)),
                Span::raw(&pkg.status),
            ]),
            Line::from(vec![
                Span::styled("License: ", Style::default().fg(tailwind::SLATE.c500)),
                Span::raw(&pkg.license),
            ]),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(tailwind::SLATE.c500)),
                Span::raw(size_str),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(tailwind::SLATE.c500)),
                Span::styled(&pkg.url, Style::default().fg(tailwind::BLUE.c400)),
            ]),
            Line::from(""),
            Line::from(Span::styled(&pkg.summary, Style::default().add_modifier(Modifier::ITALIC))),
            Line::from(""),
        ];

        if !pkg.description.is_empty() {
            lines.push(Line::from("Description:"));
            lines.push(Line::from(pkg.description.as_str()));
        }

        Text::from(lines)
    } else {
        Text::from("No package selected")
    };

    let p = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(p, area);
}