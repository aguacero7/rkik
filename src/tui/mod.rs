use std::io::{self, stdout};
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use crate::domain::ntp::ProbeResult;

/// Server status for TUI display
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub name: String,
    pub offset_ms: Option<f64>,
    pub delay_ms: Option<f64>,
    pub stratum: Option<u8>,
    pub last_query: Option<Instant>,
    pub success: bool,
    pub error: Option<String>,
}

impl ServerStatus {
    pub fn new(name: String) -> Self {
        Self {
            name,
            offset_ms: None,
            delay_ms: None,
            stratum: None,
            last_query: None,
            success: false,
            error: None,
        }
    }

    pub fn update_from_result(&mut self, result: &ProbeResult) {
        self.offset_ms = Some(result.offset_ms);
        self.delay_ms = Some(result.rtt_ms);
        self.stratum = Some(result.stratum);
        self.last_query = Some(Instant::now());
        self.success = true;
        self.error = None;
    }

    pub fn update_error(&mut self, error: String) {
        self.last_query = Some(Instant::now());
        self.success = false;
        self.error = Some(error);
    }
}

/// Global statistics for TUI
#[derive(Debug, Default, Clone)]
pub struct GlobalStats {
    pub total_queries: usize,
    pub successful_queries: usize,
    pub failed_queries: usize,
    pub avg_offset: f64,
    pub avg_delay: f64,
    pub min_offset: f64,
    pub max_offset: f64,
    pub current_cycle: u32,
}

/// TUI application state
pub struct TuiApp {
    pub servers: Vec<ServerStatus>,
    pub stats: GlobalStats,
    pub should_quit: bool,
    pub paused: bool,
    pub total_servers: usize,
    pub completed_this_cycle: usize,
}

impl TuiApp {
    pub fn new(server_names: Vec<String>) -> Self {
        let total = server_names.len();
        let servers = server_names.into_iter().map(ServerStatus::new).collect();

        Self {
            servers,
            stats: GlobalStats::default(),
            should_quit: false,
            paused: false,
            total_servers: total,
            completed_this_cycle: 0,
        }
    }

    pub fn update_server(&mut self, server_name: &str, result: &ProbeResult) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.name == server_name) {
            server.update_from_result(result);
            self.stats.successful_queries += 1;
            self.stats.total_queries += 1;
            self.completed_this_cycle += 1;
            self.recalculate_stats();
        }
    }

    pub fn update_server_error(&mut self, server_name: &str, error: String) {
        if let Some(server) = self.servers.iter_mut().find(|s| s.name == server_name) {
            server.update_error(error);
            self.stats.failed_queries += 1;
            self.stats.total_queries += 1;
            self.completed_this_cycle += 1;
        }
    }

    pub fn start_new_cycle(&mut self) {
        self.completed_this_cycle = 0;
        self.stats.current_cycle += 1;
    }

    fn recalculate_stats(&mut self) {
        let successful: Vec<_> = self.servers.iter()
            .filter(|s| s.success && s.offset_ms.is_some())
            .collect();

        if successful.is_empty() {
            return;
        }

        let offsets: Vec<f64> = successful.iter()
            .filter_map(|s| s.offset_ms)
            .collect();

        let delays: Vec<f64> = successful.iter()
            .filter_map(|s| s.delay_ms)
            .collect();

        if !offsets.is_empty() {
            self.stats.avg_offset = offsets.iter().sum::<f64>() / offsets.len() as f64;
            self.stats.min_offset = offsets.iter().cloned().fold(f64::INFINITY, f64::min);
            self.stats.max_offset = offsets.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        }

        if !delays.is_empty() {
            self.stats.avg_delay = delays.iter().sum::<f64>() / delays.len() as f64;
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.paused = !self.paused;
            }
            _ => {}
        }
    }
}

pub fn ui(frame: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Progress
            Constraint::Length(9),  // Global stats
            Constraint::Min(10),    // Server list
            Constraint::Length(3),  // Help
        ])
        .split(frame.area());

    render_title(frame, chunks[0], app);
    render_progress(frame, chunks[1], app);
    render_global_stats(frame, chunks[2], app);
    render_server_list(frame, chunks[3], app);
    render_help(frame, chunks[4]);
}

fn render_title(frame: &mut Frame, area: Rect, _app: &TuiApp) {
    let title = Paragraph::new("RKIK - Infinite Monitoring Mode")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_progress(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let progress_text = if app.paused {
        format!("⏸  PAUSED - Cycle #{} - {} / {} servers completed",
            app.stats.current_cycle,
            app.completed_this_cycle,
            app.total_servers)
    } else {
        format!("▶  Cycle #{} - {} / {} servers completed",
            app.stats.current_cycle,
            app.completed_this_cycle,
            app.total_servers)
    };

    let progress = Paragraph::new(progress_text)
        .style(Style::default().fg(if app.paused { Color::Yellow } else { Color::Green }))
        .block(Block::default().borders(Borders::ALL).title("Progress"));
    frame.render_widget(progress, area);
}

fn render_global_stats(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let success_rate = if app.stats.total_queries > 0 {
        (app.stats.successful_queries as f64 / app.stats.total_queries as f64) * 100.0
    } else {
        0.0
    };

    let stats_text = vec![
        Line::from(vec![
            Span::raw("Total queries: "),
            Span::styled(
                format!("{}", app.stats.total_queries),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Success / Failures: "),
            Span::styled(
                format!("{}", app.stats.successful_queries),
                Style::default().fg(Color::Green),
            ),
            Span::raw(" / "),
            Span::styled(
                format!("{}", app.stats.failed_queries),
                Style::default().fg(Color::Red),
            ),
            Span::raw(format!(" ({:.1}%)", success_rate)),
        ]),
        Line::from(vec![
            Span::raw("Avg offset: "),
            Span::styled(
                format!("{:.3} ms", app.stats.avg_offset),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Avg delay: "),
            Span::styled(
                format!("{:.3} ms", app.stats.avg_delay),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Offset range: "),
            Span::styled(
                format!("[{:.3}, {:.3}] ms", app.stats.min_offset, app.stats.max_offset),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Global Statistics"));
    frame.render_widget(stats, area);
}

fn render_server_list(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let items: Vec<ListItem> = app.servers.iter().map(|server| {
        let status_symbol = if server.success {
            "✓"
        } else if server.error.is_some() {
            "✗"
        } else {
            "○"
        };

        let status_color = if server.success {
            Color::Green
        } else if server.error.is_some() {
            Color::Red
        } else {
            Color::Gray
        };

        let offset_str = server.offset_ms
            .map(|o| format!("{:>8.3}", o))
            .unwrap_or_else(|| "     N/A".to_string());

        let delay_str = server.delay_ms
            .map(|d| format!("{:>8.3}", d))
            .unwrap_or_else(|| "     N/A".to_string());

        let stratum_str = server.stratum
            .map(|s| format!("{:>2}", s))
            .unwrap_or_else(|| " -".to_string());

        let line = Line::from(vec![
            Span::styled(
                format!("{} ", status_symbol),
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:<30}", server.name),
                Style::default().fg(Color::White),
            ),
            Span::raw(" Offset: "),
            Span::styled(
                format!("{} ms", offset_str),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" Delay: "),
            Span::styled(
                format!("{} ms", delay_str),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" Stratum: "),
            Span::styled(
                stratum_str,
                Style::default().fg(Color::Magenta),
            ),
        ]);

        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Servers"));
    frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new("q: Quit | p: Pause/Resume | Ctrl+C: Exit")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, area);
}

/// Run the TUI application
pub fn run_tui<F>(app: &mut TuiApp, mut update_fn: F) -> io::Result<()>
where
    F: FnMut(&mut TuiApp) -> io::Result<bool>, // Returns true if should continue
{
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run_app(&mut terminal, app, &mut update_fn);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_app<F>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    update_fn: &mut F,
) -> io::Result<()>
where
    F: FnMut(&mut TuiApp) -> io::Result<bool>,
{
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Handle events with a timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }

        // Call update function if not paused
        if !app.paused {
            if !update_fn(app)? {
                break;
            }
        }
    }

    Ok(())
}
