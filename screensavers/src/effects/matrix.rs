use rand::Rng;

/// Character pool for dense Omara Matrix rain.
/// Primary characters come from the Omara logo.
const OMARA_CHARS: &str = "OMARA";
const EXTRA_SYMBOLS: &str = ":.*\"=+-><|/\\01";

#[derive(Clone)]
pub struct MatrixDrop {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub length: u16,
    pub chars: Vec<char>,
}

pub struct MatrixRain {
    pub drops: Vec<MatrixDrop>,
    pub width: u16,
    pub height: u16,
}

pub fn create_dense_matrix_rain(width: u16, height: u16) -> MatrixRain {
    let mut rng = rand::rng();
    let mut drops = Vec::new();

    // Build character pool (Omara letters + classic Matrix symbols)
    let char_pool: Vec<char> = OMARA_CHARS
        .chars()
        .chain(EXTRA_SYMBOLS.chars())
        .collect();

    // Create very dense rain — many drops per column
    for x in 0..width {
        // Primary long drop (main rain column)
        drops.push(MatrixDrop {
            x: x as f32,
            y: rng.random_range(-(height as f32) * 1.2..0.0),
            speed: rng.random_range(0.65..1.85),
            length: rng.random_range(12..28),
            chars: char_pool.clone(),
        });

        // Extra drops for high density (this is what makes it overwhelming)
        if rng.random::<f64>() < 0.72 {
            drops.push(MatrixDrop {
                x: x as f32,
                y: rng.random_range(-(height as f32) * 1.6..-(height as f32) * 0.4),
                speed: rng.random_range(0.55..1.65),
                length: rng.random_range(7..18),
                chars: char_pool.clone(),
            });
        }

        // Occasional third drop for extreme density
        if rng.random::<f64>() < 0.35 {
            drops.push(MatrixDrop {
                x: x as f32,
                y: rng.random_range(-(height as f32) * 2.0..-(height as f32) * 0.8),
                speed: rng.random_range(0.7..1.4),
                length: rng.random_range(5..14),
                chars: char_pool.clone(),
            });
        }
    }

    MatrixRain {
        drops,
        width,
        height,
    }
}

impl MatrixRain {
    pub fn tick(&mut self) {
        for drop in &mut self.drops {
            drop.y += drop.speed;

            // Reset drop when it goes off screen
            if drop.y > self.height as f32 + drop.length as f32 {
                let mut rng = rand::rng();
                drop.y = rng.random_range(-(drop.length as f32) * 2.0..-5.0);
                drop.speed = rng.random_range(0.6..1.8);
            }
        }
    }
}
