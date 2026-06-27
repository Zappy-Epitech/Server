use std::io;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Row, Table, Tabs, Wrap, canvas::{Canvas, Rectangle}},
    Terminal, Frame,
};
use std::collections::HashMap;

use crate::tui::ServerEvent;
use crate::config::ServerConfig;

struct TuiState {
    logs: Vec<(String, Color)>,
    connected_clients: usize,
    team_counts: HashMap<String, usize>,
    freq: u32,
    game_over: Option<String>,
    start_time: Instant,
    total_commands_executed: usize,
    active_tab: usize,
    player_positions: Vec<(u32, u32)>,
}

pub fn run(rx: Receiver<ServerEvent>, config: &ServerConfig) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState {
        logs: Vec::new(),
        connected_clients: 0,
        team_counts: HashMap::new(),
        freq: config.freq,
        game_over: None,
        start_time: Instant::now(),
        total_commands_executed: 0,
        active_tab: 0,
        player_positions: Vec::new(),
    };

    for team in &config.teams {
        state.team_counts.insert(team.clone(), 0);
    }

    let res = run_app(&mut terminal, &mut state, rx, config);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn determine_log_color(msg: &str) -> Color {
    let lower_msg = msg.to_lowercase();
    if lower_msg.contains("connected") || lower_msg.contains("joined") {
        Color::Green
    } else if lower_msg.contains("disconnected") || lower_msg.contains("starved") || lower_msg.contains("dead") {
        Color::Red
    } else if lower_msg.contains("executing") {
        Color::Yellow
    } else if lower_msg.contains("gui") {
        Color::Blue
    } else if lower_msg.contains("game over") {
        Color::Magenta
    } else {
        Color::White
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut TuiState,
    rx: Receiver<ServerEvent>,
    config: &ServerConfig,
) -> io::Result<()> {
    loop {
        while let Ok(event) = rx.try_recv() {
            match event {
                ServerEvent::Log(msg) => {
                    let color = determine_log_color(&msg);
                    state.logs.push((msg, color));
                    if state.logs.len() > 200 {
                        state.logs.remove(0);
                    }
                    if color == Color::Yellow {
                        state.total_commands_executed += 1;
                    }
                }
                ServerEvent::ClientConnected => state.connected_clients += 1,
                ServerEvent::ClientDisconnected => {
                    if state.connected_clients > 0 {
                        state.connected_clients -= 1;
                    }
                }
                ServerEvent::PlayerJoinedTeam(team) => {
                    *state.team_counts.entry(team).or_insert(0) += 1;
                }
                ServerEvent::PlayerDied(team) => {
                    if let Some(count) = state.team_counts.get_mut(&team) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                }
                ServerEvent::FreqChanged(new_freq) => state.freq = new_freq,
                ServerEvent::GameOver(team) => state.game_over = Some(team),
                ServerEvent::MapSnapshot(positions) => state.player_positions = positions,
            }
        }

        terminal.draw(|f| ui(f, state, config))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('c') => state.logs.clear(),
                    KeyCode::Right | KeyCode::Char('2') => state.active_tab = 1,
                    KeyCode::Left | KeyCode::Char('1') => state.active_tab = 0,
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, state: &TuiState, config: &ServerConfig) {
    let size = f.size();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(size);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(main_chunks[0]);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Cyan));

    let ascii_art = "\
███████╗ █████╗ ██████╗ ██████╗ ██╗   ██╗
╚══███╔╝██╔══██╗██╔══██╗██╔══██╗╚██╗ ██╔╝
  ███╔╝ ███████║██████╔╝██████╔╝ ╚████╔╝
 ███╔╝  ██╔══██║██╔═══╝ ██╔═══╝   ╚██╔╝
███████╗██║  ██║██║     ██║        ██║
╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝        ╚═╝";

    f.render_widget(title_block.clone(), header_chunks[0]);

    let title_inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .split(title_block.inner(header_chunks[0]));

    let title_p = Paragraph::new(ascii_art)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    f.render_widget(title_p, title_inner_chunks[0]);

    let title_p = Paragraph::new(ascii_art)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title_p, title_inner_chunks[0]);

    let titles: Vec<Line> = vec!["[1] Global Dashboard", "[2] Live Minimap"]
        .into_iter()
        .map(Line::from)
        .collect();
    let tabs = Tabs::new(titles)
        .select(state.active_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider(" | ");
    f.render_widget(tabs, title_inner_chunks[1]);

    let (status_text, status_color) = match &state.game_over {
        Some(team) => (format!("GAME OVER\nTEAM {} WINS!", team), Color::Magenta),
        None => ("SERVER ACTIVE\nRUNNING SMOOTHLY".to_string(), Color::Green),
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().title(" System Status ").borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(status, header_chunks[1]);

    if state.active_tab == 0 {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(65),
                Constraint::Percentage(35),
            ])
            .split(main_chunks[1]);

        let logs_lines: Vec<Line> = state.logs.iter()
            .map(|(msg, color)| Line::from(Span::styled(msg.clone(), Style::default().fg(*color))))
            .collect();
        
        let logs_p = Paragraph::new(logs_lines)
            .block(Block::default().title(" Live Event Stream ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .wrap(Wrap { trim: true });
        
        let scroll_y = if state.logs.len() as u16 > content_chunks[0].height.saturating_sub(2) {
            state.logs.len() as u16 - content_chunks[0].height.saturating_sub(2)
        } else {
            0
        };
        f.render_widget(logs_p.scroll((scroll_y, 0)), content_chunks[0]);

        let metrics_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Min(0),
            ])
            .split(content_chunks[1]);

        let uptime = state.start_time.elapsed().as_secs();
        let stats_text = vec![
            Line::from(vec![Span::styled("Port: ", Style::default().fg(Color::DarkGray)), Span::raw(config.port.to_string())]),
            Line::from(vec![Span::styled("Map Size: ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{}x{}", config.width, config.height))]),
            Line::from(vec![Span::styled("Frequency (f): ", Style::default().fg(Color::DarkGray)), Span::styled(state.freq.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("Total Connections: ", Style::default().fg(Color::DarkGray)), Span::raw(state.connected_clients.to_string())]),
            Line::from(vec![Span::styled("Commands Exec: ", Style::default().fg(Color::DarkGray)), Span::raw(state.total_commands_executed.to_string())]),
            Line::from(vec![Span::styled("Uptime: ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{}s", uptime))]),
        ];
        let stats_p = Paragraph::new(stats_text)
            .block(Block::default().title(" Global Metrics ").borders(Borders::ALL).border_type(BorderType::Rounded));
        f.render_widget(stats_p, metrics_chunks[0]);

        let mut team_rows = Vec::new();
        for team in &config.teams {
            let count = *state.team_counts.get(team).unwrap_or(&0);
            team_rows.push(Row::new(vec![team.clone(), count.to_string()]));
        }

        let team_table = Table::new(team_rows, [Constraint::Percentage(60), Constraint::Percentage(40)])
            .header(Row::new(vec!["Team Name", "Active Players"]).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
            .block(Block::default().title(" Team Ladder ").borders(Borders::ALL).border_type(BorderType::Rounded));
        f.render_widget(team_table, metrics_chunks[1]);
    } else {
        let canvas = Canvas::default()
            .block(Block::default().title(" Trantor Live Map ").borders(Borders::ALL).border_type(BorderType::Rounded))
            .paint(|ctx| {
                ctx.draw(&Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: config.width as f64,
                    height: config.height as f64,
                    color: Color::DarkGray,
                });

                for (x, y) in &state.player_positions {
                    ctx.print(*x as f64, (config.height - 1 - *y) as f64, Span::styled("●", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                }
            })
            .x_bounds([0.0, config.width as f64])
            .y_bounds([0.0, config.height as f64]);
        
        f.render_widget(canvas, main_chunks[1]);
    }

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[2]);

    let total_players: usize = state.team_counts.values().sum();
    let max_capacity = (config.teams.len() * config.clients_nb) as u16;
    let load_ratio = if max_capacity > 0 {
        (total_players as f64 / max_capacity as f64).min(1.0)
    } else {
        0.0
    };
    
    let gauge_color = if load_ratio > 0.8 { Color::Red } else if load_ratio > 0.5 { Color::Yellow } else { Color::Green };
    let gauge = Gauge::default()
        .block(Block::default().title(" Server Load (Initial Slots) ").borders(Borders::ALL).border_type(BorderType::Rounded))
        .gauge_style(Style::default().fg(gauge_color))
        .ratio(load_ratio);
    f.render_widget(gauge, footer_chunks[0]);

    let shortcuts = Paragraph::new(" Shortcuts: [1/2] Tabs  |  [q] Quit  |  [c] Clear Logs ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(shortcuts, footer_chunks[1]);
}
