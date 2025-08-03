
> 1. Scope Preference: Do you prefer a comprehensive solution that handles all binary data tables, or a targeted solution just for the immediate ShotInfo/Processing problem?

Comprehensive. ExifTool has a _lot_ of tags and tables. This puts a heavier burden on this design task, though!

> 2. ProcessBinaryData Pipeline: Should we activate the unused ProcessBinaryData pipeline, or is there a reason it was never set up?

OH DEAR I didn't know it wasn't set up!
 
> 3. Manual vs Generated Trade-off: The manual implementations in canon/binary_data.rs are working well. Should we preserve this pattern or move toward full code generation?

These manually-ported sections have caused a ton of bugs already -- it's very hard to validate and almost impossible to keep in sync with ExifTool monthly releases. We can keep the current manual stuff, and find low-hanging easy-to-codegen flavors, do those, remove those from the manual implementation, and iterate like that.

> 4. ColorData Integration: ColorData functions are already working. How should our solution interact with existing working binary processors?

I honestly didn't know we'd manually done that.

But like I said before, we can codegen the "low hanging fruit" types, and in runtime, try to use the codegen impl, but if codegen doesn't have an applicable impl, fallback to the manual impl
 
> 5. ExifTool Research Depth: How deep should I research ExifTool's ProcessBinaryData implementation? This could be a significant rabbit hole.

It is! We only _really_ need support for extracting full size and thumbnail embedded JPEG and TIFF images from JPEG and RAW image formats:

- PreviewImage
- PreviewTIFF
- JpgFromRaw
- JpgFromRaw2
- ThumbnailImage
- ThumbnailTIFF

See SUPPORTED-FORMATS.md for more details