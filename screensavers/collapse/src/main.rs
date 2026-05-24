// omara-collapse - Fully self-contained black hole collapse screensaver.
// High-energy accretion disk collapse, relativistic supernova jet, and logo rebirth.

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
    x: f32,
    y: f32,
    home_x: f32,
    home_y: f32,
    ch: char,
    stripped: bool,
    orbit_angle: f32,
    orbit_radius: f32,
    swallowed: bool,
    glow: f32,
}

struct GasParticle {
    angle: f32,
    radius: f32,
    speed: f32,
    ch: char,
    color: Color,
}

struct JetParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    ch: char,
    color: Color,
}

#[derive(PartialEq, Clone, Copy)]
enum Phase {
    Orbiting,
    Imploding,
    SupernovaJet,
    Rebuilding,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_collapse(&mut terminal);

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

fn run_collapse<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut logo_cells: Vec<LogoCell> = Vec::new();
    let mut gas_particles: Vec<GasParticle> = Vec::new();
    let mut jet_particles: Vec<JetParticle> = Vec::new();

    let mut phase = Phase::Orbiting;
    let mut phase_timer = Instant::now();
    let mut last_frame = Instant::now();
    let mut last_width = 0;
    let mut last_height = 0;

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

        // Initialize / Resize handling
        if width != last_width || height != last_height {
            logo_cells.clear();
            gas_particles.clear();
            jet_particles.clear();

            let logo_x = width.saturating_sub(logo_width) / 2;
            let logo_y = height.saturating_sub(logo_height) / 2;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                for (x_offset, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        let hx = (logo_x + x_offset as u16) as f32;
                        let hy = (logo_y + y_offset as u16) as f32;
                        logo_cells.push(LogoCell {
                            x: hx,
                            y: hy,
                            home_x: hx,
                            home_y: hy,
                            ch,
                            stripped: false,
                            orbit_angle: 0.0,
                            orbit_radius: 0.0,
                            swallowed: false,
                            glow: 0.0,
                        });
                    }
                }
            }

            phase = Phase::Orbiting;
            phase_timer = Instant::now();
            last_width = width;
            last_height = height;
        }

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;

        // ACCRETION SPIRAL ORBIT MATH
        let elapsed = phase_timer.elapsed().as_secs_f32();
        let bh_angle = elapsed * 1.5;
        // Spiral radius shrinks from 24 down to 0 over 8 seconds
        let bh_radius = (24.0 - elapsed * 3.0).max(0.0);

        let bh_x = center_x + bh_angle.cos() * bh_radius * 2.0;
        let bh_y = center_y + bh_angle.sin() * bh_radius;

        match phase {
            Phase::Orbiting => {
                // The black hole spirals in. Rips nearby logo characters.
                for cell in &mut logo_cells {
                    if cell.swallowed { continue; }

                    if !cell.stripped {
                        let dx = (cell.x - bh_x) * 0.55;
                        let dy = cell.y - bh_y;
                        let dist = (dx*dx + dy*dy).sqrt();

                        // Rip threshold
                        if dist < 6.5 {
                            cell.stripped = true;
                            cell.orbit_radius = dist;
                            cell.orbit_angle = dy.atan2(dx);
                        }
                    } else {
                        // Swirl cell around the moving black hole
                        cell.orbit_radius -= 3.2 * delta;
                        cell.orbit_angle += (6.5 / cell.orbit_radius.max(0.8)) * delta;

                        cell.x = bh_x + cell.orbit_angle.cos() * cell.orbit_radius * 2.0;
                        cell.y = bh_y + cell.orbit_angle.sin() * cell.orbit_radius;

                        if cell.orbit_radius < 0.6 {
                            cell.swallowed = true;
                        }
                    }
                }

                // Add swirling gas in the accretion disk around the black hole
                if gas_particles.len() < 40 && bh_radius > 1.0 {
                    gas_particles.push(GasParticle {
                        angle: rng.random::<f32>() * std::f32::consts::TAU,
                        radius: 5.0 + rng.random::<f32>() * 5.0,
                        speed: rng.random::<f32>() * 4.0 + 3.0,
                        ch: if rng.random_bool(0.3) { '*' } else { '.' },
                        color: Color::Rgb(255, (100.0 + rng.random::<f32>() * 120.0) as u8, 0),
                    });
                }

                // Update gas particles
                for g in &mut gas_particles {
                    g.radius -= 3.0 * delta;
                    g.angle += (g.speed / g.radius.max(0.8)) * delta;
                }
                gas_particles.retain(|g| g.radius > 0.6);

                if bh_radius <= 0.0 {
                    phase = Phase::Imploding;
                    phase_timer = Instant::now();
                }
            }
            Phase::Imploding => {
                // Black hole is now locked at the center. Pull all remaining letters.
                let mut all_swallowed = true;

                for cell in &mut logo_cells {
                    if cell.swallowed { continue; }
                    all_swallowed = false;

                    cell.stripped = true;
                    let dx = (cell.x - center_x) * 0.55;
                    let dy = cell.y - center_y;
                    cell.orbit_radius = (dx*dx + dy*dy).sqrt();
                    cell.orbit_angle = dy.atan2(dx);

                    // Pull rapidly
                    cell.orbit_radius -= 8.0 * delta;
                    cell.orbit_angle += (9.5 / cell.orbit_radius.max(0.8)) * delta;

                    cell.x = center_x + cell.orbit_angle.cos() * cell.orbit_radius * 2.0;
                    cell.y = center_y + cell.orbit_angle.sin() * cell.orbit_radius;

                    if cell.orbit_radius < 0.6 {
                        cell.swallowed = true;
                    }
                }

                // Swirl gas particles into the center
                if gas_particles.len() < 80 {
                    gas_particles.push(GasParticle {
                        angle: rng.random::<f32>() * std::f32::consts::TAU,
                        radius: 8.0 + rng.random::<f32>() * 12.0,
                        speed: rng.random::<f32>() * 6.0 + 4.0,
                        ch: if rng.random_bool(0.4) { '*' } else { '.' },
                        color: Color::Rgb(255, (80.0 + rng.random::<f32>() * 175.0) as u8, 0),
                    });
                }
                for g in &mut gas_particles {
                    g.radius -= 9.0 * delta;
                    g.angle += (g.speed / g.radius.max(0.8)) * delta;
                }
                gas_particles.retain(|g| g.radius > 0.6);

                if all_swallowed && gas_particles.is_empty() {
                    phase = Phase::SupernovaJet;
                    phase_timer = Instant::now();
                    
                    // Spawn horizontal supernova jet particles
                    jet_particles.clear();
                    for _ in 0..120 {
                        let left = rng.random_bool(0.5);
                        let vx = if left {
                            -(rng.random::<f32>() * 55.0 + 25.0)
                        } else {
                            rng.random::<f32>() * 55.0 + 25.0
                        };
                        let vy = rng.random::<f32>() * 8.0 - 4.0;
                        jet_particles.push(JetParticle {
                            x: center_x,
                            y: center_y,
                            vx,
                            vy,
                            life: 1.0,
                            ch: if rng.random_bool(0.3) { '=' } else if rng.random_bool(0.3) { '#' } else { '*' },
                            color: Color::Rgb(255, 255, 255),
                        });
                    }
                }
            }
            Phase::SupernovaJet => {
                // Double-sided horizontal energy jet
                for jp in &mut jet_particles {
                    jp.x += jp.vx * delta;
                    jp.y += jp.vy * delta;
                    jp.life -= delta * 0.7; // fade out

                    // Shift color from white-hot -> pink -> deep purple
                    let r = (jp.life * 255.0) as u8;
                    let g = (jp.life * 180.0 * 0.2) as u8;
                    let b = (100.0 + jp.life * 155.0).min(255.0) as u8;
                    jp.color = Color::Rgb(r, g, b);
                }
                jet_particles.retain(|jp| jp.life > 0.0 && jp.x >= 0.0 && jp.x < width as f32);

                if jet_particles.is_empty() || elapsed > 1.8 {
                    phase = Phase::Rebuilding;
                    phase_timer = Instant::now();

                    // Position logo cells at top of screen to drop down
                    for cell in &mut logo_cells {
                        cell.x = cell.home_x;
                        cell.y = 0.0; // drop from top
                        cell.stripped = false;
                        cell.swallowed = false;
                        cell.glow = 1.8; // bright reentry flash
                    }
                }
            }
            Phase::Rebuilding => {
                // Drop cells from top down, slowing down as they lock home
                let mut all_locked = true;

                for cell in &mut logo_cells {
                    let dy = cell.home_y - cell.y;
                    if dy.abs() > 0.08 {
                        all_locked = false;
                        // Spring-like pull downwards
                        cell.y += dy * 6.5 * delta;
                    } else {
                        cell.y = cell.home_y;
                        if cell.glow > 0.0 {
                            cell.glow -= delta * 1.5;
                        }
                    }
                }

                if all_locked {
                    phase = Phase::Orbiting;
                    phase_timer = Instant::now();
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

            // 1. Draw Singularity / Accretion Disk (only in Orbiting/Imploding)
            if phase == Phase::Orbiting || phase == Phase::Imploding {
                let bx = bh_x as u16;
                let by = bh_y as u16;

                // Accretion gas particles
                for g in &gas_particles {
                    let gx = (bh_x + g.angle.cos() * g.radius * 2.0) as u16;
                    let gy = (bh_y + g.angle.sin() * g.radius) as u16;

                    if gx < area.width && gy < area.height {
                        let cell = &mut buf[(gx, gy)];
                        cell.set_symbol(&g.ch.to_string());
                        cell.set_style(Style::default().fg(g.color));
                    }
                }

                // Black Hole Horizon
                if bx < area.width && by < area.height {
                    let cell = &mut buf[(bx, by)];
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(Color::Rgb(25, 5, 0))); // dark red interior glow
                }
            }

            // 2. Draw Jet Particles (only in SupernovaJet)
            if phase == Phase::SupernovaJet {
                for jp in &jet_particles {
                    let jx = jp.x as u16;
                    let jy = jp.y as u16;

                    if jx < area.width && jy < area.height {
                        let cell = &mut buf[(jx, jy)];
                        cell.set_symbol(&jp.ch.to_string());
                        let mut style = Style::default().fg(jp.color);
                        if jp.life > 0.7 {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        cell.set_style(style);
                    }
                }
            }

            // 3. Draw Logo Cells
            if phase != Phase::SupernovaJet {
                for cell in &logo_cells {
                    if cell.swallowed { continue; }

                    let cx = cell.x as u16;
                    let cy = cell.y as u16;

                    if cx < area.width && cy < area.height {
                        let c = &mut buf[(cx, cy)];
                        c.set_symbol(&cell.ch.to_string());

                        let style = if phase == Phase::Rebuilding {
                            let glow_factor = cell.glow.min(1.5);
                            if glow_factor > 1.0 {
                                let extra = ((glow_factor - 1.0) * 2.0 * 255.0).min(255.0) as u8;
                                Style::default().fg(Color::Rgb(extra, 255, 255)).add_modifier(Modifier::BOLD)
                            } else {
                                let r = (glow_factor * 100.0) as u8;
                                let g = (glow_factor * 180.0) as u8;
                                let b = (220.0 + glow_factor * 35.0) as u8;
                                Style::default().fg(Color::Rgb(r, g, b)).add_modifier(Modifier::BOLD)
                            }
                        } else if cell.stripped {
                            // Orbiting cells are hot orange/yellow
                            let ratio = (cell.orbit_radius / 20.0).min(1.0);
                            let r = 255;
                            let g = (100.0 + ratio * 140.0) as u8;
                            Style::default().fg(Color::Rgb(r, g, 0))
                        } else {
                            // Intact logo: standard purple
                            Style::default().fg(Color::Rgb(140, 20, 240)).add_modifier(Modifier::BOLD)
                        };

                        c.set_style(style);
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
