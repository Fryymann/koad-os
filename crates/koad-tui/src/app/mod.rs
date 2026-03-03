use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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
use koad_proto::kernel::kernel_service_client::KernelServiceClient;
use koad_proto::kernel::Empty;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use tokio::net::UnixStream;
use hyper_util::rt::tokio::TokioIo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub uptime: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMapItem {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub branch: String,
    pub health: String,
}

pub struct KoadApp {
    states: [ListState; 5],
    active_column: usize,
    items_counts: [usize; 5],
    pub terminal_logs: Vec<String>,
    pub stats: Option<SystemStats>,
    pub projects: Vec<ProjectMapItem>,
}

impl KoadApp {
    pub fn new() -> Self {
        Self {
            states: [ListState::default(), ListState::default(), ListState::default(), ListState::default(), ListState::default()],
            active_column: 0,
            items_counts: [0; 5],
            terminal_logs: Vec::new(),
            stats: None,
            projects: Vec::new(),
        }
    }

    pub fn next_col(&mut self) {
        self.active_column = (self.active_column + 1) % 5;
    }

    pub fn prev_col(&mut self) {
        if self.active_column == 0 { self.active_column = 4; }
        else { self.active_column -= 1; }
    }

    pub fn scroll_down(&mut self) {
        let i = match self.states[self.active_column].selected() {
            Some(i) => {
                if i >= self.items_counts[self.active_column].saturating_sub(1) { 0 }
                else { i + 1 }
            }
            None => 0,
        };
        self.states[self.active_column].select(Some(i));
    }

    pub fn scroll_up(&mut self) {
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

pub async fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 1. Connect to Kernel
    let socket_path = "/home/ideans/.koad-os/kspine.sock";
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(move |_: Uri| {
            let path = socket_path.to_string();
            async move { 
                let stream = UnixStream::connect(path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await?;

    let mut client = KernelServiceClient::new(channel);
    let mut telemetry_stream = client.stream_telemetry(Empty {}).await?.into_inner();

    let mut app = KoadApp::new();
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Handle incoming telemetry
        if let Ok(Ok(Some(update))) = tokio::time::timeout(Duration::from_millis(1), telemetry_stream.message()).await {
            let msg = update.message;
            if let Ok(stats) = serde_json::from_str::<SystemStats>(&msg) {
                app.stats = Some(stats);
            } else {
                app.terminal_logs.push(msg);
                app.items_counts[3] = app.terminal_logs.len();
            }
        }

        terminal.draw(|f| {
            let area = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // Header
                    Constraint::Min(10),    // Main Content
                    Constraint::Length(3),  // Footer
                ])
                .split(area);

            // 1. Header
            let header = Paragraph::new(" KoadOS v3 Command Center ")
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // 2. Main Body: 4 Columns
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(chunks[1]);

            render_stateful_column(f, " [CREW] Active Agents ", vec![], body_chunks[0], &mut app.states[0], app.active_column == 0);
            
            let mut state_items = vec![];
            if let Some(s) = &app.stats {
                state_items.push(ListItem::new(format!("CPU: {:.1}%", s.cpu_usage)).style(Style::default().fg(Color::Yellow)));
                state_items.push(ListItem::new(format!("MEM: {} MB", s.memory_usage)).style(Style::default().fg(Color::Cyan)));
                state_items.push(ListItem::new(format!("UPTIME: {}s", s.uptime)).style(Style::default().fg(Color::Green)));
            }
            render_stateful_column(f, " [STATE] Live Metrics ", state_items, body_chunks[1], &mut app.states[1], app.active_column == 1);
            
            render_stateful_column(f, " [BRIDGE] Command Log ", vec![], body_chunks[2], &mut app.states[2], app.active_column == 2);
            
            let telemetry_items: Vec<ListItem> = app.terminal_logs.iter()
                .rev()
                .take(20)
                .map(|m| ListItem::new(m.clone()).style(Style::default().fg(Color::Cyan)))
                .collect();
            render_stateful_column(f, " [COMMS] Telemetry ", telemetry_items, body_chunks[3], &mut app.states[3], app.active_column == 3);

            // 3. Footer
            let footer = Paragraph::new(" [q] Exit | [TAB] Switch Col | [j/k] Scroll | v3 Alpha ")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => app.next_col(),
                    KeyCode::Right | KeyCode::Char('l') => app.next_col(),
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
