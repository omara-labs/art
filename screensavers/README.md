# Omara Screensavers

Collection of beautiful terminal screensavers for **Omara OS**, in the style of Omarchy.

## Location
All screensavers live here: `~/Projects/omara/screensavers/`

## Available Screensavers

| Binary            | Style                                      | Color Palette          | Vibe                     |
|-------------------|--------------------------------------------|------------------------|--------------------------|
| `omara-bounce`    | Classic DVD-style bouncing Omara logo      | Cycling (purple/cyan/yellow/green/pink) | **Finished & polished**  |
| `omara-matrix`    | Digital rain that settles into logo        | Green                  | Classic Omarchy          |
| `omara-beams`     | Sweeping vertical beams / spotlights       | Vibrant Purple         | Elegant, modern          |
| `omara-unstable`  | Chaotic explode outward → dramatic snap back | Orange → Purple      | Explosive & satisfying   |
| `omara-pour`      | Glyphs cascade/pour from above and build the logo | Cyan / Light Blue | Liquid, building         |

**`omara-bounce`** is currently the only screensaver considered complete and production-ready.  
More coming (Fire, Synthgrid, Swarm, Laser Etch, etc.).

## Building

```bash
cd ~/Projects/omara/screensavers
source "$HOME/.cargo/env"

# Build everything in release (recommended)
cargo build --release

# The binaries will appear in:
# target/release/omara-matrix
# target/release/omara-beams
```

## Usage (manual testing on GNOME)

```bash
cd ~/Projects/omara/screensavers

# Try them all (bounce is the finished DVD-style one)
./target/release/omara-bounce
./target/release/omara-matrix
./target/release/omara-beams
./target/release/omara-unstable
./target/release/omara-pour
```

You can also launch them inside your terminal explicitly:

```bash
kitty -e ~/Projects/omara/screensavers/target/release/omara-matrix
```

## Adding a New Screensaver

1. Create `src/bin/omara-mynewone.rs` (copy one of the existing as template).
2. Add the corresponding effect in `src/effects/mynewone.rs` if needed.
3. Register it in `Cargo.toml` under `[[bin]]`.
4. Rebuild.

## Future Integration (Niri / Omara OS)

These binaries are designed to be launched by `niri` idle detection via a small launcher script (similar to Omarchy), one per monitor, with the proper app-id/class.

Example launcher names you can create later:
- `omara-launch-screensaver-matrix`
- `omara-launch-screensaver-beams`

Then the user (or distro default) can pick which one they want as their idle screensaver.

## Branding

All savers respect `~/.config/omara/branding/screensaver.txt` for custom ASCII art.

**Canonical source**: `omara-art/assets/brand/omara.txt`

Default:

```
  ██████╗  ███╗   ███╗  █████╗  ██████╗   █████╗ 
 ██╔═══██╗ ████╗ ████║ ██╔══██╗ ██╔══██╗ ██╔══██╗
 ██║   ██║ ██╔████╔██║ ███████║ ██████╔╝ ███████║
 ██║   ██║ ██║╚██╔╝██║ ██╔══██║ ██╔══██╗ ██╔══██║
 ╚██████╔╝ ██║ ╚═╝ ██║ ██║  ██║ ██║  ██║ ██║  ██║
  ╚═════╝  ╚═╝     ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝
```
