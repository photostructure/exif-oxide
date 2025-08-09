#!/bin/bash
# Update imports from snake_case to ExifTool-faithful _pm names
# This updates existing code to use the new naming convention

set -e

echo "=== Updating Generated Module Imports ==="
echo "This script will update all imports from snake_case to _pm format"
echo

# Define the mappings
declare -A MAPPINGS=(
    ["apple"]="Apple_pm"
    ["canon"]="Canon_pm"
    ["canon_custom"]="CanonCustom_pm"
    ["canon_raw"]="CanonRaw_pm"
    ["casio"]="Casio_pm"
    ["dji"]="DJI_pm"
    ["dng"]="DNG_pm"
    ["exif"]="Exif_pm"
    ["exif_tool"]="ExifTool_pm"
    ["fuji_film"]="FujiFilm_pm"
    ["geo_tiff"]="GeoTiff_pm"
    ["gimp"]="GIMP_pm"
    ["go_pro"]="GoPro_pm"
    ["gps"]="GPS_pm"
    ["h264"]="H264_pm"
    ["hp"]="HP_pm"
    ["iptc"]="IPTC_pm"
    ["jpeg"]="JPEG_pm"
    ["jpeg2000"]="Jpeg2000_pm"
    ["kodak"]="Kodak_pm"
    ["kyocera_raw"]="KyoceraRaw_pm"
    ["mac_os"]="MacOS_pm"
    ["matroska"]="Matroska_pm"
    ["microsoft"]="Microsoft_pm"
    ["mie"]="MIE_pm"
    ["minolta_raw"]="MinoltaRaw_pm"
    ["motorola"]="Motorola_pm"
    ["nikon"]="Nikon_pm"
    ["nintendo"]="Nintendo_pm"
    ["ogg"]="Ogg_pm"
    ["olympus"]="Olympus_pm"
    ["panasonic"]="Panasonic_pm"
    ["panasonic_raw"]="PanasonicRaw_pm"
    ["pentax"]="Pentax_pm"
    ["photoshop"]="Photoshop_pm"
    ["png"]="PNG_pm"
    ["quick_time"]="QuickTime_pm"
    ["red"]="Red_pm"
    ["ricoh"]="Ricoh_pm"
    ["riff"]="RIFF_pm"
    ["samsung"]="Samsung_pm"
    ["sanyo"]="Sanyo_pm"
    ["sigma"]="Sigma_pm"
    ["sigma_raw"]="SigmaRaw_pm"
    ["sony"]="Sony_pm"
    ["sony_idc"]="SonyIDC_pm"
    ["vorbis"]="Vorbis_pm"
    ["xmp"]="XMP_pm"
)

# Check if we're in dry-run mode
DRY_RUN=false
if [ "$1" = "--dry-run" ]; then
    DRY_RUN=true
    echo "DRY RUN MODE - No changes will be made"
    echo "================================================"
    echo
fi

# Function to execute or preview commands
execute_cmd() {
    if [ "$DRY_RUN" = true ]; then
        echo "[DRY RUN] $@"
    else
        "$@"
    fi
}

# Sort keys by length (longest first) to avoid partial replacements
# This is important so "panasonic_raw" is replaced before "panasonic"
SORTED_KEYS=($(
    for key in "${!MAPPINGS[@]}"; do
        echo "$key"
    done | awk '{ print length, $0 }' | sort -rn | cut -d' ' -f2-
))

echo "Will update imports for ${#MAPPINGS[@]} modules"
echo

# Update each module's imports
for old_name in "${SORTED_KEYS[@]}"; do
    new_name="${MAPPINGS[$old_name]}"
    
    echo "Updating: $old_name → $new_name"
    
    # Pattern variations to handle (no word boundary needed with length-sorted replacements):
    # - generated::module::
    # - generated::module;
    # - generated::module,  (in use statements with multiple imports)
    # - generated::module }  (at end of use statement)
    
    # Find files containing the old import pattern
    files=$(rg -l "generated::${old_name}" src/ tests/ 2>/dev/null || true)
    
    if [ -n "$files" ]; then
        file_count=$(echo "$files" | wc -l)
        echo "  Found in $file_count file(s)"
        
        # Update the imports
        echo "$files" | while read -r file; do
            if [ -n "$file" ]; then
                if [ "$DRY_RUN" = true ]; then
                    echo "  [DRY RUN] Would update: $file"
                else
                    # Use sd to replace the patterns - since we're sorted by length,
                    # we can safely replace all occurrences
                    sd "generated::${old_name}::" "generated::${new_name}::" "$file"
                    sd "generated::${old_name};" "generated::${new_name};" "$file"
                    sd "generated::${old_name}," "generated::${new_name}," "$file"
                    sd "generated::${old_name} }" "generated::${new_name} }" "$file"
                fi
            fi
        done
    else
        echo "  No files found with this import"
    fi
done

echo
echo "=== Import Update Complete ==="

if [ "$DRY_RUN" = true ]; then
    echo
    echo "This was a dry run. To apply changes, run without --dry-run:"
    echo "  ./scripts/update_generated_imports.sh"
else
    echo
    echo "Next steps:"
    echo "1. Update codegen/src/strategies/output_locations.rs to generate _pm names"
    echo "2. Run: make codegen"
    echo "3. Run: cargo check"
    echo "4. Commit all changes"
fi