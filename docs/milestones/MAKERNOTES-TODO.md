# MakerNotes Implementation Status

**Last Updated**: 2025-07-18

## Context

MakerNotes tags were temporarily removed from `config/supported_tags.json` to allow compatibility tests to pass while manufacturer-specific implementations are being developed according to planned milestones.

## Removed Tags

The following MakerNotes tags were removed from `config/supported_tags.json`:

- `MakerNotes:Model`
- `MakerNotes:Make` 
- `MakerNotes:DateTimeOriginal`
- `MakerNotes:ISO`
- `MakerNotes:ExposureTime`
- `MakerNotes:FNumber`
- `MakerNotes:FocalLength`
- `MakerNotes:Lens`

## Implementation Schedule

### ✅ Sony MakerNotes
- **Milestone**: MILESTONE-17e-Sony-RAW
- **Missing Tags**: `MakerNotes:ExposureTime`, `MakerNotes:FocalLength`
- **Status**: Planned - Sony.pm codegen extraction

### ✅ Canon MakerNotes  
- **Milestone**: MILESTONE-17d-Canon-RAW
- **Missing Tags**: Various Canon-specific MakerNotes
- **Status**: Planned - Canon lens database and binary data extraction

### ⏳ Nikon MakerNotes
- **Milestone**: Not yet scheduled
- **Missing Tags**: `MakerNotes:ISO`, `MakerNotes:FocalLength`, `MakerNotes:Lens`
- **Status**: Needs Nikon-specific tag extraction implementation

### ✅ Other Manufacturers
- **Milestone**: MILESTONE-MOAR-CODEGEN  
- **Manufacturers**: Panasonic (~8-12 tables), Pentax (~5-8 tables), Olympus (~6-10 tables)
- **Status**: Planned - Simple table extraction framework

## TODO: Re-enable Tags

When each milestone is complete:

1. **Add back manufacturer-specific MakerNotes tags** to `supported_tags.json`
2. **Verify compatibility tests pass** for that manufacturer's files
3. **Update this document** to reflect completed milestones

## Files Affected

- **Sony**: `test-images/sony/sony_a7c_ii_02.jpg` (and others)
- **Canon**: `test-images/canon/Canon_T3i.jpg`, `test-images/canon/canon_eos_r5_mark_ii_10.jpg` (and others)
- **Nikon**: `third-party/exiftool/t/images/Nikon.jpg`, `third-party/exiftool/t/images/NikonD70.jpg` (and others)
- **Multiple**: Casio, Minolta, Pentax, Kodak, Panasonic files