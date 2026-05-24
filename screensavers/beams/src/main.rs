// omara-beams - Fully self-contained screensaver.
// Sweeping vertical spotlights and purple/blue/pink color gradients.

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

struct Star {
    x: f32,
    y: f32,
    phase: f32,
    ch: char,
}

struct DustParticle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

struct Spotlight {
    origin_x_ratio: f32,
    angle_center: f32,
    angle_amplitude: f32,
    speed: f32,
    phase_offset: f32,
    spread: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run_beams(&mut terminal);

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

fn run_beams<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logo = load_branding();
    let logo_lines: Vec<&str> = logo.lines().collect();
    let logo_height = logo_lines.len() as u16;
    let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(56) as u16;

    let mut rng = rand::rng();

    // Initialize empty collections; will be scaled to size on first frame
    let mut stars = Vec::new();
    let mut particles = Vec::new();
    let mut last_width = 0;
    let mut last_height = 0;

    let spotlights = vec![
        Spotlight {
            origin_x_ratio: 0.15,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.55,
            speed: 0.85,
            phase_offset: 0.0,
            spread: 0.15,
            color_r: 160.0,
            color_g: 20.0,
            color_b: 255.0, // Rich Purple-Blue
        },
        Spotlight {
            origin_x_ratio: 0.50,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.70,
            speed: 0.55,
            phase_offset: 2.5,
            spread: 0.12,
            color_r: 0.0,
            color_g: 130.0,
            color_b: 255.0, // Neon Electric Blue
        },
        Spotlight {
            origin_x_ratio: 0.85,
            angle_center: std::f32::consts::FRAC_PI_2,
            angle_amplitude: 0.60,
            speed: 1.05,
            phase_offset: 4.5,
            spread: 0.14,
            color_r: 255.0,
            color_g: 0.0,
            color_b: 130.0, // Bright Pink-Magenta
        },
    ];

    let start_time = Instant::now();
    let mut last_frame = Instant::now();

    loop {
        // Exit on key press or mouse event
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break Ok(());
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        let time = start_time.elapsed().as_secs_f32();

        let size = terminal.size()?;
        let width = size.width;
        let height = size.height;

        if width == 0 || height == 0 {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        // Dynamically scale star/particle counts to screen size
        if width != last_width || height != last_height {
            let area = width as usize * height as usize;

            // Star density: roughly 1 star per 32 characters, bounded nicely
            let target_stars = (area / 32).clamp(30, 300);
            if stars.len() < target_stars {
                while stars.len() < target_stars {
                    stars.push(Star {
                        x: rng.random::<f32>(),
                        y: rng.random::<f32>(),
                        phase: rng.random::<f32>() * std::f32::consts::TAU,
                        ch: if rng.random_bool(0.1) { '✦' } else if rng.random_bool(0.3) { '•' } else { '.' },
                    });
                }
            } else if stars.len() > target_stars {
                stars.truncate(target_stars);
            }

            // Dust density: roughly 1 particle per 24 characters, bounded nicely
            let target_particles = (area / 24).clamp(30, 250);
            if particles.len() < target_particles {
                while particles.len() < target_particles {
                    particles.push(DustParticle {
                        x: rng.random::<f32>(),
                        y: rng.random::<f32>(),
                        vx: rng.random::<f32>() * 0.04 - 0.02,
                        vy: -(rng.random::<f32>() * 0.025 + 0.015),
                    });
                }
            } else if particles.len() > target_particles {
                particles.truncate(target_particles);
            }

            last_width = width;
            last_height = height;
        }

        // Update dust particles
        for p in &mut particles {
            p.x += p.vx * delta;
            p.y += p.vy * delta;

            // wrap around normalized space (particles float up)
            if p.y < 0.0 {
                p.y = 1.0;
                p.x = rng.random::<f32>();
            }
            if p.x < 0.0 {
                p.x = 1.0;
            }
            if p.x > 1.0 {
                p.x = 0.0;
            }
        }

        terminal.draw(|f| {
            let area = f.area();
            let buf = f.buffer_mut();

            let y_origin = area.height as f32;
            let max_dist = y_origin * 1.6;

            // Pre-calculate angles for the spotlights
            let current_angles: Vec<f32> = spotlights.iter().map(|spot| {
                spot.angle_center + spot.angle_amplitude * (time * spot.speed + spot.phase_offset).sin()
            }).collect();

            // 1. Draw background and searchlight beams
            for y in 0..area.height {
                for x in 0..area.width {
                    let mut r = 0.0f32;
                    let mut g = 0.0f32;
                    let mut b = 0.0f32;

                    for (i, spot) in spotlights.iter().enumerate() {
                        let x_origin = spot.origin_x_ratio * area.width as f32;
                        let dx = (x as f32 - x_origin) * 0.55; // aspect ratio scaling
                        let dy = y_origin - y as f32;

                        if dy > 0.0 {
                            let angle = dy.atan2(dx);
                            let dist = (dx*dx + dy*dy).sqrt();
                            let current_angle = current_angles[i];
                            let mut da = angle - current_angle;
                            da = (da + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;

                            if da.abs() < spot.spread {
                                let angular_intensity = 1.0 - (da.abs() / spot.spread);
                                let dist_intensity = (1.0 - dist / max_dist).max(0.0);
                                // high-frequency beam texture wave
                                let wave = 0.88 + 0.12 * (dist * 0.28 - time * 14.0).sin();
                                let intensity = angular_intensity * dist_intensity * wave;

                                r += intensity * spot.color_r;
                                g += intensity * spot.color_g;
                                b += intensity * spot.color_b;
                            }
                        }
                    }

                    let r_val = r.min(255.0);
                    let g_val = g.min(255.0);
                    let b_val = b.min(255.0);

                    // Compute background color (dark rich glow of the beam)
                    let bg_r = (r_val * 0.15) as u8;
                    let bg_g = (g_val * 0.15) as u8;
                    let bg_b = (b_val * 0.15) as u8;

                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(" ");
                    cell.set_style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));
                }
            }

            // Helper lambda to look up light values at a coordinate
            let get_light_at = |cx: u16, cy: u16| -> (f32, f32, f32, f32) {
                let mut r = 0.0f32;
                let mut g = 0.0f32;
                let mut b = 0.0f32;
                let mut total_intensity = 0.0f32;

                for (i, spot) in spotlights.iter().enumerate() {
                    let x_origin = spot.origin_x_ratio * area.width as f32;
                    let dx = (cx as f32 - x_origin) * 0.55;
                    let dy = y_origin - cy as f32;

                    if dy > 0.0 {
                        let angle = dy.atan2(dx);
                        let dist = (dx*dx + dy*dy).sqrt();
                        let current_angle = current_angles[i];
                        let mut da = angle - current_angle;
                        da = (da + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;

                        if da.abs() < spot.spread {
                            let angular_intensity = 1.0 - (da.abs() / spot.spread);
                            let dist_intensity = (1.0 - dist / max_dist).max(0.0);
                            let wave = 0.88 + 0.12 * (dist * 0.28 - time * 14.0).sin();
                            let intensity = angular_intensity * dist_intensity * wave;

                            r += intensity * spot.color_r;
                            g += intensity * spot.color_g;
                            b += intensity * spot.color_b;
                            total_intensity += intensity;
                        }
                    }
                }
                (r.min(255.0), g.min(255.0), b.min(255.0), total_intensity.min(1.0))
            };

            // 2. Draw stars
            for star in &stars {
                let sx = (star.x * area.width as f32) as u16;
                let sy = (star.y * area.height as f32) as u16;

                if sx < area.width && sy < area.height {
                    let (_, _, _, intensity) = get_light_at(sx, sy);

                    let sparkle = ((time * 2.2 + star.phase).sin() + 1.0) * 0.5;
                    let final_brightness = sparkle * 0.4 + intensity * 0.6;

                    let star_r = (65.0 + final_brightness * 190.0) as u8;
                    let star_g = (65.0 + final_brightness * 190.0) as u8;
                    let star_b = (85.0 + final_brightness * 170.0) as u8;

                    let cell = &mut buf[(sx, sy)];
                    cell.set_symbol(&star.ch.to_string());
                    cell.set_style(cell.style().fg(Color::Rgb(star_r, star_g, star_b)));
                }
            }

            // 3. Draw dust particles
            for p in &particles {
                let px = (p.x * area.width as f32) as u16;
                let py = (p.y * area.height as f32) as u16;

                if px < area.width && py < area.height {
                    let (_, _, _, intensity) = get_light_at(px, py);

                    if intensity > 0.05 {
                        let ch = if intensity > 0.6 { '*' } else if intensity > 0.3 { '+' } else { '.' };
                        let p_r = (140.0 + intensity * 115.0).min(255.0) as u8;
                        let p_g = (100.0 + intensity * 155.0).min(255.0) as u8;
                        let p_b = (255.0) as u8;

                        let cell = &mut buf[(px, py)];
                        cell.set_symbol(&ch.to_string());
                        cell.set_style(cell.style().fg(Color::Rgb(p_r, p_g, p_b)));
                    }
                }
            }

            // 4. Draw logo
            let logo_x = area.width.saturating_sub(logo_width) / 2;
            let logo_y = area.height.saturating_sub(logo_height) / 2;

            for (y_offset, line) in logo_lines.iter().enumerate() {
                let cy = logo_y + y_offset as u16;
                if cy >= area.height { continue; }

                for (x_offset, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let cx = logo_x + x_offset as u16;
                    if cx >= area.width { continue; }

                    let (_, _, _, intensity) = get_light_at(cx, cy);

                    let style = if intensity > 0.05 {
                        let l_r = (90.0 + intensity * 165.0).min(255.0) as u8;
                        let l_g = (20.0 + intensity * 180.0).min(255.0) as u8;
                        let l_b = (120.0 + intensity * 135.0).min(255.0) as u8;
                        Style::default()
                            .fg(Color::Rgb(l_r, l_g, l_b))
                            .add_modifier(ratatui::style::Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Rgb(45, 20, 60))
                    };

                    let cell = &mut buf[(cx, cy)];
                    cell.set_symbol(&ch.to_string());
                    cell.set_style(style.bg(cell.style().bg.unwrap_or(Color::Black)));
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
