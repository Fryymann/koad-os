#![allow(dead_code, unused_imports, clippy::type_complexity)]

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};
use anyhow::Result;
use crate::KoadDB;

struct KoadApp {
    states: [ListState; 4],
    active_column: usize,
    items_counts: [usize; 4],
}

impl KoadApp {
    fn new() -> Self {
        Self {
            states: [ListState::default(), ListState::default(), ListState::default(), ListState::default()],
            active_column: 0,
            items_counts: [0; 4],
        }
    }

    fn next_col(&mut self) {
        self.active_column = (self.active_column + 1) % 4;
    }

    fn prev_col(&mut self) {
        if self.active_column == 0 { self.active_column = 3; }
        else { self.active_column -= 1; }
    }

    fn scroll_down(&mut self) {
        let i = match self.states[self.active_column].selected() {
            Some(i) => {
                if i >= self.items_counts[self.active_column].saturating_sub(1) { 0 }
                else { i + 1 }
            }
            None => 0,
        };
        self.states[self.active_column].select(Some(i));
    }

    fn scroll_up(&mut self) {
        let i = match self.states[self.active_column].selected() {
            Some(i) => {
                if i == 0 { self.items_counts[self.active_column].saturating_sub(1) }
                else { i - 1 }
            }
            None => 0,
        };
        self.states[self.active_column].select(Some(i));
    }
}

pub fn run_dash(db: &KoadDB) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = KoadApp::new();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Length(3),  // Active Spec Banner
                    Constraint::Min(10),    // Main Content
                    Constraint::Length(3),  // Footer
                ])
                .split(area);

            // 1. Header
            let header = Paragraph::new(" KoadOS Command Center - [v0.2 Stateful] ")
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. Active Spec Banner
            let spec_text = if let Ok(Some((title, _, status, _))) = db.get_spec() {
                format!(" ACTIVE SPEC: {} [{}] ", title.to_uppercase(), status)
            } else {
                " NO ACTIVE SPEC - Use 'koad spec set' to begin ".to_string()
            };
            let spec_banner = Paragraph::new(spec_text)
                .style(Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Green)));
            f.render_widget(spec_banner, chunks[1]);

            // 3. Main Body: 4 Columns
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(chunks[2]);

            // Data Fetching & Rendering
            
            // 0. PLAN
            let mut plan_items: Vec<ListItem> = Vec::new();
            if let Ok(wfs) = db.get_workflows(None, 20) {
                app.items_counts[0] = wfs.len();
                for (_, title, status, project) in wfs {
                    let style = match status.as_deref() {
                        Some("Active") => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                        Some("Pinned") => Style::default().fg(Color::Yellow),
                        _ => Style::default().fg(Color::White),
                    };
                    let prefix = if status.as_deref() == Some("Pinned") { "📌 " } else { "   " };
                    plan_items.push(ListItem::new(format!("{}[{}] {}", prefix, project, title)).style(style));
                }
            }
            render_stateful_column(f, " [PLAN] Pinned & Pending ", plan_items, body_chunks[0], &mut app.states[0], app.active_column == 0);

            // 1. MIND
            let mut mind_items: Vec<ListItem> = Vec::new();
            if let Ok(recent) = db.query("", 15, None) {
                for (_, cat, content, _) in recent {
                    let style = match cat.as_str() {
                        "fact" => Style::default().fg(Color::Cyan),
                        "learning" => Style::default().fg(Color::Green),
                        _ => Style::default().fg(Color::White),
                    };
                    mind_items.push(ListItem::new(format!("[{}] {}", cat.to_uppercase(), content)).style(style));
                }
            }
            if let Ok(ponders) = db.get_ponderings(10) {
                for p in ponders { mind_items.push(ListItem::new(format!("> {}", p)).style(Style::default().fg(Color::Magenta))); }
            }
            app.items_counts[1] = mind_items.len();
            render_stateful_column(f, " [MIND] Knowledge ", mind_items, body_chunks[1], &mut app.states[1], app.active_column == 1);

            // 2. VOICE
            let mut voice_items: Vec<ListItem> = Vec::new();
            if let Ok(notes) = db.get_notes(10) {
                for (_, content, _) in notes { voice_items.push(ListItem::new(format!("(Note) {}", content))); }
            }
            if let Ok(brainstorms) = db.get_recent_brainstorms(10) {
                for (content, cat, _) in brainstorms {
                    let style = if cat == "rant" { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Yellow) };
                    voice_items.push(ListItem::new(format!("[{}] {}", cat.to_uppercase(), content)).style(style));
                }
            }
            app.items_counts[2] = voice_items.len();
            render_stateful_column(f, " [VOICE] Notes & Ideas ", voice_items, body_chunks[2], &mut app.states[2], app.active_column == 2);

            // 3. ACTIVITY
            let mut activity_items: Vec<ListItem> = Vec::new();
            if let Ok(execs) = db.get_recent_executions(12) {
                for (cmd, args, status) in execs {
                    let style = if status == "success" { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) };
                    activity_items.push(ListItem::new(format!("[CLI] {} {}", cmd, args)).style(style));
                }
            }
            if let Ok(deltas) = db.get_recent_deltas(30) {
                for (path, event, _) in deltas {
                    let name = std::path::Path::new(&path).file_name().unwrap_or_default().to_string_lossy();
                    activity_items.push(ListItem::new(format!("[BOOST] {}: {}", event, name)).style(Style::default().fg(Color::Blue)));
                }
            }
            app.items_counts[3] = activity_items.len();
            render_stateful_column(f, " [ACTIVITY] Pulse ", activity_items, body_chunks[3], &mut app.states[3], app.active_column == 3);

            // 4. Footer
            let footer = Paragraph::new(" [q] Exit | [TAB/h/l] Switch Col | [j/k] Scroll | [Auto-refresh: 250ms] ")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => app.next_col(),
                    KeyCode::Left | KeyCode::Char('h') => app.prev_col(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn render_stateful_column(
    f: &mut ratatui::Frame,
    title: &str,
    items: Vec<ListItem>,
    area: Rect,
    state: &mut ListState,
    is_active: bool,
) {
    let border_style = if is_active {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title).border_style(border_style))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, state);
}
