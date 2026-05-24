// omara-pour - Fully self-contained screensaver.
// Glyphs cascade/pour from above and build the logo in cyan/light blue.

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style, Modifier},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use rand::Rng;

// === BRANDING ===
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

struct LogoCell {
    x: u16,
    y: u16,
    ch: char,
    active: bool,
    glow: f32,
}

struct Drop {
    x: f32,
    y: f32,
    vy: f32,
    ch: char,
}

struct Splash {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

#[derive(PartialEq)]
enum Phase {
    Building,
    Complete,
    WashingAway,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_pour(&mut terminal);

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

fn run_pour<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut logo_cells: Vec<LogoCell> = Vec::new();
    let mut drops: Vec<Drop> = Vec::new();
    let mut splashes: Vec<Splash> = Vec::new();

    let mut phase = Phase::Building;
    let mut phase_timer = Instant::now();

    let mut last_width = 0;
    let mut last_height = 0;

    let glyphs_pool: Vec<char> = "OMARA░▒▓█*+=:-".chars().collect();

    let mut last_frame = Instant::now();

    loop {
        // Exit on key press or mouse event
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32().min(0.1);
        last_frame = now;

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        if width == 0 || height == 0 {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // Handle terminal resizing and (re)positioning the logo cells
        if width != last_width || height != last_height {
            logo_cells.clear();
            splashes.clear();

            let logo_x = width.saturating_sub(logo_width) / 2;
            let logo_y = height.saturating_sub(logo_height) / 2;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                for (x_offset, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        logo_cells.push(LogoCell {
                            x: logo_x + x_offset as u16,
                            y: logo_y + y_offset as u16,
                            ch,
                            active: false,
                            glow: 0.0,
                        });
                    }
                }
            }

            drops.clear();
            phase = Phase::Building;
            last_width = width;
            last_height = height;
        }

        // Adjust drop count dynamically based on screen width
        let target_drops = (width as usize * 4 / 3).clamp(20, 150);
        if drops.len() < target_drops {
            while drops.len() < target_drops {
                // 50% chance to target a column containing an inactive logo cell
                let x = if phase == Phase::Building && rng.random_bool(0.5) {
                    let inactive_cells: Vec<&LogoCell> = logo_cells.iter().filter(|c| !c.active).collect();
                    if !inactive_cells.is_empty() {
                        let selected_cell = inactive_cells[rng.random_range(0..inactive_cells.len())];
                        selected_cell.x as f32
                    } else {
                        rng.random::<f32>() * width as f32
                    }
                } else {
                    rng.random::<f32>() * width as f32
                };

                drops.push(Drop {
                    x,
                    y: -(rng.random::<f32>() * height as f32),
                    vy: rng.random::<f32>() * 12.0 + 8.0, // falling speed
                    ch: glyphs_pool[rng.random_range(0..glyphs_pool.len())],
                });
            }
        } else if drops.len() > target_drops {
            drops.truncate(target_drops);
        }

        // Update phase timers and rules
        match phase {
            Phase::Building => {
                let all_active = logo_cells.iter().all(|c| c.active);
                if all_active {
                    phase = Phase::Complete;
                    phase_timer = Instant::now();
                }
            }
            Phase::Complete => {
                if phase_timer.elapsed() > Duration::from_secs(6) {
                    phase = Phase::WashingAway;
                    phase_timer = Instant::now();
                }
            }
            Phase::WashingAway => {
                // Deactivate a few random cells per frame
                let active_indices: Vec<usize> = logo_cells
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.active)
                    .map(|(i, _)| i)
                    .collect();

                if active_indices.is_empty() {
                    phase = Phase::Building;
                    phase_timer = Instant::now();
                } else {
                    // Wash away 2-4 cells per frame
                    let count = rng.random_range(2..6).min(active_indices.len());
                    for _ in 0..count {
                        let idx = active_indices[rng.random_range(0..active_indices.len())];
                        logo_cells[idx].active = false;
                        logo_cells[idx].glow = 0.0;

                        // Spawn a falling drop where the cell was
                        drops.push(Drop {
                            x: logo_cells[idx].x as f32,
                            y: logo_cells[idx].y as f32,
                            vy: rng.random::<f32>() * 8.0 + 6.0,
                            ch: logo_cells[idx].ch,
                        });
                    }
                }
            }
        }

        // Update drops
        for drop in &mut drops {
            drop.y += drop.vy * delta;

            let col = drop.x as u16;
            let row = drop.y as u16;

            if col < width && row < height {
                // Check if the drop hits any logo cell in its column
                if phase == Phase::Building {
                    if let Some(cell) = logo_cells
                        .iter_mut()
                        .find(|c| c.x == col && c.y == row && !c.active)
                    {
                        // Land and activate!
                        cell.active = true;
                        cell.glow = 1.6; // super bright landing flash

                        // Spawn splash particles
                        for _ in 0..rng.random_range(2..5) {
                            splashes.push(Splash {
                                x: col as f32,
                                y: row as f32,
                                vx: rng.random::<f32>() * 3.0 - 1.5,
                                vy: -(rng.random::<f32>() * 2.5 + 1.0),
                                life: 1.0,
                            });
                        }

                        // Reset drop to the top
                        drop.y = -(rng.random::<f32>() * 4.0);
                        drop.x = if rng.random_bool(0.6) {
                            let inactive_cells: Vec<&LogoCell> = logo_cells.iter().filter(|c| !c.active).collect();
                            if !inactive_cells.is_empty() {
                                inactive_cells[rng.random_range(0..inactive_cells.len())].x as f32
                            } else {
                                rng.random::<f32>() * width as f32
                            }
                        } else {
                            rng.random::<f32>() * width as f32
                        };
                        drop.vy = rng.random::<f32>() * 12.0 + 8.0;
                    }
                } else if phase == Phase::Complete {
                    // Drops bounce off the top edges of active logo cells
                    let is_top_edge = logo_cells.iter().any(|c| c.x == col && c.y == row && c.active)
                        && !logo_cells.iter().any(|c| c.x == col && c.y == row.saturating_sub(1) && c.active);

                    if is_top_edge && rng.random_bool(0.3) {
                        // Landed on active logo - splash and reset
                        for _ in 0..rng.random_range(1..3) {
                            splashes.push(Splash {
                                x: col as f32,
                                y: row as f32,
                                vx: rng.random::<f32>() * 2.0 - 1.0,
                                vy: -(rng.random::<f32>() * 1.5 + 0.5),
                                life: 0.8,
                            });
                        }
                        drop.y = -(rng.random::<f32>() * 4.0);
                        drop.x = rng.random::<f32>() * width as f32;
                        drop.vy = rng.random::<f32>() * 12.0 + 8.0;
                    }
                }
            }

            // Reset drops that fall past the bottom
            if drop.y >= height as f32 {
                drop.y = -(rng.random::<f32>() * 4.0);
                drop.x = if phase == Phase::Building && rng.random_bool(0.5) {
                    let inactive_cells: Vec<&LogoCell> = logo_cells.iter().filter(|c| !c.active).collect();
                    if !inactive_cells.is_empty() {
                        inactive_cells[rng.random_range(0..inactive_cells.len())].x as f32
                    } else {
                        rng.random::<f32>() * width as f32
                    }
                } else {
                    rng.random::<f32>() * width as f32
                };
                drop.vy = rng.random::<f32>() * 12.0 + 8.0;
            }
        }

        // Update splashes
        for s in &mut splashes {
            s.x += s.vx * delta;
            s.y += s.vy * delta;
            s.vy += 9.8 * delta; // gravity
            s.life -= delta * 2.2; // decay life
        }
        splashes.retain(|s| s.life > 0.0);

        // Update logo cell glows
        for cell in &mut logo_cells {
            if cell.active && cell.glow > 0.0 {
                cell.glow -= delta * 1.2; // cool down
                if cell.glow < 0.0 {
                    cell.glow = 0.0;
                }
            }
        }

        // Render
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            // Clear screen (black background)
            for y in 0..area.height {
                for x in 0..area.width {
                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(Color::Black));
                }
            }

            // 1. Draw falling drops (water streams)
            for drop in &drops {
                let dx = drop.x as u16;
                let dy = drop.y as u16;

                if dx < area.width && dy < area.height {
                    // Draw a stream tail
                    for tail in 0..3 {
                        let ty = dy.saturating_sub(tail);
                        if ty < area.height {
                            let brightness = 200 - (tail * 60) as u8;
                            let cell = &mut buf[(dx, ty)];
                            // Only draw rain tail if the cell is currently black background (doesn't overwrite active logo)
                            let is_logo_active = logo_cells.iter().any(|c| c.x == dx && c.y == ty && c.active);
                            if !is_logo_active {
                                cell.set_symbol(&drop.ch.to_string());
                                cell.set_style(Style::default().fg(Color::Rgb(0, brightness, brightness + 55)));
                            }
                        }
                    }
                }
            }

            // 2. Draw splashes
            for s in &splashes {
                let sx = s.x as u16;
                let sy = s.y as u16;

                if sx < area.width && sy < area.height {
                    let is_logo_active = logo_cells.iter().any(|c| c.x == sx && c.y == sy && c.active);
                    if !is_logo_active {
                        let cell = &mut buf[(sx, sy)];
                        let alpha = (s.life * 255.0) as u8;
                        cell.set_symbol(".");
                        cell.set_style(Style::default().fg(Color::Rgb(alpha / 2, alpha, alpha)));
                    }
                }
            }

            // 3. Draw logo (building or completed)
            for cell in &logo_cells {
                if cell.x < area.width && cell.y < area.height {
                    let c = &mut buf[(cell.x, cell.y)];

                    if cell.active {
                        let glow_factor = cell.glow.min(1.5);
                        let style = if glow_factor > 1.0 {
                            // Flash color (white-cyan)
                            let extra = ((glow_factor - 1.0) * 2.0 * 255.0).min(255.0) as u8;
                            Style::default()
                                .fg(Color::Rgb(extra, 255, 255))
                                .add_modifier(Modifier::BOLD)
                        } else {
                            // Normal active glow fading to steady cyan
                            let r = (glow_factor * 120.0) as u8;
                            let g = (140.0 + glow_factor * 115.0) as u8;
                            let b = (220.0 + glow_factor * 35.0) as u8;
                            Style::default()
                                .fg(Color::Rgb(r, g, b))
                                .add_modifier(Modifier::BOLD)
                        };
                        c.set_symbol(&cell.ch.to_string());
                        c.set_style(style);
                    } else {
                        // Drawing inactive logo cells as a very dim cyan outline
                        c.set_symbol(&cell.ch.to_string());
                        c.set_style(Style::default().fg(Color::Rgb(0, 30, 45)));
                    }
                }
            }
        })?;

        let frame_time = Duration::from_millis(16);
        let elapsed = now.elapsed();
        if let Some(remaining) = frame_time.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }
    }
}
