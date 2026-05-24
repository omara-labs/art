#!/usr/bin/env bash
# Generate Omara Boot Theme Assets
# Requires: ImageMagick v7+

set -euo pipefail

# =============================================================================
# DIRECTORIES
# =============================================================================

THEME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(dirname "$THEME_DIR")/brand"
OUTPUT_DIR="$THEME_DIR/build"
INSTALL_DIR="/usr/share/plymouth/themes/omara-boot"

# =============================================================================
# CONFIGURATION
# =============================================================================

RESOLUTION="1024x1024"
FONT="JetBrains-Mono"

# Colors
RED="#ff5555"
RED_BRIGHT="#ff0000"
STAR_COLOR="#ffffff80"  # Semi-transparent white
BACKGROUND="#000000"    # Black

# Files
ASCII_FILE="$SOURCE_DIR/omara-ascii.txt"
BACKGROUND_IMG="$OUTPUT_DIR/background.png"
WATERMARK="$OUTPUT_DIR/watermark.png"

# =============================================================================
# FUNCTIONS
# =============================================================================

# Auto-calculate font size for 55% of image width
calculate_font_size() {
    local img_width=$(echo "$RESOLUTION" | cut -d'x' -f1)
    local target_width=$(echo "$img_width * 0.55" | bc)
    local char_count=60
    echo "$(echo "scale=0; $target_width / $char_count * 0.8 * 48" | bc)" | cut -d'.' -f1
}

# Generate starfield background
generate_background() {
    local width=$(echo "$RESOLUTION" | cut -d'x' -f1)
    local height=$(echo "$RESOLUTION" | cut -d'x' -f2)
    
    echo "✨ Generating starfield background..."
    
    magick -size "$RESOLUTION" xc:"$BACKGROUND" "$BACKGROUND_IMG"
    
    # Add random stars (160 like bounce screensaver)
    for i in $(seq 1 160); do
        local x=$((RANDOM % width))
        local y=$((RANDOM % height))
        local opacity=$((RANDOM % 60 + 40))  # 40-100 as percentage
        
        local star_type=$((RANDOM % 3))
        local char=""
        case $star_type in
            0) char="✦" ;;
            1) char="•" ;;
            *) char="·" ;;
        esac
        
        # Convert hex color to rgba for ImageMagick
        local r=$(printf "%d" "0x${STAR_COLOR:1:2}")
        local g=$(printf "%d" "0x${STAR_COLOR:3:2}")
        local b=$(printf "%d" "0x${STAR_COLOR:5:2}")
        local alpha=$(echo "scale=2; $opacity / 100" | bc)
        local star_size=$((RANDOM % 3 + 1))  # 1-3px radius
        
        magick "$BACKGROUND_IMG" \
            -fill "rgba($r,$g,$b,$alpha)" \
            -draw "circle $x,$y $((x + star_size)),$y" \
            "$BACKGROUND_IMG"
    done
}

# Generate watermark from ASCII art
generate_watermark() {
    local fontsize=$(calculate_font_size)
    echo "🎨 Generating watermark (font size: $fontsize)..."
    
    local tmpfile=$(mktemp)
    cat "$ASCII_FILE" > "$tmpfile"
    
    magick -size "$RESOLUTION" xc:none \
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
    local y_pos=40
    
    echo "🎬 Generating $frames throbber frames (back-and-forth dots)..."
    
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
        
        magick -size "${size}x${size}" xc:none "$filename"
        
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

mkdir -p "$OUTPUT_DIR"

generate_background
generate_watermark
generate_throbber

echo ""
echo "✅ Done! Assets generated in $OUTPUT_DIR"
echo ""
echo "To install:"
echo "  sudo cp -r $OUTPUT_DIR/* $INSTALL_DIR/"
echo "  sudo plymouth-set-default-theme omara-boot"
echo "  sudo dracut --force"
