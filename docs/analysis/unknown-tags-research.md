# Unknown Tags Research

Research document identifying ExifTool sources for required tags that were not initially found in tag-metadata.json.

**Generated**: 2025-12-08
**Total Tags Researched**: 16 + 1 deferred

---

## Creative Commons Tags (XMP-cc)

All Creative Commons tags are defined in the `cc` namespace in XMP2.pl.

### License

- **Source**: lib/Image/ExifTool/XMP2.pl:1431
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: string (Resource URI)
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: Work property. `Resource => 1` indicates URI reference.

### AttributionName

- **Source**: lib/Image/ExifTool/XMP2.pl:1432
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: string
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: Work property for creator/artist name attribution.

### AttributionURL

- **Source**: lib/Image/ExifTool/XMP2.pl:1433
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: string (Resource URI)
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: Work property. `Resource => 1` indicates URI reference.

### UseGuidelines

- **Source**: lib/Image/ExifTool/XMP2.pl:1435
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: string (Resource URI)
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: Work property pointing to usage guidelines. `Resource => 1` indicates URI reference.

### Permits

- **Source**: lib/Image/ExifTool/XMP2.pl:1437-1446
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: Bag of Resource URIs
- **ValueConv**: No
- **PrintConv**: Yes - Hash map converting cc: URIs to human-readable text:
  - `cc:Sharing` → 'Sharing'
  - `cc:DerivativeWorks` → 'Derivative Works'
  - `cc:Reproduction` → 'Reproduction'
  - `cc:Distribution` → 'Distribution'
- **Implementation Complexity**: Medium
- **Notes**: License property listing permitted uses. `List => 'Bag'`, `Resource => 1`.

### Requires

- **Source**: lib/Image/ExifTool/XMP2.pl:1447-1458
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: Bag of Resource URIs
- **ValueConv**: No
- **PrintConv**: Yes - Hash map converting cc: URIs to human-readable text:
  - `cc:Copyleft` → 'Copyleft'
  - `cc:LesserCopyleft` → 'Lesser Copyleft'
  - `cc:SourceCode` → 'Source Code'
  - `cc:ShareAlike` → 'Share Alike'
  - `cc:Notice` → 'Notice'
  - `cc:Attribution` → 'Attribution'
- **Implementation Complexity**: Medium
- **Notes**: License property listing required conditions. `List => 'Bag'`, `Resource => 1`.

### Prohibits

- **Source**: lib/Image/ExifTool/XMP2.pl:1459-1466
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: Bag of Resource URIs
- **ValueConv**: No
- **PrintConv**: Yes - Hash map converting cc: URIs to human-readable text:
  - `cc:HighIncomeNationUse` → 'High Income Nation Use'
  - `cc:CommercialUse` → 'Commercial Use'
- **Implementation Complexity**: Medium
- **Notes**: License property listing prohibited uses. `List => 'Bag'`, `Resource => 1`.

### Jurisdiction

- **Source**: lib/Image/ExifTool/XMP2.pl:1467
- **Namespace**: XMP-cc (Creative Commons)
- **Data Type**: string (Resource URI)
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: License property specifying applicable jurisdiction. `Resource => 1` indicates URI reference.

---

## MWG Region Tags (XMP-mwg-rs)

Metadata Working Group region tags for face detection and image areas.

### RegionList

- **Source**: lib/Image/ExifTool/MWG.pm:473 (struct), 481 (flattened export)
- **Namespace**: XMP-mwg-rs (MWG Regions)
- **Data Type**: Bag of MWG RegionStruct
- **ValueConv**: No
- **PrintConv**: Yes (for Type field only)
- **Implementation Complexity**: Very High
- **Notes**: Deeply nested structure containing:
  - **Area**: Complex struct with x, y, w, h, unit fields
  - **Type**: PrintConv for 'Face', 'Pet', 'Focus', 'BarCode'
  - **Name**, **Description**: Simple strings
  - **FocusUsage**: PrintConv for evaluation status
  - **BarCodeValue**: String
  - **Extensions**: Variable namespace
  - **Rotation**: Real number (Lightroom extension)

  Requires full XMP struct parsing infrastructure.

---

## MWG Keyword Tags (XMP-mwg-kw)

Metadata Working Group hierarchical keyword tags.

### KeywordInfo

- **Source**: lib/Image/ExifTool/MWG.pm:499
- **Namespace**: XMP-mwg-kw (MWG Keywords)
- **Data Type**: Struct containing Hierarchy field
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: High
- **Notes**: Root container for hierarchical keyword structure. Contains:
  - **Hierarchy**: Bag of MWG KeywordStruct items (recursive)

### HierarchicalKeywords

- **Source**: lib/Image/ExifTool/MWG.pm:506-523
- **Namespace**: XMP-mwg-kw (MWG Keywords)
- **Data Type**: Recursive struct (unrolled to 6 levels)
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Very High
- **Notes**: Mapped from KeywordsHierarchy. Each KeywordStruct contains:
  - **Keyword**: String
  - **Applied**: Boolean
  - **Children**: Bag of KeywordStruct (recursive)

  ExifTool flattens to 6 levels: HierarchicalKeywords1 through HierarchicalKeywords6, each with Applied and Children variants.

---

## IPTC Extension Tags (XMP-Iptc4xmpExt)

IPTC Extensions version 1.3+ tags for person identification.

### PersonInImageWDetails

- **Source**: lib/Image/ExifTool/XMP2.pl:619
- **Namespace**: XMP-Iptc4xmpExt (IPTC Extensions)
- **Data Type**: Bag of PersonDetails structs
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: High
- **Notes**: Complex structure containing:
  - **PersonId**: List/Bag
  - **PersonName**: lang-alt (language-alternate)
  - **PersonCharacteristic**: Bag of CVTermDetails structures with CvTermId, CvTermName, CvId, CvTermRefinedAbout
  - **PersonDescription**: lang-alt

  Exports flattened tags including PersonInImageName.

### PersonInImageName

- **Source**: lib/Image/ExifTool/XMP2.pl:634
- **Namespace**: XMP-Iptc4xmpExt (IPTC Extensions)
- **Data Type**: lang-alt string (language-alternate)
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: Flattened/aliased tag (`Flat => 1`) extracting PersonName from PersonInImageWDetails structure. Supports multiple languages.

---

## XMP Media Management Tags (XMP-xmpMM)

XMP Media Management tags for edit history tracking.

### HistoryWhen

- **Source**: lib/Image/ExifTool/XMP.pm:341
- **Namespace**: XMP-stEvt (within xmpMM namespace)
- **Data Type**: DateTime
- **ValueConv**: Yes - `%dateTimeInfo` provides datetime conversion
- **PrintConv**: Yes - `ConvertDateTime` via dateTimeInfo
- **Implementation Complexity**: Medium
- **Notes**: Field within stEvt (ResourceEvent) structure, used in History list under xmpMM. Part of the resource event struct:
  - action, instanceID, parameters, softwareAgent, **when**, changed

  Groups => { 2 => 'Time' }

---

## Media Management Software Tags

Tags from media management applications.

### People

- **Source**: lib/Image/ExifTool/XMP2.pl:1523 (mediapro), 1541 (expressionmedia)
- **Namespace**: XMP-mediapro (iView MediaPro) or XMP-expressionmedia (Microsoft Expression Media)
- **Data Type**: Bag of strings
- **ValueConv**: No
- **PrintConv**: No
- **Implementation Complexity**: Low
- **Notes**: Also defined in JPEG.pm:580 for JPEG-specific context. Simple `List => 'Bag'` type containing people names.

---

## EXIF/DNG Tags

DNG-specific EXIF tags.

### DNGLensInfo

- **Source**: lib/Image/ExifTool/Exif.pm:3475-3483
- **Namespace**: EXIF (IFD0)
- **Tag ID**: 0xc630
- **Data Type**: rational64u[4] (4-element rational array)
- **ValueConv**: No
- **PrintConv**: Yes - `PrintLensInfo` function
- **PrintConvInv**: Yes - `ConvertLensInfo` function
- **Implementation Complexity**: Medium
- **Notes**: DNG lens information as 4 rational values:
  - Minimum focal length
  - Maximum focal length
  - Minimum f-number (at min focal length)
  - Maximum f-number (at max focal length)

  Groups => { 2 => 'Camera' }, WriteGroup => 'IFD0'

---

## Deferred Tags

Tags that require separate implementation due to complexity or non-standard nature.

### ImageDataHash

- **Source**: lib/Image/ExifTool/WriteExif.pl:423-462
- **Function**: `AddImageDataHash($$$)`
- **Data Type**: Computed SHA-256 digest
- **Implementation Complexity**: High
- **Notes**: **NOT a metadata tag** - runtime-computed hash of image data.

  **Used by**:
  - FujiFilm.pm:1979 (raw processing)
  - SigmaRaw.pm:554 (Sigma raw processing)
  - RIFF.pm:2187 (RIFF/AVI/WAV processing)
  - QuickTime.pm (QuickTime processing)
  - PhaseOne.pm:589 (PhaseOne raw processing)

  **Implementation requires**:
  - Identifying image data sections via `IsImageData` tag flag
  - Accumulating SHA-256 hash across multiple data blocks
  - Handling offset/size pairs for non-contiguous image data

  **Status**: Deferred to separate implementation TPP

---

## Summary by Implementation Complexity

### Low Effort (9 tags)
- License, AttributionName, AttributionURL, UseGuidelines, Jurisdiction
- PersonInImageName, HistoryWhen, People, DNGLensInfo

### Medium Effort (3 tags)
- Permits, Requires, Prohibits (PrintConv hash maps for cc: URIs)

### High Effort (2 tags)
- PersonInImageWDetails (nested struct with lang-alt)
- KeywordInfo (container for hierarchical keywords)

### Very High Effort (2 tags)
- RegionList (deeply nested struct with Area, Type, FocusUsage)
- HierarchicalKeywords (recursive struct unrolled to 6 levels)

### Deferred (1 tag)
- ImageDataHash (computed value requiring image data parsing)

---

## Removed from Required Tags

### CameraModelName
**Reason**: Not a distinct tag. This is the `Model` tag (EXIF 0x110, Exif.pm:567-576) with `Description => 'Camera Model Name'`. Already implemented via standard Model tag.

### FileVersion
**Reason**: Not required by PhotoStructure. Also format-dependent with different implementations per file format (SigmaRaw, IPTC, PSP, ZIP) - not a unified tag.
