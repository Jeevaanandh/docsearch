use crate::App;
use crate::open_file::open;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io;

pub fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(5),
                ])
                .split(f.size());

            // Title
            let title = Paragraph::new("Top Results")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL).title("DocSearch"));
            f.render_widget(title, chunks[0]);

            // Menu items
            let items: Vec<ListItem> = app
                .options
                .iter()
                .enumerate()
                .map(|(i, option)| {
                    let style = if i == app.selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if i == app.selected { "⦿ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, option)).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Files"))
                .style(Style::default().fg(Color::White));
            f.render_widget(list, chunks[1]);

            // Footer with controls and selection info
            let footer = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        "↑/↓",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Navigate | "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Select | "),
                    Span::styled(
                        "Q",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": Quit"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Selected: ", Style::default().fg(Color::Cyan)),
                    Span::raw(app.get_selected_option()),
                ]),
            ])
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::White));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        if app.get_selected_option().contains("Exit") {
                            return Ok(());
                        }
                        // Here you could handle other menu selections
                        open(app.get_selected_option());
                    }
                    _ => {}
                }
            }
        }
    }
}
