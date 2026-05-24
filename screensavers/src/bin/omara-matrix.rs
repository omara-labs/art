// omara-matrix - Dense Matrix-style rain using Omara characters

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    widgets::Block,
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

use omara_screensavers::effects::matrix::{create_dense_matrix_rain, MatrixRain};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    execute!(terminal.backend_mut(), crossterm::cursor::Hide)?;

    let result = run(&mut terminal);

    execute!(
        terminal.backend_mut(),
        crossterm::cursor::Show,
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;

    result
}

fn run<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal.size()?;
    let mut rain = create_dense_matrix_rain(size.width, size.height);
    let start = Instant::now();
    let mut last = Instant::now();

    loop {
        if event::poll(Duration::from_millis(8))? {
            if matches!(event::read()?, Event::Key(_) | Event::Mouse(_)) {
                break;
            }
        }

        // Auto-exit after ~45 seconds for testing
        if start.elapsed() > Duration::from_secs(45) {
            break;
        }

        rain.tick();

        terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::default().style(Style::default().bg(Color::Black)),
                area,
            );

            // Draw dense rain with fading trails
            for drop in &rain.drops {
                let x = drop.x.round() as u16;
                if x >= area.width {
                    continue;
                }

                for i in 0..drop.length {
                    let y = (drop.y - i as f32).round() as isize;
                    if y < 0 || y >= area.height as isize {
                        continue;
                    }

                    let ch = drop.chars[i as usize % drop.chars.len()];

                    // Brightness fades down the trail
                    let brightness = if i == 0 {
                        230u8
                    } else {
                        ((180 - (i as u16 * 9)).max(30)) as u8
                    };

                    let color = Color::Rgb(0, brightness, 60);

                    let para = ratatui::widgets::Paragraph::new(ch.to_string())
                        .style(Style::default().fg(color));
                    f.render_widget(para, ratatui::layout::Rect::new(x, y as u16, 1, 1));
                }
            }
        })?;

        let now = Instant::now();
        if let Some(d) = Duration::from_millis(16).checked_sub(now.duration_since(last)) {
            std::thread::sleep(d);
        }
        last = now;
    }

    Ok(())
}

