// omara-wormhole - Fully self-contained wormhole tunnel screensaver.
// Volumetric 3D fly-through starfield with spacetime warping from passing singularities.

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

struct TunnelStar {
    x: f32, // offset from center
    y: f32,
    z: f32, // depth (100.0 down to 0.1)
    ch: char,
}

struct ZoomLetter {
    x: f32,
    y: f32,
    z: f32,
    ch: char,
    color: Color,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_wormhole(&mut terminal);

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

fn run_wormhole<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut stars: Vec<TunnelStar> = Vec::new();
    let mut zoom_letters: Vec<ZoomLetter> = Vec::new();

    let mut last_frame = Instant::now();
    let start_time = Instant::now();
    let mut last_width = 0;
    let mut last_height = 0;

    let star_symbols = ['.', '*', 'o', 'O', '+', 'x'];

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

        let time = start_time.elapsed().as_secs_f32();

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        if width == 0 || height == 0 {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // Initialize / Resize handling
        if width != last_width || height != last_height {
            stars.clear();
            zoom_letters.clear();

            let target_stars = (width as usize * 2).clamp(80, 240);
            for _ in 0..target_stars {
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                let r = rng.random::<f32>() * 12.0 + 1.5;
                stars.push(TunnelStar {
                    x: theta.cos() * r,
                    y: theta.sin() * r,
                    z: rng.random::<f32>() * 99.0 + 1.0,
                    ch: star_symbols[rng.random_range(0..star_symbols.len())],
                });
            }

            last_width = width;
            last_height = height;
        }

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        // Black Hole / Spacetime Warp trigger parameters
        // Passing black hole moves horizontally across the screen every 9 seconds
        let loop_period = 9.0;
        let bh_time = time % loop_period;
        let has_black_hole = bh_time < 5.0;
        let bh_x = if has_black_hole {
            let ratio = bh_time / 5.0; // 0.0 to 1.0
            width as f32 * 1.4 - ratio * (width as f32 * 1.8)
        } else {
            -100.0
        };
        let bh_y = center_y;
        let bh_r_event = (height as f32 * 0.12).clamp(3.0, 7.0);

        // Update tunnel stars (flying towards viewer)
        let tunnel_speed = 34.0f32;
        for star in &mut stars {
            star.z -= tunnel_speed * delta;

            if star.z <= 0.1 {
                star.z = 100.0;
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                let r = rng.random::<f32>() * 15.0 + 1.5;
                star.x = theta.cos() * r;
                star.y = theta.sin() * r;
                star.ch = star_symbols[rng.random_range(0..star_symbols.len())];
            }
        }

        // Spawn occasional zoom letters shooting out from center
        if zoom_letters.len() < 16 && rng.random_bool(0.06) {
            // Find a random non-space character from logo to shoot out
            let ry = rng.random_range(0..logo_height as usize);
            let line = logo_lines[ry];
            let active_chars: Vec<(usize, char)> = line.chars().enumerate().filter(|(_, ch)| *ch != ' ').collect();
            if !active_chars.is_empty() {
                let (_, ch) = active_chars[rng.random_range(0..active_chars.len())];
                let theta = rng.random::<f32>() * std::f32::consts::TAU;
                // Position offset
                zoom_letters.push(ZoomLetter {
                    x: theta.cos() * 2.0,
                    y: theta.sin() * 2.0,
                    z: 100.0,
                    ch,
                    color: Color::Rgb(0, (140.0 + rng.random::<f32>() * 115.0) as u8, 255),
                });
            }
        }

        // Update zoom letters
        for zl in &mut zoom_letters {
            zl.z -= tunnel_speed * 1.15 * delta; // slightly faster than stars
        }
        zoom_letters.retain(|zl| zl.z > 0.5);

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

            // fov perspective scaling
            let fov = area.width as f32 * 0.42;

            // 1. Draw tunnel stars
            for star in &stars {
                let raw_x = center_x + (star.x / star.z) * fov;
                let raw_y = center_y + (star.y / star.z) * fov * 0.55;

                // Apply gravitational lensing / warping if black hole is on screen
                let mut sx = raw_x;
                let mut sy = raw_y;

                if has_black_hole {
                    let dx = sx - bh_x;
                    let dy = (sy - bh_y) * 2.2;
                    let d = (dx*dx + dy*dy).sqrt();

                    if d > bh_r_event {
                        // Deflection warp factor
                        let deflection = 15.0 / d.max(1.0);
                        sx -= dx * deflection * 0.05;
                        sy -= dy * deflection * 0.05;
                    }
                }

                let px = sx as u16;
                let py = sy as u16;

                if px < area.width && py < area.height {
                    // Cull stars that fall into the Event Horizon
                    if has_black_hole {
                        let dx = sx - bh_x;
                        let dy = (sy - bh_y) * 2.2;
                        if (dx*dx + dy*dy).sqrt() <= bh_r_event {
                            continue;
                        }
                    }

                    let intensity = 1.0 - (star.z / 100.0);
                    let color_val = (intensity * 255.0) as u8;
                    // Blue-shift: higher values for blue
                    let star_color = Color::Rgb((color_val / 2) as u8, color_val, 255);

                    // Dynamic star symbols based on depth
                    let symbol = if star.z < 12.0 {
                        'O'
                    } else if star.z < 35.0 {
                        'o'
                    } else {
                        star.ch
                    };

                    let cell = &mut buf[(px, py)];
                    cell.set_symbol(&symbol.to_string());
                    cell.set_style(Style::default().fg(star_color));
                }
            }

            // 2. Draw zoom letters
            for zl in &zoom_letters {
                let raw_x = center_x + (zl.x / zl.z) * fov;
                let raw_y = center_y + (zl.y / zl.z) * fov * 0.55;

                let mut sx = raw_x;
                let mut sy = raw_y;

                if has_black_hole {
                    let dx = sx - bh_x;
                    let dy = (sy - bh_y) * 2.2;
                    let d = (dx*dx + dy*dy).sqrt();

                    if d > bh_r_event {
                        let deflection = 18.0 / d.max(1.0);
                        sx -= dx * deflection * 0.05;
                        sy -= dy * deflection * 0.05;
                    }
                }

                let px = sx as u16;
                let py = sy as u16;

                if px < area.width && py < area.height {
                    if has_black_hole {
                        let dx = sx - bh_x;
                        let dy = (sy - bh_y) * 2.2;
                        if (dx*dx + dy*dy).sqrt() <= bh_r_event {
                            continue;
                        }
                    }

                    let cell = &mut buf[(px, py)];
                    cell.set_symbol(&zl.ch.to_string());
                    
                    let mut style = Style::default().fg(zl.color);
                    if zl.z < 25.0 {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    cell.set_style(style);
                }
            }

            // 3. Draw static Logo at the center/far end of the tunnel
            let logo_x = area.width.saturating_sub(logo_width) / 2;
            let logo_y = area.height.saturating_sub(logo_height) / 2;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                let raw_y = logo_y + y_offset as u16;
                if raw_y >= area.height { continue; }

                for (x_offset, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let raw_x = logo_x + x_offset as u16;
                    if raw_x >= area.width { continue; }

                    let mut sx = raw_x as f32;
                    let mut sy = raw_y as f32;

                    if has_black_hole {
                        let dx = sx - bh_x;
                        let dy = (sy - bh_y) * 2.2;
                        let d = (dx*dx + dy*dy).sqrt();

                        if d > bh_r_event {
                            let deflection = 24.0 / d.max(1.0);
                            sx -= dx * deflection * 0.05;
                            sy -= dy * deflection * 0.05;
                        }
                    }

                    let px = sx as u16;
                    let py = sy as u16;

                    if px < area.width && py < area.height {
                        if has_black_hole {
                            let dx = sx - bh_x;
                            let dy = (sy - bh_y) * 2.2;
                            if (dx*dx + dy*dy).sqrt() <= bh_r_event {
                                continue;
                            }
                        }

                        let cell = &mut buf[(px, py)];
                        cell.set_symbol(&ch.to_string());
                        
                        // Deep cyan-purple style for far end logo
                        cell.set_style(Style::default()
                            .fg(Color::Rgb(0, 150, 180))
                            .add_modifier(Modifier::BOLD));
                    }
                }
            }

            // 4. Draw passing Black Hole
            if has_black_hole {
                for y in 0..area.height {
                    for x in 0..area.width {
                        let dx = x as f32 - bh_x;
                        let dy = (y as f32 - bh_y) * 2.2;
                        let d = (dx*dx + dy*dy).sqrt();

                        if d <= bh_r_event {
                            let cell = &mut buf[(x, y)];
                            cell.set_symbol(" ");
                            cell.set_style(Style::default().bg(Color::Black));
                        } else if d <= bh_r_event + 1.8 {
                            // Swirling gravitational red edge
                            let cell = &mut buf[(x, y)];
                            cell.set_symbol(".");
                            cell.set_style(Style::default().fg(Color::Rgb(200, 30, 0)));
                        }
                    }
                }
            }
        })?;

        let frame_time = Duration::from_millis(16);
        let elapsed_time = now.elapsed();
        if let Some(remaining) = frame_time.checked_sub(elapsed_time) {
            std::thread::sleep(remaining);
        }
    }
}
