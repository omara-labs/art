// omara-matrix - Fully self-contained Matrix-style digital rain screensaver.
// Dense green rain using OMARA letters + matrix symbols.
// Fully dynamic / resize friendly. Dim OMARA logo centered.

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Terminal,
};
use std::io;
use std::time::Duration;
use rand::Rng;

pub const DEFAULT_ART: &str = include_str!("../../../assets/brand/omara.txt");

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

const CHAR_POOL: &str = "OMARA:.*\"=+-><|/\\01";

struct RainDrop {
    x: f32,
    y: f32,
    speed: f32,
    length: u16,
    chars: Vec<char>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_matrix(&mut terminal);

    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;

    if let Err(err) = result {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_matrix<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut drops: Vec<RainDrop> = Vec::new();
    let char_pool: Vec<char> = CHAR_POOL.chars().collect();
    let mut rng = rand::rng();

    // We will rebuild drops on first frame based on actual size
    let mut last_width: u16 = 0;
    let mut last_height: u16 = 0;

    loop {
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        // Rebuild / adjust drops if terminal was resized
        if width != last_width || height != last_height {
            drops.clear();
            let target_count = (width as usize * 3).max(60);
            for _ in 0..target_count {
                let x = rng.random::<f32>() * width as f32;
                let y = rng.random::<f32>() * -(height as f32 * 1.5);
                let speed = rng.random::<f32>() * 1.35 + 0.55;
                let length = rng.random::<u16>() % 18 + 6;
                let mut chars = char_pool.clone();
                if rng.random_bool(0.3) {
                    let rot = rng.random_range(0..chars.len());
                    chars.rotate_left(rot);
                }
                drops.push(RainDrop { x, y, speed, length, chars });
            }
            last_width = width;
            last_height = height;
        }

        // Update rain
        for drop in &mut drops {
            drop.y += drop.speed;

            if drop.y - drop.length as f32 > height as f32 {
                drop.y = rng.random::<f32>() * -(height as f32 * 0.8);
                drop.speed = rng.random::<f32>() * 1.35 + 0.55;
                drop.length = rng.random::<u16>() % 18 + 6;
                drop.x = rng.random::<f32>() * width as f32;
            }
        }

        // Extra density for very wide terminals
        if rng.random::<f32>() < 0.12 {
            drops.push(RainDrop {
                x: rng.random::<f32>() * width as f32,
                y: rng.random::<f32>() * -(height as f32 * 0.6),
                speed: rng.random::<f32>() * 1.4 + 0.6,
                length: rng.random::<u16>() % 14 + 5,
                chars: char_pool.clone(),
            });
        }

        // Keep reasonable density
        let target = (width as usize * 3).max(80);
        if drops.len() > target + 80 {
            drops.truncate(target);
        }

        terminal.draw(|f| {
            let area = f.area();

            // Black background
            f.render_widget(
                ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
                area,
            );

            // Dim centered OMARA logo
            let logo_x = width.saturating_sub(logo_width) / 2;
            let logo_y = height.saturating_sub(logo_height) / 2;
            let dim_green = Color::Rgb(0, 48, 18);

            for (i, line) in logo_lines.iter().enumerate() {
                let y = logo_y + i as u16;
                if y < height {
                    let para = Paragraph::new(*line).style(Style::default().fg(dim_green));
                    f.render_widget(para, Rect::new(logo_x, y, logo_width, 1));
                }
            }

            // Dense green rain
            for drop in &drops {
                for i in 0..drop.length {
                    let yy = drop.y - i as f32;
                    if yy < 0.0 { continue; }
                    let y = yy as u16;
                    if y >= height { continue; }
                    let x = drop.x as u16;
                    if x >= width { continue; }

                    let ch = drop.chars[i as usize % drop.chars.len()];
                    let brightness = if i == 0 {
                        230
                    } else {
                        195u16.saturating_sub(i as u16 * 10)
                    };
                    let color = Color::Rgb(0, brightness as u8, 32);

                    let para = Paragraph::new(ch.to_string())
                        .style(Style::default().fg(color));
                    f.render_widget(para, Rect::new(x, y, 1, 1));
                }
            }
        })?;

        std::thread::sleep(Duration::from_millis(13));
    }
}
