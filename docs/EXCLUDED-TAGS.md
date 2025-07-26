# Excluded Tags

**🚨 CRITICAL: These exclusions are exceptions to [TRUST-EXIFTOOL.md](TRUST-EXIFTOOL.md) for scope management.**

The following tags have limited to no use, and effort should not be expended on
extraction, ValueConv, or PrintConv (regardless of TagMetadata)

NOTE! Even though `ExifImageWidth` and `ExifImageHeight` are untrustworthy for
most image types due to the fact that almost no image application updates that
metadata correctly, we MUST parse it correctly to support several types of RAW
image formats (like Canon).

```
EXIF:CFAPattern
Composite:CFAPattern
EXIF:RedBlueBalance
EXIF:CalculateLV
EXIF:CalcScaleFactor35efl
EXIF:Software
EXIF:ExifVersion
```
