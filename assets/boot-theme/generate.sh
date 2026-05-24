#!/usr/bin/env bash
# Generate Omara Boot Theme Assets
# Requires: ImageMagick v7+

set -euo pipefail

THEME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(dirname "$THEME_DIR")/brand"
OUTPUT_DIR="$THEME_DIR/build"  # Build locally, then install with sudo
RESOLUTION="1024x1024"
FONT="JetBrains-Mono"
INSTALL_DIR="/usr/share/plymouth/themes/omara-boot"

# =============================================================================
# CONFIGURATION
# =============================================================================

# Colors (from omara-configs theme)
RED="#ff5555"
RED_BRIGHT="#ff0000"
BACKGROUND="#00000000"  # Transparent

# ASCII art file
ASCII_FILE="$SOURCE_DIR/omara-ascii.txt"

# Output files
WATERMARK="$OUTPUT_DIR/watermark.png"

# ============================================================================= 
# FUNCTIONS
# =============================================================================

# Auto-calculate font size for 55% of image width
calculate_font_size() {
    local img_width=$(echo "$RESOLUTION" | cut -d'x' -f1)
    local target_width=$(echo "$img_width * 0.55" | bc)
    local char_count=60  # Approximate width of ASCII art
    # font_size = target_width / char_count * 0.8 * 48
    echo "$(echo "scale=0; $target_width / $char_count * 0.8 * 48" | bc)" | cut -d'.' -f1
}

# Generate watermark from ASCII art
generate_watermark() {
    local fontsize=$(calculate_font_size)
    echo "🎨 Generating watermark (font size: $fontsize)..."
    
    # Read ASCII art and create text image
    # Using label: protocol with explicit newlines
    local ascii_text=$(cat "$ASCII_FILE" | sed 's/$/\\n/g' | tr -d '\n')
    
    magick -size "$RESOLUTION" xc:"$BACKGROUND" \
        -font "$FONT" \
        -pointsize "$fontsize" \
        -fill "$RED" \
        -gravity center \
        -interline-spacing 20 \
        -annotate +0+0 "$ascii_text" \
        "$WATERMARK"
}

# Alternative watermark generation (more reliable)
generate_watermark_v2() {
    local fontsize=$(calculate_font_size)
    echo "🎨 Generating watermark (font size: $fontsize)..."
    
    # Create a temporary file with proper text
    local tmpfile=$(mktemp)
    cat "$ASCII_FILE" > "$tmpfile"
    
    magick -size "$RESOLUTION" xc:"$BACKGROUND" \
        -font "$FONT" \
        -pointsize "$fontsize" \
        -fill "$RED" \
        -gravity center \
        -interline-spacing 20 \
        -annotate +0+0 @"$tmpfile" \
        "$WATERMARK"
    
    rm -f "$tmpfile"
}

# Generate throbber frames (dots animation)
generate_throbber() {
    local frames=10
    local size=128
    local dot_size=24
    local spacing=20
    local dot_color="$RED"
    local active_dot_color="$RED_BRIGHT"
    local y_pos=40  # Vertically center between OMARA bottom and screen bottom
    
    echo "🎬 Generating $frames throbber frames (back-and-forth dots)..."
    
    # Patterns: back-and-forth animation
    local patterns=(
        "●ooooo"
        "o●oooo"
        "oo●ooo"
        "ooo●oo"
        "oooo●o"
        "oooo●o"
        "ooo●oo"
        "oo●ooo"
        "o●oooo"
        "●ooooo"
    )
    
    for i in $(seq 0 $((frames - 1))); do
        local pattern="${patterns[$i]}"
        local filename="$OUTPUT_DIR/throbber-$(printf '%04d' $((i+1))).png"
        
        # Create transparent canvas
        magick -size "${size}x${size}" xc:"$BACKGROUND" "$filename"
        
        # Calculate x positions for dots
        local total_width=$(( ${#pattern} * (dot_size + spacing) - spacing ))
        local start_x=$(( (size - total_width) / 2 ))
        
        for j in $(seq 0 $(( ${#pattern} - 1 ))); do
            local char="${pattern:$j:1}"
            local x=$(( start_x + j * (dot_size + spacing) ))
            local y=$y_pos
            
            if [[ "$char" == "●" ]]; then
                color="$active_dot_color"
            else
                color="$dot_color"
            fi
            
            # Draw filled circle
            magick "$filename" \
                -fill "$color" \
                -draw "circle $x,$y $((x + dot_size)),$y" \
                "$filename"
        done
    done
}

# =============================================================================
# MAIN
# =============================================================================

echo "🚀 Generating Omara Boot Theme Assets"
echo "=================================="

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate assets
generate_watermark_v2
generate_throbber

echo ""
echo "✅ Done! Assets generated in $OUTPUT_DIR"
echo ""
echo "To install:"
echo "  sudo cp -r $OUTPUT_DIR/* $INSTALL_DIR/"
echo "  sudo plymouth-set-default-theme omara-boot"
echo "  sudo dracut --force"
