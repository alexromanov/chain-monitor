use super::app::{App, LogLevel, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};
use crate::chains::Chain;
use ratatui::widgets::Sparkline;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());
    
    draw_header(f, chunks[0]);
    draw_tabs(f, app, chunks[1]);
    
    match app.current_tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[2]),
        Tab::ChainDetails => draw_chain_details(f, app, chunks[2]),
        Tab::Logs => draw_logs(f, app, chunks[2]),
    }
    
    draw_footer(f, chunks[3]);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("🔗 ", Style::default().fg(Color::Cyan)),
            Span::styled(
                "Multi-Chain Monitor",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default());
    
    f.render_widget(header, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Dashboard [1]", "Chain Details [2]", "Logs [3]"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .select(match app.current_tab {
            Tab::Dashboard => 0,
            Tab::ChainDetails => 1,
            Tab::Logs => 2,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    
    f.render_widget(tabs, area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    
    draw_chain_table(f, app, chunks[0]);
    draw_mini_logs(f, app, chunks[1]);
}

fn draw_chain_table(f: &mut Frame, app: &App, area: Rect) {
    let header_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    
    let header = Row::new(vec![
        "Chain",
        "Status",
        "Height",
        "TPS",
        "Block Time",
        "Txs",
    ])
    .style(header_style)
    .height(1);
    
    let chains: Vec<_> = Chain::all();
    let rows: Vec<Row> = chains
        .iter()
        .enumerate()
        .map(|(idx, chain)| {
            let status = app.chains.get(chain);
            
            let (height, tps, block_time, txs, health) = if let Some(s) = status {
                let height = s.latest_block
                    .as_ref()
                    .map(|b| b.height.to_string())
                    .unwrap_or_else(|| "-".to_string());
                
                let tps = s.metrics
                    .as_ref()
                    .map(|m| format!("{:.2}", m.tps))
                    .unwrap_or_else(|| "-".to_string());
                
                let block_time = s.metrics
                    .as_ref()
                    .map(|m| format!("{:.1}s", m.block_time))
                    .unwrap_or_else(|| "-".to_string());
                
                let txs = s.latest_block
                    .as_ref()
                    .map(|b| b.transaction_count.to_string())
                    .unwrap_or_else(|| "-".to_string());
                
                let health = s.health_indicator();
                
                (height, tps, block_time, txs, health)
            } else {
                ("-".to_string(), "-".to_string(), "-".to_string(), "-".to_string(), "🔴")
            };
            
            let style = if idx == app.selected_chain_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
            } else {
                Style::default().fg(Color::White)
            };
            
            Row::new(vec![
                chain.name().to_string(),
                health.to_string(),
                height,
                tps,
                block_time,
                txs,
            ])
            .style(style)
        })
        .collect();
    
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Chains (↑/↓ to select)"),
    );
    
    f.render_widget(table, area);
}

fn draw_mini_logs(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(20)
        .map(|log| {
            let style = match log.level {
                LogLevel::Info => Style::default().fg(Color::Green),
                LogLevel::Warning => Style::default().fg(Color::Yellow),
                LogLevel::Error => Style::default().fg(Color::Red),
            };
            
            let elapsed = log.timestamp.elapsed().as_secs();
            let time_str = if elapsed < 60 {
                format!("{}s ago", elapsed)
            } else {
                format!("{}m ago", elapsed / 60)
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::DarkGray)),
                Span::styled(&log.message, style),
            ]))
        })
        .collect();
    
    let logs_widget = List::new(logs).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Recent Logs"),
    );
    
    f.render_widget(logs_widget, area);
}

fn draw_chain_details(f: &mut Frame, app: &App, area: Rect) {
    if let Some(status) = app.get_selected_chain() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(area);
        
        let mut details = vec![
            Line::from(vec![
                Span::styled("Chain: ", Style::default().fg(Color::Yellow)),
                Span::raw(status.chain.name()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::raw(status.health_indicator()),
            ]),
        ];
        
        if let Some(block) = &status.latest_block {
            details.push(Line::from(vec![
                Span::styled("Latest Block: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("#{}", block.height)),
            ]));
            details.push(Line::from(vec![
                Span::styled("Block Hash: ", Style::default().fg(Color::Yellow)),
                Span::raw(&block.hash[..20.min(block.hash.len())]),
                Span::raw("..."),
            ]));
            details.push(Line::from(vec![
                Span::styled("Transactions: ", Style::default().fg(Color::Yellow)),
                Span::raw(block.transaction_count.to_string()),
            ]));
        }
        
        if let Some(metrics) = &status.metrics {
            details.push(Line::from(vec![
                Span::styled("TPS: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:.2}", metrics.tps)),
            ]));
            details.push(Line::from(vec![
                Span::styled("Block Time: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{:.2}s", metrics.block_time)),
            ]));
        }
        
        let details_widget = Paragraph::new(details).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Chain Details"),
        );
        
        f.render_widget(details_widget, chunks[0]);

        if !status.tps_history.is_empty() {
            let tps_data: Vec<u64> = status.tps_history.iter().copied().collect();
            let max_tps = tps_data.iter().max().copied().unwrap_or(1);
            
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("TPS History (max: {})", max_tps)),
                )
                .data(&tps_data)
                .style(Style::default().fg(Color::Cyan));
            
            f.render_widget(sparkline, chunks[1]);
        }
        
        let history: Vec<ListItem> = status
            .block_history
            .iter()
            .rev()
            .take(20)
            .map(|block| {
                ListItem::new(Line::from(format!(
                    "Block #{} - {} txs",
                    block.height, block.transaction_count
                )))
            })
            .collect();
        
        let history_widget = List::new(history).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Blocks"),
        );
        
        f.render_widget(history_widget, chunks[1]);
    } else {
        let msg = Paragraph::new("No chain selected")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, area);
    }
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .map(|log| {
            let style = match log.level {
                LogLevel::Info => Style::default().fg(Color::Green),
                LogLevel::Warning => Style::default().fg(Color::Yellow),
                LogLevel::Error => Style::default().fg(Color::Red),
            };
            
            let elapsed = log.timestamp.elapsed().as_secs();
            let time_str = if elapsed < 60 {
                format!("{}s ago", elapsed)
            } else if elapsed < 3600 {
                format!("{}m ago", elapsed / 60)
            } else {
                format!("{}h ago", elapsed / 3600)
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::DarkGray)),
                Span::styled(&log.message, style),
            ]))
        })
        .collect();
    
    let logs_widget = List::new(logs).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Application Logs"),
    );
    
    f.render_widget(logs_widget, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(": Quit | "),
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(": Select Chain | "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(": Switch View | "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(": Refresh"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(footer, area);
}