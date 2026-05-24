// omara-unstable - Fully self-contained screensaver.
// Chaotic explosion outward -> dramatic snap back. Transitions from Orange -> Purple.

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

struct Particle {
    home_x: f32,
    home_y: f32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ch: char,
    glow: f32,
    snapped: bool,
}

struct Star {
    x: f32,
    y: f32,
    phase: f32,
    ch: char,
}

#[derive(PartialEq)]
enum Phase {
    Assembled,
    Exploding,
    Chaos,
    SnapBack,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_unstable(&mut terminal);

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

fn run_unstable<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    let mut particles: Vec<Particle> = Vec::new();
    let mut stars: Vec<Star> = Vec::new();

    let mut phase = Phase::Assembled;
    let mut phase_timer = Instant::now();
    let mut ambient_flash = 0.0f32;

    let mut last_width = 0;
    let mut last_height = 0;

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

        // Handle resizing and initializing logo particles
        if width != last_width || height != last_height {
            particles.clear();
            stars.clear();

            let logo_x = width.saturating_sub(logo_width) / 2;
            let logo_y = height.saturating_sub(logo_height) / 2;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                for (x_offset, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        let hx = (logo_x + x_offset as u16) as f32;
                        let hy = (logo_y + y_offset as u16) as f32;
                        particles.push(Particle {
                            home_x: hx,
                            home_y: hy,
                            x: hx,
                            y: hy,
                            vx: 0.0,
                            vy: 0.0,
                            ch,
                            glow: 0.0,
                            snapped: true,
                        });
                    }
                }
            }

            // Create background stars
            let target_stars = (width as usize * height as usize / 40).clamp(20, 100);
            for i in 0..target_stars {
                stars.push(Star {
                    x: rng.random::<f32>(),
                    y: rng.random::<f32>(),
                    phase: rng.random::<f32>() * std::f32::consts::TAU,
                    ch: if i % 8 == 0 { '✦' } else if i % 3 == 0 { '•' } else { '.' },
                });
            }

            phase = Phase::Assembled;
            phase_timer = Instant::now();
            ambient_flash = 0.0;
            last_width = width;
            last_height = height;
        }

        // Decay ambient flash
        if ambient_flash > 0.0 {
            ambient_flash -= delta * 3.5;
            if ambient_flash < 0.0 {
                ambient_flash = 0.0;
            }
        }

        // Particle dynamics update based on phase
        match phase {
            Phase::Assembled => {
                // Keep particles locked home and cool down their glow
                for p in &mut particles {
                    p.x = p.home_x;
                    p.y = p.home_y;
                    p.vx = 0.0;
                    p.vy = 0.0;
                    p.snapped = true;
                    if p.glow > 0.0 {
                        p.glow -= delta * 1.5;
                    }
                }

                if phase_timer.elapsed() > Duration::from_secs(5) {
                    phase = Phase::Exploding;
                }
            }
            Phase::Exploding => {
                let center_x = width as f32 / 2.0;
                let center_y = height as f32 / 2.0;

                // Push all particles outwards from center
                for p in &mut particles {
                    let mut dx = p.x - center_x;
                    let mut dy = (p.y - center_y) * 2.2; // vertical scale for aspect ratio

                    // Avoid division by zero
                    if dx.abs() < 0.1 && dy.abs() < 0.1 {
                        dx = rng.random::<f32>() * 2.0 - 1.0;
                        dy = rng.random::<f32>() * 2.0 - 1.0;
                    }

                    let angle = dy.atan2(dx);
                    // Add a tiny random dispersion angle
                    let disp = rng.random::<f32>() * 0.4 - 0.2;
                    let speed = rng.random::<f32>() * 22.0 + 16.0;

                    p.vx = speed * (angle + disp).cos();
                    p.vy = speed * (angle + disp).sin() * 0.48; // scale down vertical speed
                    p.glow = 1.0;
                    p.snapped = false;
                }

                ambient_flash = 1.0; // trigger background flash
                phase = Phase::Chaos;
                phase_timer = Instant::now();
            }
            Phase::Chaos => {
                // Let particles drift, bounce off boundaries, and experience drag
                for p in &mut particles {
                    p.x += p.vx * delta;
                    p.y += p.vy * delta;

                    // Bounce off boundaries with a small speed loss
                    let bounce_loss = 0.72;
                    if p.x < 0.0 {
                        p.x = 0.0;
                        p.vx = -p.vx * bounce_loss;
                    } else if p.x >= width as f32 {
                        p.x = width as f32 - 1.0;
                        p.vx = -p.vx * bounce_loss;
                    }

                    if p.y < 0.0 {
                        p.y = 0.0;
                        p.vy = -p.vy * bounce_loss;
                    } else if p.y >= height as f32 {
                        p.y = height as f32 - 1.0;
                        p.vy = -p.vy * bounce_loss;
                    }

                    // Apply gentle drag
                    p.vx *= 1.0 - 0.85 * delta;
                    p.vy *= 1.0 - 0.85 * delta;

                    // Slowly decay glow
                    if p.glow > 0.1 {
                        p.glow -= delta * 0.15;
                    }
                }

                if phase_timer.elapsed() > Duration::from_secs(5) {
                    phase = Phase::SnapBack;
                    phase_timer = Instant::now();
                }
            }
            Phase::SnapBack => {
                let mut all_snapped = true;

                for p in &mut particles {
                    if p.snapped {
                        // Stay locked in
                        p.x = p.home_x;
                        p.y = p.home_y;
                        p.vx = 0.0;
                        p.vy = 0.0;
                        if p.glow > 0.0 {
                            p.glow -= delta * 1.5;
                        }
                        continue;
                    }

                    all_snapped = false;

                    // Calculate vector back home
                    let dx = p.home_x - p.x;
                    let dy = p.home_y - p.y;
                    let dist = (dx*dx + dy*dy).sqrt();

                    if dist < 0.5 {
                        // Close enough - snap in place!
                        p.x = p.home_x;
                        p.y = p.home_y;
                        p.vx = 0.0;
                        p.vy = 0.0;
                        p.glow = 1.5; // lock-in flash
                        p.snapped = true;
                    } else {
                        // Apply strong spring pull force towards home
                        let spring_strength = 20.0;
                        p.vx += dx * spring_strength * delta;
                        p.vy += dy * spring_strength * delta;

                        // High drag/damping during snap-back to prevent crazy oscillation
                        let damping = 4.2;
                        p.vx *= 1.0 - damping * delta;
                        p.vy *= 1.0 - damping * delta;

                        p.x += p.vx * delta;
                        p.y += p.vy * delta;
                    }
                }

                if all_snapped {
                    phase = Phase::Assembled;
                    phase_timer = Instant::now();
                }
            }
        }

        // Render
        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            // 1. Draw background
            // If ambient flash is active, background has a very deep red/orange tint
            let bg_r = (ambient_flash * 20.0) as u8;
            let bg_g = (ambient_flash * 4.0) as u8;
            let bg_b = 0;

            for y in 0..area.height {
                for x in 0..area.width {
                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));
                }
            }

            // 2. Draw background stars
            for star in &stars {
                let sx = (star.x * area.width as f32) as u16;
                let sy = (star.y * area.height as f32) as u16;

                if sx < area.width && sy < area.height {
                    let sparkle = ((now.duration_since(phase_timer).as_secs_f32() * 2.0 + star.phase).sin() + 1.0) * 0.5;
                    
                    // Stars tint slightly orange during explosion flash, else dim purple/grey
                    let mut r = (50.0 + sparkle * 80.0) as u8;
                    let mut g = (50.0 + sparkle * 80.0) as u8;
                    let mut b = (65.0 + sparkle * 75.0) as u8;

                    if ambient_flash > 0.05 {
                        r = r.saturating_add((ambient_flash * 150.0) as u8);
                        g = g.saturating_add((ambient_flash * 50.0) as u8);
                        b = b.saturating_sub((ambient_flash * 40.0) as u8);
                    }

                    let cell = &mut buf[(sx, sy)];
                    cell.set_symbol(&star.ch.to_string());
                    cell.set_style(Style::default().fg(Color::Rgb(r, g, b)));
                }
            }

            // 3. Draw particles
            // Calculate center of logo to gauge distance during chaos/snap back
            let center_x = area.width as f32 / 2.0;
            let center_y = area.height as f32 / 2.0;
            let max_possible_dist = (center_x*center_x + center_y*center_y).sqrt().max(1.0);

            for p in &particles {
                let px = p.x as u16;
                let py = p.y as u16;

                if px < area.width && py < area.height {
                    // Color transitions based on phase and distance from center
                    // Near center/snapped = Vibrant Purple
                    // Far away/exploded = Orange / Red
                    let dx = p.x - center_x;
                    let dy = p.y - center_y;
                    let dist = (dx*dx + dy*dy).sqrt();

                    let color = if phase == Phase::Assembled {
                        // Settle into purple with cell glow influence
                        let glow_factor = p.glow.min(1.5);
                        if glow_factor > 1.0 {
                            // Snap lock flash (bright white-purple)
                            let extra = ((glow_factor - 1.0) * 2.0 * 255.0).min(255.0) as u8;
                            Color::Rgb(extra.max(160), 220, 255)
                        } else {
                            // Purple
                            let r = (110 + (glow_factor * 110.0) as u16).min(255) as u8;
                            let b = (210 + (glow_factor * 45.0) as u16).min(255) as u8;
                            Color::Rgb(r, 20, b)
                        }
                    } else {
                        // Dynamic transition: distance ratio determines color
                        let ratio = (dist / max_possible_dist).min(1.0);
                        
                        // Far ratio -> Orange/Red (R:255, G:100, B:0)
                        // Close ratio -> Purple (R:140, G:20, B:240)
                        let r = (255.0 * ratio + 140.0 * (1.0 - ratio)) as u8;
                        let g = (110.0 * ratio + 20.0 * (1.0 - ratio)) as u8;
                        let b = (240.0 * (1.0 - ratio)) as u8;
                        
                        Color::Rgb(r, g, b)
                    };

                    let cell = &mut buf[(px, py)];
                    cell.set_symbol(&p.ch.to_string());
                    
                    let mut style = Style::default().fg(color);
                    if phase == Phase::Assembled || p.glow > 0.8 {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    cell.set_style(style);
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
