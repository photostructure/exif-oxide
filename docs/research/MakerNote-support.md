# MakerNote.pm support research

## What do `Condition`s look like in third-party/exiftool/lib/Image/ExifTool/MakerNotes.pm ?

mrm@speedy:~/src/exif-oxide$ perl ./codegen/scripts/field_extractor.pl third-party/exiftool/lib/Image/ExifTool/MakerNotes.pm Main | jq '[.data[] | .Condition | select(. != null) | gsub("^\\s+|\\s+$"; "")] | unique' 

### 1. Simple regex matching with =~ and !~

    "$$self{Make} =~ /^Canon/",

### 2. `and` `or` `eq` `ne`

    "$$self{Make} =~ /^Leica Camera AG/ and $$valPt =~ /^LEICA0/",

                Condition => q{
                $$self{Model} !~ /EOS/ or
                $$self{Model} =~ /\b(1DS?|5D|D30|D60|10D|20D|30D|K236)$/ or
                $$self{Model} =~ /\b((300D|350D|400D) DIGITAL|REBEL( XTi?)?|Kiss Digital( [NX])?)$/
            },


### 3. Parameters

- `$$self{Make}`
- `$$self{Model}`
- `$$valPt`
- `$$self{TIFF_TYPE}` (but maybe skip this for now and add it later)

### 4. COMMENTS!

Watch out for whitespace normalization, because HE ADDED COMMENTS IN THE STRING

(can we use perltidy to delete comments? or just `/#.+\n//g` and hope for the best?)

  "$$self{Make}=~/^EASTMAN KODAK/ and\n            ($$self{Model}=~/CX(4200|4230|4300|4310|6200|6230)/ or\n            # try to pick up similar models we haven't tested yet\n            $$valPt=~/^\\0(\\x1a\\x18|\\x3a\\x08|\\x59\\xf8|\\x14\\x80)\\0/)",
  "return undef unless $$self{Make}=~/^(SIGMA|FOVEON)/i;\n            # save version number in \"MakerNoteSigmaVer\" member variable\n            $$self{MakerNoteSigmaVer} = $$valPt=~/^SIGMA\\0\\0\\0.(.)/s ? ord($1) : -1;\n            return 1;",

### OMG MORE NON-UTF8 REGEX

  "$$self{Model}=~/(Kodak|PixPro)/i and $$valPt =~ /^II\\x2a\\0\\x08\\0\\0\\0.\\0\\0\\0/s",
  "$$self{Model}=~/(Kodak|PixPro)/i and $$valPt =~ /^MM\\0\\x2a\\0\\0\\0\\x08\\0\\0\\0./s",
  "$$valPt =~ /^ISLMAKERNOTE000\\0/",
  "$$valPt =~ /^LEICA CAMERA AG\\0/",
  "$$valPt =~ /^LEICA\\0[\\x01\\x04\\x05\\x06\\x07\\x10\\x1a]\\0/",
  "$$valPt =~ /^LEICA\\0[\\x08\\x09\\x0a]\\0/",
  "$$valPt =~ /^LEICA\\0\\x02\\xff/",
  "$$valPt =~ /^LSI1\\0/",
  "$$valPt =~ /^OLYMPUS\\0/",
  "$$valPt =~ /^OM SYSTEM\\0/",
  "$$valPt =~ /^RECONYXH2\\0/",
  "$$valPt =~ /^RECONYXUF\\0/",

So we need to do escaping shenanigans just like we did in codegen/src/strategies/magic_numbers.rs -- this functionality should be extracted so that both magic_numbers and we use the same shared utility functions.

### Scary outliers

Maybe let's skip these for now?

  "($$self{Make} eq 'Leica Camera AG' and ($$self{Model} eq 'S2' or\n            $$self{Model} eq 'LEICA M (Typ 240)' or $$self{Model} eq 'LEICA S (Typ 006)'))",
  "($$self{Make}=~/^SONY/ or ($$self{Make}=~/^HASSELBLAD/ and\n            $$self{Model}=~/^(HV|Stellar|Lusso|Lunar)/)) and $$valPt!~/^\\x01\\x00/",
  "return undef unless $$self{Make}=~/^(SIGMA|FOVEON)/i;\n            # save version number in \"MakerNoteSigmaVer\" member variable\n            $$self{MakerNoteSigmaVer} = $$valPt=~/^SIGMA\\0\\0\\0.(.)/s ? ord($1) : -1;\n            return 1;",
  "return undef unless $$valPt =~ /^(IIII.waR|MMMMRaw.)/s;\n            $self->OverrideFileType($$self{TIFF_TYPE} = 'IIQ') if $count > 1000000;\n            return 1;",
  "uc $$self{Make} eq 'SAMSUNG' and ($$self{TIFF_TYPE} eq 'SRW' or\n            $$valPt=~/^(\\0.\\0\\x01\\0\\x07\\0{3}\\x04|.\\0\\x01\\0\\x07\\0\\x04\\0{3})0100/s)"


(omg, `$self->OverrideFileType`? and $count > 1000000? what even is this? yikes)

## NOTE: Condition is in some "tag_kit"/ ExifTool Tag tables too!

perl ./codegen/scripts/field_extractor.pl third-party/exiftool/lib/Image/ExifTool/Canon.pm FocalLength

```json
{"data":{"0":{"Name":"FocalType","PrintConv":{"1":"Fixed","2":"Zoom"},"RawConv":"$val ? $val : undef"},"1":{"Name":"FocalLength","PrintConv":"\"$val mm\"","PrintConvInv":"$val=~s/\\s*mm//;$val","Priority":0,"RawConv":"$val ? $val : undef","RawConvInv":"\n            my $focalUnits = $$self{FocalUnits};\n            unless ($focalUnits) {\n                $focalUnits = 1;\n                # (this happens when writing FocalLength to CRW images)\n                $self->Warn(\"FocalUnits not available for FocalLength conversion (1 assumed)\");\n            }\n            return $val * $focalUnits;\n        ","ValueConv":"$val / ($$self{FocalUnits} || 1)","ValueConvInv":"$val"},"2":[{"Condition":"\n                $$self{Model} !~ /EOS/ or\n                $$self{Model} =~ /\\b(1DS?|5D|D30|D60|10D|20D|30D|K236)$/ or\n                $$self{Model} =~ /\\b((300D|350D|400D) DIGITAL|REBEL( XTi?)?|Kiss Digital( [NX])?)$/\n            ","Name":"FocalPlaneXSize","Notes":"\n                these focal plane sizes are only valid for some models, and are affected by\n                digital zoom if applied\n            ","PrintConv":"sprintf(\"%.2f mm\",$val)","PrintConvInv":"$val=~s/\\s*mm$//;$val","RawConv":"$val < 40 ? undef : $val","ValueConv":"$val * 25.4 / 1000","ValueConvInv":"int($val * 1000 / 25.4 + 0.5)"},{"Name":"FocalPlaneXUnknown","Unknown":1}],"3":[{"Condition":"\n                $$self{Model} !~ /EOS/ or\n                $$self{Model} =~ /\\b(1DS?|5D|D30|D60|10D|20D|30D|K236)$/ or\n                $$self{Model} =~ /\\b((300D|350D|400D) DIGITAL|REBEL( XTi?)?|Kiss Digital( [NX])?)$/\n            ","Name":"FocalPlaneYSize","PrintConv":"sprintf(\"%.2f mm\",$val)","PrintConvInv":"$val=~s/\\s*mm$//;$val","RawConv":"$val < 40 ? undef : $val","ValueConv":"$val * 25.4 / 1000","ValueConvInv":"int($val * 1000 / 25.4 + 0.5)"},{"Name":"FocalPlaneYUnknown","Unknown":1}],"CHECK_PROC":"[Function: Image::ExifTool::CheckBinaryData]","FIRST_ENTRY":0,"FORMAT":"int16u","GROUPS":{"0":"MakerNotes","2":"Image"},"PROCESS_PROC":"[Function: Image::ExifTool::ProcessBinaryData]","WRITABLE":1,"WRITE_PROC":"[Function: Image::ExifTool::WriteBinaryData]"},"metadata":{"is_composite_table":0,"size":11},"module":"Canon","name":"FocalLength","type":"hash"}
```

mrm@speedy:~/src/exif-oxide$ perl ./codegen/scripts/field_extractor.pl third-party/exiftool/lib/Image/ExifTool/Canon.pm FocalLength | jq '[.. | .Condition? | select(. != null) | gsub("^\\s+|\\s+$"; "")] | unique' 

```json
[
  "$$self{Model} !~ /EOS/ or\n                $$self{Model} =~ /\\b(1DS?|5D|D30|D60|10D|20D|30D|K236)$/ or\n                $$self{Model} =~ /\\b((300D|350D|400D) DIGITAL|REBEL( XTi?)?|Kiss Digital( [NX])?)$/"
]
```