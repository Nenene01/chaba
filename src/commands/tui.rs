use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

use crate::config::Config;
use crate::core::git::GitOps;
use crate::core::worktree::WorktreeManager;
use crate::error::Result;

pub async fn execute() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load reviews
    let config = Config::load()?;
    let manager = WorktreeManager::new(config)?;
    let _git_ops = GitOps::open()?;
    let reviews = manager.list()?;

    let mut selected = 0;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Title
            let title = Paragraph::new("🍵 Chaba - Review Environments")
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Review list
            let items: Vec<ListItem> = reviews
                .iter()
                .enumerate()
                .map(|(i, review)| {
                    let status = if review.worktree_path.exists() {
                        "✓"
                    } else {
                        "⚠️"
                    };

                    let content = format!(
                        "{} PR #{:<6} {} ({})",
                        status,
                        review.pr_number,
                        review.branch,
                        if review.worktree_path.exists() {
                            "Active"
                        } else {
                            "Missing"
                        }
                    );

                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    ListItem::new(Line::from(vec![Span::styled(content, style)]))
                })
                .collect();

            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Reviews"));
            f.render_widget(list, chunks[1]);

            // Help
            let help = Paragraph::new("↑/↓: Navigate | Enter: Open | q: Quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if selected < reviews.len().saturating_sub(1) {
                            selected += 1;
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        if selected < reviews.len() {
                            // Show selected review info
                            // In a real implementation, this would navigate to a detail view
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}
