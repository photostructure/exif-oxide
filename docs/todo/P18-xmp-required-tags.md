# Technical Project Plan: XMP Required Tags Implementation

## Project Overview

- **Goal**: Implement extraction of 63 XMP tags marked as required by PhotoStructure
- **Problem**: XMP parsing is not yet implemented, blocking access to critical workflow and rights metadata

## Background & Context

- XMP uses RDF/XML format embedded in images
- 50 required tags span multiple XMP namespaces (dc, xmp, xmpRights, etc.)
- Critical for rights management, people tagging, and processing history

## Technical Foundation

- **Key areas**:
  - Need XMP parser implementation
  - XML/RDF parsing
  - Namespace handling
  - XMP packet location in various file formats
- **Standards**: XMP Specification Part 1-3

## Required XMP Tags (63 total)

### High Frequency Core Tags (>50% frequency)
- **ImageHeight** (1.000) - Image dimensions
- **ImageWidth** (1.000) - Image dimensions
- **Make** (1.000) - Camera manufacturer
- **Model** (1.000) - Camera model
- **ExposureTime** (0.990) - Shutter speed
- **CreateDate** (0.970) - When created
- **DateTimeOriginal** (0.970) - Original capture time
- **FNumber** (0.970) - Aperture f-stop
- **FocalLength** (0.950) - Lens focal length
- **Orientation** (0.920) - Image rotation
- **ModifyDate** (0.890) - Last modified
- **ISO** (0.890) - ISO sensitivity
- **Software** (0.600) - Processing software

### Camera/Lens Information
- **LensID** (0.200) - Lens identification
- **LensModel** (0.100) - Lens model name
- **LensInfo** (0.086) - Lens specifications
- **LensMake** (0.022) - Lens manufacturer
- **ApertureValue** (0.390) - APEX aperture
- **ShutterSpeedValue** (0.380) - APEX shutter speed

### GPS Location Tags
- **GPSLatitude** (0.079) - Latitude coordinate
- **GPSLongitude** (0.079) - Longitude coordinate
- **GPSAltitude** (0.061) - Altitude
- **GPSProcessingMethod** (0.012) - GPS processing
- **GPSDateStamp** (0.027) - GPS date
- **GPSTimeStamp** (0.029) - GPS time

### Rights Management
- **Copyright** (0.200) - Copyright notice
- **License** (0.001) - Usage license
- **AttributionName** (0.001) - Credit name
- **AttributionURL** (0.001) - Credit URL
- **Permits** (0.001) - Permitted uses
- **Prohibits** (0.001) - Prohibited uses
- **Requires** (0.001) - Required actions
- **UseGuidelines** (0.001) - Usage guidelines
- **Jurisdiction** (0.001) - Legal jurisdiction

### People & Regions
- **PersonInImage** (0.000) - People in photo
- **PersonInImageName** (0.001) - Person names
- **PersonInImageWDetails** (0.001) - Person details
- **People** (0.001) - People list
- **RegionList** (0.005) - Face regions
- **HierarchicalKeywords** (0.005) - Keyword hierarchy

### Content & Workflow
- **Title** (0.021) - Image title
- **Description** (0.003) - Image description
- **Subject** (0.004) - Subject keywords
- **Keywords** (0.001) - Flat keywords
- **Rating** (0.140) - Star rating
- **CreatorTool** (0.032) - Creation software
- **MetadataDate** (0.020) - Metadata modified
- **LastKeywordXMP** (0.002) - Last keyword
- **Categories** (0.051) - Category tags
- **CatalogSets** (0.000) - Catalog membership
- **TagsList** (0.000) - All tags
- **Source** (0.000) - Image source
- **HierarchicalSubject** (0.001) - Subject hierarchy
- **KeywordInfo** (0.005) - Keyword metadata

### Time/Date Tags
- **DateTimeDigitized** (0.004) - Digitized time
- **SubSecTimeDigitized** (0.084) - Subsecond precision
- **CreationDate** (0.001) - Creation date
- **DigitalCreationDateTime** (0.001) - Digital creation
- **HistoryWhen** (0.001) - History timestamp
- **TrackCreateDate** (0.002) - Track creation
- **TrackModifyDate** (0.002) - Track modified

### Other Metadata
- **ColorSpace** (1.000) - Color space
- **MeteringMode** (1.000) - Metering mode
- **YCbCrPositioning** (1.000) - YCbCr positioning
- **FlashModel** (0.011) - Flash unit model
- **State** (0.010) - Location state
- **City** (0.010) - Location city
- **Country** (0.010) - Location country

## Work Completed

- ❌ No XMP parsing infrastructure yet
- ✅ Tag metadata identifies XMP namespace tags

## Remaining Tasks

### Critical - XMP Infrastructure

1. **XMP Packet Detection**
   - Find XMP packets in JPEG APP1 segments (after EXIF)
   - Extract from TIFF/DNG (typically in IFD0)
   - Handle sidecar .xmp files
   - Support extended XMP for large packets

2. **XML/RDF Parser**
   - Parse RDF/XML structure
   - Handle multiple namespaces (dc, xmp, xmpRights, exif, etc.)
   - Extract simple properties, arrays, and structures
   - Handle different RDF syntaxes (abbreviated, full)

3. **Namespace Registry**
   ```
   dc: http://purl.org/dc/elements/1.1/
   xmp: http://ns.adobe.com/xap/1.0/
   xmpRights: http://ns.adobe.com/xap/1.0/rights/
   exif: http://ns.adobe.com/exif/1.0/
   photoshop: http://ns.adobe.com/photoshop/1.0/
   xmpMM: http://ns.adobe.com/xap/1.0/mm/
   MWG-rs: http://www.metadataworkinggroup.com/schemas/regions/
   ```

### High Priority - Camera/EXIF Tags in XMP

Many standard EXIF tags are duplicated in XMP with high frequency:

1. **Core Camera Settings**
   - ExposureTime, FNumber, ISO (>90% frequency)
   - FocalLength, Make, Model
   - Map from exif: namespace

2. **Image Properties**
   - ImageWidth/Height, Orientation
   - ColorSpace, MeteringMode
   - Often in tiff: or exif: namespaces

3. **Timestamps**
   - CreateDate, ModifyDate, DateTimeOriginal
   - Handle timezone formatting differences

### Medium Priority - Content & Rights

1. **Dublin Core (dc:) Tags**
   - title, description, subject (arrays)
   - creator, rights, source
   - Handle language alternatives (xml:lang)

2. **Rights Management (xmpRights:)**
   - Marked, WebStatement, UsageTerms
   - Certificate, Owner
   - Complex structured properties

3. **People & Regions (MWG-rs:)**
   - RegionList with Areas and Names
   - Parse rectangle/circle regions
   - Handle rotation adjustments

### Low Priority - Workflow Tags

1. **Hierarchical Keywords (lr:)**
   - hierarchicalSubject structures
   - Maintain parent-child relationships

2. **History (xmpMM:)**
   - History array of actions
   - InstanceID, DocumentID tracking

3. **Photoshop (photoshop:)**
   - Category, SupplementalCategories
   - Instructions, Credit

## Prerequisites

- XML parsing library or implementation
- Understanding of RDF structure
- XMP specification compliance

## Testing Strategy

- Test with Adobe-created XMP
- Verify namespace handling
- Compare with ExifTool XMP extraction
- Test sidecar files

## Success Criteria

- Extract all 63 required XMP tags
- Handle multiple XMP namespaces correctly
- Parse structured properties (arrays, structs, alternatives)
- Match ExifTool output for XMP data
- Support both embedded and sidecar XMP

## Gotchas & Tribal Knowledge

### XMP Location Issues
- **JPEG**: XMP in APP1 segment after EXIF (starts with "http://ns.adobe.com/xap/1.0/\0")
- **TIFF/DNG**: Usually in IFD0 tag 0x02BC (XMP)
- **Multiple Packets**: Some files have XMP in multiple locations
- **Extended XMP**: Large packets split across multiple APP1 segments

### Parsing Complexities
- **Namespace Prefixes**: Arbitrary (dc:title vs dublin:title)
- **RDF Syntax**: Same data can be expressed multiple ways
- **Language Alternatives**: dc:title can have multiple languages
- **Empty vs Missing**: Distinguish between empty string and no value

### Data Duplication
- **EXIF/XMP Overlap**: Many tags exist in both (prefer EXIF if conflicts)
- **Precedence**: EXIF > XMP > IPTC for standard tags
- **Format Differences**: Dates, GPS coords formatted differently

### Structure Types
- **Simple Properties**: `<dc:format>image/jpeg</dc:format>`
- **Arrays**: `<dc:subject><rdf:Bag><rdf:li>keyword</rdf:li></rdf:Bag></dc:subject>`
- **Structures**: `<xmpRights:UsageTerms><rdf:Alt><rdf:li xml:lang="x-default">terms</rdf:li></rdf:Alt></xmpRights:UsageTerms>`

### Common Mistakes
- **UTF-8 BOM**: XMP should not have BOM but some tools add it
- **Whitespace**: Significant in some contexts, not in others
- **CDATA**: May contain embedded XML that needs escaping
- **Packet Wrapper**: <?xpacket> wrapper is optional

### Special Cases
- **Lightroom**: Uses lr: namespace for hierarchical keywords
- **Photoshop**: Stores legacy IPTC data in XMP
- **Face Regions**: MWG standard vs proprietary formats
- **GPS**: Different coordinate format than EXIF (decimal vs DMS)