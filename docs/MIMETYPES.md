# TODO: Supported File Formats for Digital Asset Management

This document lists all file formats that should be supported by exif-oxide for use in digital asset management systems. The list is based on ExifTool's comprehensive format support and focuses on formats relevant to professional photography and videography.

## Image Formats

### Common Formats

| Format              | Extensions         | MIME Type                                    | Description                                    |
| ------------------- | ------------------ | -------------------------------------------- | ---------------------------------------------- |
| JPEG                | .jpg, .jpeg, .jpe  | `image/jpeg`                                 | Joint Photographic Experts Group               |
| PNG                 | .png               | `image/png`                                  | Portable Network Graphics                      |
| TIFF                | .tiff, .tif        | `image/tiff`                                 | Tagged Image File Format                       |
| WebP                | .webp              | `image/webp`                                 | Google Web Picture format                      |
| HEIC/HEIF           | .heic, .heif, .hif | `image/heic`, `image/heif`                   | High Efficiency Image Format (images)          |
| HEIC/HEIF Sequences | .heics, .heifs     | `image/heic-sequence`, `image/heif-sequence` | High Efficiency Image Format (image sequences) |
| AVIF                | .avif              | `image/avif`                                 | AV1 Image File Format                          |
| BMP                 | .bmp               | `image/bmp`                                  | Bitmap Image File                              |
| GIF                 | .gif               | `image/gif`                                  | Graphics Interchange Format                    |

### RAW Image Formats

#### Canon

| Format | Extensions | MIME Type           |
| ------ | ---------- | ------------------- |
| CR2    | .cr2       | `image/x-canon-cr2` |
| CR3    | .cr3       | `image/x-canon-cr3` |
| CRW    | .crw       | `image/x-canon-crw` |

#### Nikon

| Format | Extensions | MIME Type           |
| ------ | ---------- | ------------------- |
| NEF    | .nef       | `image/x-nikon-nef` |
| NRW    | .nrw       | `image/x-nikon-nrw` |

#### Sony

| Format | Extensions | MIME Type          |
| ------ | ---------- | ------------------ |
| ARW    | .arw       | `image/x-sony-arw` |
| ARQ    | .arq       | `image/x-sony-arq` |
| SR2    | .sr2       | `image/x-sony-sr2` |
| SRF    | .srf       | `image/x-sony-srf` |

#### Fujifilm

| Format | Extensions | MIME Type              |
| ------ | ---------- | ---------------------- |
| RAF    | .raf       | `image/x-fujifilm-raf` |

#### Olympus

| Format | Extensions | MIME Type             |
| ------ | ---------- | --------------------- |
| ORF    | .orf       | `image/x-olympus-orf` |

#### Panasonic

| Format | Extensions | MIME Type               |
| ------ | ---------- | ----------------------- |
| RAW    | .raw       | `image/x-panasonic-raw` |
| RW2    | .rw2       | `image/x-panasonic-rw2` |

#### Other Manufacturers

| Manufacturer | Format | Extensions | MIME Type                |
| ------------ | ------ | ---------- | ------------------------ |
| Adobe        | DNG    | .dng       | `image/x-adobe-dng`      |
| Epson        | ERF    | .erf       | `image/x-epson-erf`      |
| GoPro        | GPR    | .gpr       | `image/x-gopro-gpr`      |
| Hasselblad   | 3FR    | .3fr       | `image/x-hasselblad-3fr` |
| Hasselblad   | FFF    | .fff       | `image/x-hasselblad-fff` |
| Kodak        | DCR    | .dcr       | `image/x-kodak-dcr`      |
| Kodak        | K25    | .k25       | `image/x-kodak-k25`      |
| Kodak        | KDC    | .kdc       | `image/x-kodak-kdc`      |
| Leica        | RWL    | .rwl       | `image/x-leica-rwl`      |
| Mamiya       | MEF    | .mef       | `image/x-mamiya-mef`     |
| Minolta      | MRW    | .mrw       | `image/x-minolta-mrw`    |
| Pentax       | PEF    | .pef, .dng | `image/x-pentax-pef`     |
| Phase One    | IIQ    | .iiq       | `image/x-phaseone-iiq`   |
| Samsung      | SRW    | .srw       | `image/x-samsung-srw`    |
| Sigma        | X3F    | .x3f       | `image/x-sigma-x3f`      |

## Video Formats

| Format     | Extensions        | MIME Type           | Description                                    |
| ---------- | ----------------- | ------------------- | ---------------------------------------------- |
| 3GPP       | .3gp, .3gpp       | `video/3gpp`        | 3GPP Multimedia                                |
| 3GPP2      | .3g2              | `video/3gpp2`       | 3GPP2 Multimedia                               |
| ASF        | .asf              | `video/x-ms-asf`    | Advanced Systems Format                        |
| AVI        | .avi              | `video/x-msvideo`   | Audio Video Interleave                         |
| Canon CRM  | .crm              | `video/x-canon-crm` | Canon RAW Movie                                |
| HEIF Video | .heic, .heif      | `video/heif`        | High Efficiency Image Format (video sequences) |
| M4V        | .m4v              | `video/x-m4v`       | iTunes Video File                              |
| Matroska   | .mkv              | `video/x-matroska`  | Matroska Video                                 |
| MNG        | .mng              | `video/mng`         | Multiple-image Network Graphics                |
| MP4        | .mp4, .insv       | `video/mp4`         | MPEG-4 Part 14 (insv = Insta360)               |
| MPEG       | .m2v, .mpeg, .mpg | `video/mpeg`        | MPEG Video                                     |
| MPEG-TS    | .mts, .m2ts, .ts  | `video/m2ts`        | MPEG Transport Stream                          |
| QuickTime  | .mov, .qt         | `video/quicktime`   | Apple QuickTime Movie                          |
| WebM       | .webm             | `video/webm`        | WebM Video                                     |
| WMV        | .wmv              | `video/x-ms-wmv`    | Windows Media Video                            |

## Metadata/Sidecar Formats

| Format      | Extensions | MIME Type                    | Description                            |
| ----------- | ---------- | ---------------------------- | -------------------------------------- |
| XMP         | .xmp       | `application/rdf+xml`        | Extensible Metadata Platform           |
| ICC Profile | .icc, .icm | `application/vnd.iccprofile` | International Color Consortium Profile |

## Professional/Adobe Formats

| Format    | Extensions | MIME Type                   | Description              |
| --------- | ---------- | --------------------------- | ------------------------ |
| Photoshop | .psd, .psb | `image/vnd.adobe.photoshop` | Adobe Photoshop Document |
| DCP       | .dcp       | `image/x-adobe-dcp`         | Adobe DNG Camera Profile |

## Implementation Priority

### Phase 1 - Core Formats (Current)

- [ ] JPEG (.jpg, .jpeg)
- [ ] Basic EXIF extraction
- [ ] XMP sidecar support
- [ ] Thumbnail/preview extraction

### Phase 2 - Essential DAM Formats

- [ ] HEIC/HEIF support
- [ ] PNG with embedded metadata
- [ ] TIFF (foundation for many RAW formats)
- [ ] MP4/MOV video metadata

### Phase 3 - Camera RAW Formats

- [ ] Canon CR2/CR3
- [ ] Nikon NEF
- [ ] Sony ARW
- [ ] Sony ARQ
- [ ] Fujifilm RAF
- [ ] Olympus ORW
- [ ] Panasonic RW2
- [ ] DNG (Adobe Digital Negative)

### Phase 4 - Extended Support

- [ ] Additional RAW formats
- [ ] WebP/AVIF modern formats
- [ ] Extended video format support
- [ ] Professional formats (PSD, etc.)

## Notes

1. **MIME Types**: The MIME types listed are commonly used conventions, but some formats (especially RAW) don't have officially registered MIME types. We should match whatever ExifTool uses.

2. **Priority**: Focus on formats used by major camera manufacturers and common interchange formats first.

3. **Metadata Standards**: All formats should support extraction of:

   - EXIF data
   - XMP metadata
   - IPTC data (where applicable)
   - Maker notes
   - Embedded thumbnails/previews

4. **Performance Target**: Maintain 10-20x performance improvement over ExifTool for all supported formats.

5. **HEIF/HEIC Dual Nature**: HEIF containers can contain both single images and video sequences. The same file extensions (.heic, .heif) are used for both, requiring content inspection to determine the actual media type. Both should be supported for comprehensive DAM functionality.

6. **Additional RAW Formats**: Some less common RAW formats may need investigation:
   - GPR (GoPro RAW) - action camera format
   - MEF (Mamiya) - medium format camera
   - CRM (Canon) - newer Canon format variant
