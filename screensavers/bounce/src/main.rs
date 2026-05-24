// omara-bounce - Fully self-contained DVD-style bouncing Omara logo screensaver.
// Self-contained, resize-friendly version.

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
use std::time::{Duration, Instant};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_bounce(&mut terminal);

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

fn run_bounce<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut x: f32 = 5.0;
    let mut y: f32 = 3.0;
    let mut vx: f32 = 7.0;
    let mut vy: f32 = 3.8;

    let colors: Vec<Color> = vec![
        Color::Rgb(180, 0, 255),
        Color::Rgb(0, 200, 255),
        Color::Rgb(255, 200, 0),
        Color::Rgb(0, 255, 150),
        Color::Rgb(255, 100, 180),
    ];
    let mut color_index: usize = 0;

    // Simple starfield
    struct Star {
        x: f32,
        y: f32,
        ch: char,
        base_color: Color,
        phase: f32,
    }

    let mut rng = rand::rng();
    let stars: Vec<Star> = (0..160)
        .map(|i| {
            let ch = if i % 17 == 0 { '✦' } else if i % 7 == 0 { '•' } else { '.' };
            let brightness = if i % 9 == 0 { 200 } else if i % 4 == 0 { 140 } else { 80 };
            Star {
                x: rng.random::<f32>(),
                y: rng.random::<f32>(),
                ch,
                base_color: Color::Rgb(brightness, brightness, brightness + 25),
                phase: rng.random::<f32>() * std::f32::consts::TAU,
            }
        })
        .collect();

    let flare_indices: Vec<usize> = vec![7, 29, 61];
    let mut last_frame = Instant::now();

    loop {
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        let size = terminal.size()?;
        let max_x = size.width.saturating_sub(logo_width) as f32;
        let max_y = size.height.saturating_sub(logo_height) as f32;

        // Delta-time movement
        x += vx * delta;
        y += vy * delta;

        let mut hit_horizontal = false;
        let mut hit_vertical = false;

        if x <= 0.0 {
            x = 0.0;
            vx = -vx;
            hit_horizontal = true;
        }
        if x >= max_x {
            x = max_x;
            vx = -vx;
            hit_horizontal = true;
        }
        if y <= 0.0 {
            y = 0.0;
            vy = -vy;
            hit_vertical = true;
        }
        if y >= max_y {
            y = max_y;
            vy = -vy;
            hit_vertical = true;
        }

        if hit_horizontal || hit_vertical {
            color_index = (color_index + 1) % colors.len();
        }

        terminal.draw(|f| {
            let area = f.area();

            f.render_widget(
                ratatui::widgets::Block::default().style(Style::default().bg(Color::Black)),
                area,
            );

            // Starfield + simple lens flares
            let time = last_frame.elapsed().as_secs_f32();

            for star in &stars {
                let sx = (star.x * area.width as f32) as u16;
                let sy = (star.y * area.height as f32) as u16;

                if sx < area.width && sy < area.height {
                    let mut c = star.base_color;
                    let sparkle = ((time * 2.7 + star.phase).sin() * 70.0) as i16;
                    if let Color::Rgb(r, g, b) = c {
                        let adj = sparkle.clamp(-55, 55) as u8;
                        c = Color::Rgb(
                            r.saturating_add(adj),
                            g.saturating_add(adj / 2),
                            b.saturating_add(adj / 3),
                        );
                    }
                    let star_para = Paragraph::new(star.ch.to_string()).style(Style::default().fg(c));
                    f.render_widget(star_para, Rect::new(sx, sy, 1, 1));
                }
            }

            // Lens flares on a few stars
            for &idx in &flare_indices {
                if let Some(star) = stars.get(idx) {
                    let sx = (star.x * area.width as f32) as u16;
                    let sy = (star.y * area.height as f32) as u16;

                    for dx in 1..6 {
                        let alpha = 80 - (dx * 12) as u8;
                        if sx + dx < area.width {
                            let flare = Paragraph::new("─").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 30)));
                            f.render_widget(flare, Rect::new(sx + dx, sy, 1, 1));
                        }
                        if sx >= dx {
                            let flare = Paragraph::new("─").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 30)));
                            f.render_widget(flare, Rect::new(sx - dx, sy, 1, 1));
                        }
                    }
                    for dy in 1..4 {
                        let alpha = 70 - (dy * 15) as u8;
                        if sy + dy < area.height {
                            let flare = Paragraph::new("│").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 20)));
                            f.render_widget(flare, Rect::new(sx, sy + dy, 1, 1));
                        }
                        if sy >= dy {
                            let flare = Paragraph::new("│").style(Style::default().fg(Color::Rgb(alpha, alpha, alpha + 20)));
                            f.render_widget(flare, Rect::new(sx, sy - dy, 1, 1));
                        }
                    }
                }
            }

            // Bouncing logo
            let logo_rect = Rect {
                x: x as u16,
                y: y as u16,
                width: logo_width,
                height: logo_height,
            };

            let logo_para = Paragraph::new(logo.clone())
                .style(Style::default().fg(colors[color_index]).add_modifier(ratatui::style::Modifier::BOLD));

            f.render_widget(logo_para, logo_rect);
        })?;

        let frame_time = Duration::from_millis(16);
        let elapsed = now.elapsed();
        if let Some(remaining) = frame_time.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
    }
}
