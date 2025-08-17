## Expression improvements

- finish and validate P17-string-formatting-expression-compiler.md
- add, review, and start function calling and string concatenation to the expression parser

## Continue work on docs/todo/P13-olympus-required-tags.md
- finish and validate docs/todo/P15-ifd-parsing-completeness-and-context-bugs.md
- continue work on docs/todo/P13-olympus-required-tags.md:
  
● Update Todos

  ⎿  ☒ Research complete: Missing runtime format detection for Equipment conditional dispatch              
     ☒ Implement runtime format detection for Equipment subdirectory processing
     ☒ Investigation complete: MakerNotes signature detection not triggering Olympus-specific processing
     ☒ Root cause identified: MakerNotes processed as direct IFD, bypassing signature detection entirely
     ☒ Root cause confirmed: Generated process_tag_0x927c_subdirectory is stub missing conditional dispatch
     ☒ MakerNotes conditional dispatch system implemented successfully
     ☒ Test MakerNotes conditional dispatch with Olympus sample image
     ☒ Fix Equipment tag name resolution - tags show as Tag_F100 instead of CameraType2
     ☒ Implement LensID composite calculation using Equipment LensType and olympusLensTypes lookup
     ☐ Add ExpressionEvaluator integration to subdirectory dispatch system
     ☐ BLOCKED: Build error in src/formats/mod.rs - is_gps_coordinate_tag function missing
     ☐ INVESTIGATE: Debug MakerNotes tag location - verify why 0x927c not found in test image
     ☐ Replace placeholder test data with actual Equipment IFD parsing
     ☐ Implement proper binary Equipment processing for old cameras
     ☐ Test Equipment processing with multiple Olympus camera models

## Continue other required tags jobs

- canon
- nikon
- sony
- fuji


- spin up an analysis job to see how we're going to get the last 108 tags fixed

- upgrade BinaryDataTag::from_legacy consumers?