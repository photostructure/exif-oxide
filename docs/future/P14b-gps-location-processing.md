# Technical Project Plan: GPS and Location Processing

## Project Overview

- **Goal**: Implement comprehensive GPS coordinate processing and location metadata extraction
- **Problem**: GPS destination tags missing, GPS composite calculations incomplete, location metadata scattered across manufacturers

## Background & Context

- GPS data appears in multiple forms: EXIF GPS IFD, XMP location tags, MakerNotes location data
- ExifTool provides sophisticated GPS coordinate conversion and destination processing
- Location data critical for PhotoStructure's geotagging and organization features

## Technical Foundation

- **Key files**:
  - `src/gps/` - GPS coordinate processing (if exists)
  - `src/implementations/value_conv.rs` - GPS coordinate conversion
  - `src/exif/gps_ifd.rs` - GPS IFD processing
  - `src/composite_tags/implementations.rs` - GPS composite calculations
  - `third-party/exiftool/lib/Image/ExifTool/GPS.pm` - ExifTool GPS reference

## Work Completed (from P10a)

- ✅ Basic GPS coordinate extraction (GPSLatitude, GPSLongitude, GPSAltitude)
- ✅ GPS ToDegrees ValueConv implemented
- ✅ GPS coordinate PrintConv returns decimal degrees
- ✅ Rational array to decimal degree conversion working

## Remaining Tasks

### High Priority - Missing GPS Destination Tags

**From compatibility test failures:**

1. **GPSDestLatitude** (0x0014)
   - Currently missing from extraction
   - Same format as GPSLatitude but for destination coordinates
   - Must include GPSDestLatitudeRef processing

2. **GPSDestLongitude** (0x0016) 
   - Currently missing from extraction
   - Same format as GPSLongitude but for destination coordinates
   - Must include GPSDestLongitudeRef processing

3. **GPS Composite Calculations**
   ```rust
   // Missing from composite_tags/implementations.rs
   pub fn calculate_gps_position(lat: f64, lon: f64) -> String {
       // ExifTool format: "34.05223 -118.24368" (decimal degrees)
       format!("{:.5} {:.5}", lat, lon)
   }
   
   pub fn calculate_gps_datetime(date_stamp: &str, time_stamp: &[u32; 3]) -> String {
       // Combine GPSDateStamp + GPSTimeStamp to UTC format
       // ExifTool: "2023:07:15 14:30:25Z"
   }
   ```

### Medium Priority - Extended GPS Tags

4. **GPSProcessingMethod** (0x001B)
   - ASCII string with possible encoding prefix
   - Handle "GPS", "CELLID", "WLAN", "MANUAL" values

5. **GPSAreaInformation** (0x001C)
   - Location area information (often Unicode)
   - May contain city/region names

6. **GPSMapDatum** (0x0012)
   - Map datum reference (e.g., "WGS-84")
   - Important for coordinate system accuracy

### Advanced Priority - Manufacturer Location Extensions

7. **Canon GPS Integration**
   - Canon cameras may store additional GPS data in MakerNotes
   - Check Canon.pm for GPS-related tags

8. **Smartphone GPS Metadata**  
   - Modern smartphones store rich location data
   - XMP location tags may supplement EXIF GPS

9. **Panasonic/Olympus Location Tags**
   - Some cameras store location names in MakerNotes
   - City, Country tags in manufacturer-specific formats

## Implementation Strategy

### Phase 1: Complete Basic GPS Tags

**GPS Destination Tags**:
```rust
// Add to src/exif/gps_ifd.rs or appropriate GPS handler
pub const GPS_DEST_LATITUDE: u16 = 0x0014;
pub const GPS_DEST_LATITUDE_REF: u16 = 0x0015;  
pub const GPS_DEST_LONGITUDE: u16 = 0x0016;
pub const GPS_DEST_LONGITUDE_REF: u16 = 0x0017;

pub fn process_gps_destination_coordinates(
    dest_lat: &[u32; 6],     // [deg_num, deg_den, min_num, min_den, sec_num, sec_den]
    dest_lat_ref: &str,      // "N" or "S"
    dest_lon: &[u32; 6],     // Same format as latitude
    dest_lon_ref: &str,      // "E" or "W"
) -> (f64, f64) {
    // Same conversion logic as regular GPS coordinates
    let lat_decimal = convert_dms_to_decimal(dest_lat, dest_lat_ref);
    let lon_decimal = convert_dms_to_decimal(dest_lon, dest_lon_ref);
    (lat_decimal, lon_decimal)
}
```

### Phase 2: GPS Composite Tags

**GPSPosition Composite**:
```rust
// Add to src/composite_tags/implementations.rs
pub fn compute_gps_position(available_tags: &HashMap<String, TagValue>) -> Option<String> {
    let lat = available_tags.get("GPSLatitude")?.as_f64()?;
    let lon = available_tags.get("GPSLongitude")?.as_f64()?;
    
    // ExifTool format: decimal degrees with 5 decimal places
    Some(format!("{:.5} {:.5}", lat, lon))
}

pub fn compute_gps_datetime(available_tags: &HashMap<String, TagValue>) -> Option<String> {
    let date_stamp = available_tags.get("GPSDateStamp")?.as_string()?;  // "2023:07:15"
    let time_stamp = available_tags.get("GPSTimeStamp")?.as_rational_array()?; // [14,1,30,1,25,1]
    
    // Convert time rationals to HH:MM:SS
    let hours = time_stamp[0] / time_stamp[1];
    let minutes = time_stamp[2] / time_stamp[3]; 
    let seconds = time_stamp[4] / time_stamp[5];
    
    // ExifTool format: "YYYY:MM:DD HH:MM:SSZ" (UTC)
    Some(format!("{}T{:02}:{:02}:{:02}Z", date_stamp.replace(':', "-"), hours, minutes, seconds))
}
```

### Phase 3: Advanced GPS Features

**GPS Processing Method**:
```rust
pub fn format_gps_processing_method(raw_data: &[u8]) -> String {
    // Check for encoding prefix (ASCII\0\0\0, Unicode\0\0, etc.)
    if raw_data.len() >= 8 {
        let encoding = &raw_data[0..8];
        match encoding {
            b"ASCII\0\0\0" => String::from_utf8_lossy(&raw_data[8..]).to_string(),
            b"Unicode\0\0" => decode_unicode(&raw_data[8..]),
            b"JIS\0\0\0\0\0" => decode_jis(&raw_data[8..]),
            _ => String::from_utf8_lossy(raw_data).to_string(),
        }
    } else {
        String::from_utf8_lossy(raw_data).to_string()
    }
}
```

## Prerequisites

- **P10a: EXIF Required Tags** - GPS IFD processing must be working
- **P12: Composite Required Tags** - Composite tag infrastructure needed
- GPS rational value conversion infrastructure (already exists per P10a)

## Testing Strategy

- **GPS Test Images**: Use images with destination coordinates (navigation apps, professional cameras)
- **Smartphone Images**: Test iPhone/Android GPS metadata  
- **Compare with ExifTool**: Verify decimal degree precision and formatting
- **Edge Cases**: Test images near poles, prime meridian, equator

## Success Criteria & Quality Gates

### You are NOT done until this is done:

1. **GPS Destination Tag Extraction**:
   - [ ] GPSDestLatitude and GPSDestLongitude extracting correctly
   - [ ] Destination coordinates converted to decimal degrees
   - [ ] GPSDestLatitudeRef/GPSDestLongitudeRef applied for sign

2. **GPS Composite Calculations**:
   - [ ] GPSPosition composite calculating correctly
   - [ ] GPSDateTime composite combining date and time stamps

3. **Specific Tag Validation** (must be added to `config/supported_tags.json` and pass `make compat-force`):
   ```json
   GPS destination tags:
   - "EXIF:GPSDestLatitude"    // Must show decimal degrees
   - "EXIF:GPSDestLongitude"   // Must show decimal degrees  
   - "EXIF:GPSProcessingMethod" // Must handle encoding prefixes
   
   GPS composite tags:
   - "Composite:GPSPosition"   // Must show "lat lon" format
   - "Composite:GPSDateTime"   // Must show UTC format
   ```

4. **Validation Commands**:
   ```bash
   # Test with GPS-enabled images:
   cargo run --bin compare-with-exiftool test-images/iphone/gps_photo.jpg GPS:
   cargo run --bin compare-with-exiftool test-images/garmin/waypoint.jpg GPS:
   
   # Verify composite calculations:
   make compat-force
   make compat-test | grep -E "(GPSPosition|GPSDateTime|GPSDestL)"
   ```

5. **Manual Validation**:
   - Compare GPS decimal degree precision with ExifTool (5 decimal places)
   - Verify destination coordinates process when present
   - Confirm GPSDateTime shows UTC timezone marker ("Z")

## Gotchas & Tribal Knowledge

### GPS Coordinate Precision
- **ExifTool Standard**: 5 decimal places for decimal degrees
- **Accuracy**: ~1 meter precision at equator
- **Sign Convention**: Negative for South latitude, West longitude

### GPS Destination vs Regular Coordinates
- **Regular GPS**: Where photo was taken (GPSLatitude/GPSLongitude)
- **Destination GPS**: Where photo subject is located (GPSDestLatitude/GPSDestLongitude)
- **Use Case**: Travel photos may show destination, not current location

### GPS Processing Method Encoding
- **Legacy Format**: May include 8-byte encoding prefix
- **Common Values**: "GPS", "NETWORK", "MANUAL", "CELLID", "WLAN"
- **Encoding Types**: ASCII, Unicode, JIS

### GPS DateTime Handling
- **GPS Time**: Always UTC, no timezone offset
- **Format**: Combines GPSDateStamp (YYYY:MM:DD) + GPSTimeStamp (rational array)
- **Precision**: Typically to nearest second

### Coordinate System Notes
- **Default Datum**: WGS-84 (GPS standard)
- **Other Datums**: May be specified in GPSMapDatum tag
- **Conversion**: Don't convert between datums - preserve as-is

### Common GPS Tag Locations
- **EXIF GPS IFD**: Standard location (tags 0x0001-0x001F)
- **XMP Location**: May duplicate or supplement GPS data
- **MakerNotes**: Some manufacturers store additional GPS data

## Dependencies

- GPS IFD processing infrastructure (from P10a)
- Rational value conversion (already implemented)
- Composite tag calculation framework (P12)
- String encoding handling for GPS text fields

---

**🗺️ Implementation Focus**: This TPP covers the GPS metadata extraction gaps identified in compatibility testing, particularly destination coordinates and composite GPS calculations that are critical for location-aware workflows.