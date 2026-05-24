/// omara-unstable
/// Fully self-contained screensaver.
/// 
/// TODO: Implement your animation logic here, modeled after omara-bounce.

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    Terminal,
};
use std::io;
use std::time::Duration;

// === BRANDING (recommended pattern - copy this block) ===
pub const DEFAULT_ART: &str = include_str!("../../assets/brand/omara.txt");

pub fn load_branding() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        let user_path = config_dir.join("omara/branding/screensaver.txt");
        if user_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&user_path) {
                if !content.trim().is_empty() {
                    return content;
                }
            }
        }
    }
    DEFAULT_ART.to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    // TODO: Implement your screensaver logic here
    let _logo = load_branding();

    loop {
        if event::poll(Duration::from_millis(16))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break;
            }
        }
        terminal.draw(|f| {
            f.render_widget(
                ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
                f.area(),
            );
        })?;
        std::thread::sleep(Duration::from_millis(16));
    }

    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}
