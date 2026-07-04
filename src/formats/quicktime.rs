//! QuickTime / MP4 (ISO Base Media File Format) streaming atom walker
//!
//! Reads the metadata slice of a QuickTime container without loading the whole
//! file: it walks the atom tree with `Seek`, decoding the small header boxes and
//! seeking past media data (`mdat`, which is gigabytes in real videos — we never
//! `read_to_end`). This is the hand-written "Option A" walker from the TPP
//! (`_todo/20260703-P1-quicktime-video-read.md`): a procedural container walk
//! like `jpeg.rs`/`tiff.rs`, dispatched from `formats/mod.rs`.
//!
//! Trust ExifTool (docs/TRUST-EXIFTOOL.md): the atom routing mirrors
//! QuickTime.pm's SubDirectory chain and the leaf decoders mirror the
//! ProcessBinaryData tables. Conversions live in
//! [`crate::implementations::quicktime`]; citations are inline.
//!
//! ExifTool reference: QuickTime.pm ProcessMOV:9932, atom header:9974/10036-10090.
//!
//! Scope (TPP Task 2): mvhd (MovieHeader:1343), tkhd (TrackHeader:1493),
//! mdhd (MediaHeader:7239) — dates, durations, dimensions. Task 3's
//! meta/keys/ilst and udta, and Task 4's Rotation from hdlr HandlerType +
//! tkhd MatrixStructure, are left as clearly-marked TODO arms so the container
//! structure is already in place.

use std::io::{Read, Seek, SeekFrom};

use indexmap::IndexMap;
use tracing::trace;

use crate::implementations::quicktime as qt;
use crate::types::{Result, TagEntry, TagValue};

/// Guard against pathologically deep / cyclic atom nesting (fuzz target, Task 5).
const MAX_DEPTH: u32 = 16;

/// The ExifTool group (G0/G1) for QuickTime container tags. ExifTool `-G` emits
/// `QuickTime:CreateDate`, so Group0 is `QuickTime`.
const GROUP: &str = "QuickTime";

/// Which container's children we are iterating. Each variant's `match` arm cites
/// the QuickTime.pm SubDirectory table it mirrors.
#[derive(Clone, Copy, Debug)]
enum Container {
    /// File top level → QuickTime::Main (QuickTime.pm:548): `moov`.
    TopLevel,
    /// `moov` → QuickTime::Movie (QuickTime.pm:1201): `mvhd`, `trak`, `udta`, `meta`.
    Movie,
    /// `trak` → QuickTime::Track (QuickTime.pm:1424): `tkhd`, `mdia`, `udta`, `meta`.
    Track,
    /// `mdia` → QuickTime::Media (QuickTime.pm:7218): `mdhd`, `hdlr`, `minf`.
    Media,
    /// `minf` → QuickTime::MediaInfo (QuickTime.pm:7289): `stbl` (→ stsd, Task 3).
    MediaInfo,
    /// `stbl` → QuickTime::SampleTable (QuickTime.pm:7365): `stsd` (Task 3).
    SampleTable,
}

/// How a duplicate tag across tracks resolves. TrackHeader entries carry
/// `Priority => 0` so the FIRST track wins; MediaHeader/Handler use the default
/// priority so the LAST value wins (ExifTool FoundTag:9536-9588). Getting this
/// wrong lets an audio track's zero dimensions clobber the video track's.
#[derive(Clone, Copy)]
enum Priority {
    /// FIRST occurrence wins (TrackHeader, Priority => 0).
    First,
    /// LAST occurrence wins (MovieHeader / MediaHeader default).
    Last,
}

/// Walk a QuickTime container and return its `QuickTime:*` TagEntries.
///
/// Best-effort: malformed or truncated atoms stop the walk (never panic) and we
/// return whatever was decoded so far. Only hard reader I/O errors propagate.
pub fn extract_quicktime_metadata<R: Read + Seek>(reader: &mut R) -> Result<Vec<TagEntry>> {
    let file_end = reader.seek(SeekFrom::End(0))?;
    let mut walker = Walker {
        reader,
        tags: IndexMap::new(),
        time_scale: None,
        handler_type: None,
    };
    walker.process(Container::TopLevel, 0, file_end, 0)?;
    Ok(walker.into_entries())
}

struct Walker<'a, R: Read + Seek> {
    reader: &'a mut R,
    /// Accumulated tags keyed by name (Group0 is always QuickTime), already
    /// priority-resolved so exactly one value per tag reaches the output.
    tags: IndexMap<String, TagEntry>,
    /// `$$self{TimeScale}` — the *movie* timescale from mvhd (MovieHeader idx 3).
    /// Used for both Duration and TrackDuration (%durationInfo:314).
    time_scale: Option<u32>,
    /// `$$self{HandlerType}` from the most recent hdlr (Handler idx 8). Captured
    /// for Task 4's Rotation (needs the first `vide` track's MatrixStructure).
    handler_type: Option<[u8; 4]>,
}

impl<R: Read + Seek> Walker<'_, R> {
    /// Iterate the atoms in `[start, end)` of the given container kind, recursing
    /// into the containers this task cares about and seeking past everything else
    /// (including `mdat`). Mirrors the ProcessMOV loop (QuickTime.pm:10033-10090).
    fn process(&mut self, kind: Container, start: u64, end: u64, depth: u32) -> Result<()> {
        if depth > MAX_DEPTH {
            trace!("quicktime: max atom depth reached, stopping");
            return Ok(());
        }
        let mut pos = start;
        while pos + 8 <= end {
            self.reader.seek(SeekFrom::Start(pos))?;
            let mut header = [0u8; 8];
            if read_full(self.reader, &mut header)?.is_none() {
                break; // truncated header
            }
            let size32 = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
            let atom_type = [header[4], header[5], header[6], header[7]];

            // Atom size + header size. ExifTool QuickTime.pm:10036-10090:
            //   size == 1 → 64-bit extended size in the next 8 bytes;
            //   size == 0 → atom runs to the end of the file/container.
            let (atom_size, header_size): (u64, u64) = match size32 {
                1 => {
                    let mut ext = [0u8; 8];
                    if read_full(self.reader, &mut ext)?.is_none() {
                        break; // truncated extended size
                    }
                    (u64::from_be_bytes(ext), 16)
                }
                0 => (end.saturating_sub(pos), 8),
                n => (n as u64, 8),
            };

            // Reject malformed sizes rather than panicking or looping forever.
            if atom_size < header_size {
                trace!(
                    "quicktime: atom '{}' size {} < header {}, stopping",
                    fourcc(&atom_type),
                    atom_size,
                    header_size
                );
                break;
            }
            let content_start = pos + header_size;
            let atom_end = match pos.checked_add(atom_size) {
                Some(e) if e <= end => e,
                _ => {
                    trace!(
                        "quicktime: atom '{}' extends past container, stopping",
                        fourcc(&atom_type)
                    );
                    break;
                }
            };
            let content_len = atom_end - content_start;

            self.dispatch(
                kind,
                &atom_type,
                content_start,
                content_len,
                atom_end,
                depth,
            )?;

            if size32 == 0 {
                break; // ran to end
            }
            pos = atom_end;
        }
        Ok(())
    }

    /// Route one atom: recurse into containers, decode leaf boxes, or fall through
    /// (seek past). Each arm cites the QuickTime.pm table it implements.
    fn dispatch(
        &mut self,
        kind: Container,
        atom_type: &[u8; 4],
        content_start: u64,
        content_len: u64,
        atom_end: u64,
        depth: u32,
    ) -> Result<()> {
        match (kind, atom_type) {
            // ----- containers -----
            (Container::TopLevel, b"moov") => {
                self.process(Container::Movie, content_start, atom_end, depth + 1)?;
            }
            (Container::Movie, b"mvhd") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_mvhd(&buf);
            }
            (Container::Movie, b"trak") => {
                self.process(Container::Track, content_start, atom_end, depth + 1)?;
            }
            (Container::Track, b"tkhd") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_tkhd(&buf);
            }
            (Container::Track, b"mdia") => {
                self.process(Container::Media, content_start, atom_end, depth + 1)?;
            }
            (Container::Media, b"mdhd") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_mdhd(&buf);
            }
            (Container::Media, b"hdlr") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_hdlr(&buf);
            }
            (Container::Media, b"minf") => {
                self.process(Container::MediaInfo, content_start, atom_end, depth + 1)?;
            }
            (Container::MediaInfo, b"stbl") => {
                self.process(Container::SampleTable, content_start, atom_end, depth + 1)?;
            }

            // ----- TODO arms (structure ready for later tasks) -----
            // Task 3: moov/meta (bare container, Movie:1218) + moov/trak/meta →
            //   keys/ilst indirection (ProcessKeys:9779) → Make/Model/Software/
            //   CreationDate/GPSCoordinates/LensModel.
            (Container::Movie | Container::Track, b"meta") => {
                trace!("quicktime: TODO Task 3 meta/keys/ilst");
            }
            // Task 3/5: udta (UserData:1585) → XMP_ atom:1711, Canon CNTH, etc.
            (Container::Movie | Container::Track, b"udta") => {
                trace!("quicktime: TODO Task 3/5 udta");
            }
            // Task 3: stsd (SampleTable:7365 → ProcessSampleDesc:9629 →
            //   VisualSampleDesc:7585) → CompressorName when HandlerType == 'vide'.
            (Container::SampleTable, b"stsd") => {
                trace!("quicktime: TODO Task 3 stsd/CompressorName");
            }

            // ----- everything else (incl. mdat): seek past, never read -----
            _ => {
                trace!(
                    "quicktime: skipping '{}' ({} bytes) in {:?}",
                    fourcc(atom_type),
                    content_len,
                    kind
                );
            }
        }
        Ok(())
    }

    /// Read a small leaf-atom payload into memory. Only ever called for header
    /// boxes (< a few hundred bytes), never for `mdat`.
    fn read_content(&mut self, start: u64, len: u64) -> Result<Vec<u8>> {
        // Defensive cap: header boxes are tiny; refuse to allocate for a bogus
        // length (a fuzzer can claim a huge mvhd).
        const MAX_LEAF: u64 = 64 * 1024;
        let len = len.min(MAX_LEAF) as usize;
        self.reader.seek(SeekFrom::Start(start))?;
        let mut buf = vec![0u8; len];
        match read_full(self.reader, &mut buf)? {
            Some(()) => Ok(buf),
            None => {
                // Truncated: return what fits so decoders' length checks bail out.
                buf.clear();
                Ok(buf)
            }
        }
    }

    // ----- leaf decoders (byte layouts follow the ProcessBinaryData tables;
    //       version-1 boxes shift the date/duration fields to int64u, ExifTool
    //       Hook `$format = "int64u", $varSize += 4`) -----

    /// mvhd → QuickTime::MovieHeader (QuickTime.pm:1343). Yields CreateDate (idx
    /// 1), ModifyDate (idx 2), sets `$$self{TimeScale}` (idx 3), Duration (idx 4).
    fn decode_mvhd(&mut self, p: &[u8]) {
        let Some(version) = p.first().copied() else {
            return;
        };
        let (create, modify, timescale, duration) = if version == 0 {
            if p.len() < 20 {
                return;
            }
            (
                be_u32(p, 4) as u64,
                be_u32(p, 8) as u64,
                be_u32(p, 12),
                be_u32(p, 16) as u64,
            )
        } else {
            // version 1: 64-bit dates/duration (Hook, MovieHeader:1373/1380/1390).
            if p.len() < 32 {
                return;
            }
            (be_u64(p, 4), be_u64(p, 12), be_u32(p, 20), be_u64(p, 24))
        };
        // MovieHeader idx 3 RawConv `$$self{TimeScale} = $val`.
        self.time_scale = Some(timescale);
        // mvhd is unique within moov → Last (plain set).
        self.add_date("CreateDate", create, Priority::Last);
        self.add_date("ModifyDate", modify, Priority::Last);
        self.add_duration("Duration", duration, Some(timescale), Priority::Last);
    }

    /// tkhd → QuickTime::TrackHeader (QuickTime.pm:1493). All entries are
    /// `Priority => 0` (FIRST track wins). Yields TrackCreateDate (idx 1),
    /// TrackModifyDate (idx 2), TrackDuration (idx 5, uses the *movie* TimeScale),
    /// ImageWidth (idx 19), ImageHeight (idx 20). MatrixStructure (idx 10) is
    /// captured for Task 4's Rotation (TODO).
    fn decode_tkhd(&mut self, p: &[u8]) {
        let Some(version) = p.first().copied() else {
            return;
        };
        let (create, modify, duration, width, height) = if version == 0 {
            if p.len() < 84 {
                return;
            }
            (
                be_u32(p, 4) as u64,
                be_u32(p, 8) as u64,
                be_u32(p, 20) as u64,
                be_u32(p, 76),
                be_u32(p, 80),
            )
        } else {
            // version 1: create/modify/duration are int64u (varSize += 4 each), so
            // MatrixStructure/ImageWidth/ImageHeight shift by 12 bytes.
            if p.len() < 96 {
                return;
            }
            (
                be_u64(p, 4),
                be_u64(p, 12),
                be_u64(p, 28),
                be_u32(p, 88),
                be_u32(p, 92),
            )
        };
        self.add_date("TrackCreateDate", create, Priority::First);
        self.add_date("TrackModifyDate", modify, Priority::First);
        // TrackDuration divides by the movie TimeScale, not a per-track one.
        self.add_duration("TrackDuration", duration, self.time_scale, Priority::First);
        // ImageWidth/Height: FixWrongFormat (QuickTime.pm:8872) — 0 → no tag, so
        // an audio track (0×0) never overrides the video track's dimensions.
        if let Some(w) = qt::fix_wrong_format(width) {
            self.add_scalar("ImageWidth", TagValue::U32(w), Priority::First);
        }
        if let Some(h) = qt::fix_wrong_format(height) {
            self.add_scalar("ImageHeight", TagValue::U32(h), Priority::First);
        }
        // TODO Task 4: capture MatrixStructure (idx 10, fixed32s[9] at
        // byte 40 v0 / 52 v1) paired with this track's HandlerType for Rotation
        // (CalcRotation:8797 uses the first vide track's matrix).
    }

    /// mdhd → QuickTime::MediaHeader (QuickTime.pm:7239). Default priority (LAST
    /// track wins). Yields MediaCreateDate (idx 1), MediaModifyDate (idx 2),
    /// MediaTimeScale (idx 3, `$$self{MediaTS}`), MediaDuration (idx 4, divided by
    /// this same mdhd's MediaTS via RawConv:7270).
    fn decode_mdhd(&mut self, p: &[u8]) {
        let Some(version) = p.first().copied() else {
            return;
        };
        let (create, modify, media_ts, duration) = if version == 0 {
            if p.len() < 20 {
                return;
            }
            (
                be_u32(p, 4) as u64,
                be_u32(p, 8) as u64,
                be_u32(p, 12),
                be_u32(p, 16) as u64,
            )
        } else {
            if p.len() < 32 {
                return;
            }
            (be_u64(p, 4), be_u64(p, 12), be_u32(p, 20), be_u64(p, 24))
        };
        self.add_date("MediaCreateDate", create, Priority::Last);
        self.add_date("MediaModifyDate", modify, Priority::Last);
        // MediaDuration uses the SAME mdhd's MediaTS (MediaHeader:7270-7271).
        self.add_duration("MediaDuration", duration, Some(media_ts), Priority::Last);
    }

    /// hdlr → QuickTime::Handler (QuickTime.pm:8391). Task 2 only needs the
    /// HandlerType (idx 8, `undef[4]` at byte offset 8) as `$$self{HandlerType}`
    /// state for Task 4's Rotation. HandlerDescription (idx 24) is a supported tag
    /// but its Pascal/C-string RawConv (Handler:8457) lands in a later task.
    fn decode_hdlr(&mut self, p: &[u8]) {
        // Handler has no table FORMAT, so keys are byte offsets: HandlerType at 8.
        if p.len() >= 12 {
            let mut ht = [0u8; 4];
            ht.copy_from_slice(&p[8..12]);
            // ExifTool skips 'alis'/'url ' when recording HandlerType (Handler:8412).
            if &ht != b"alis" && &ht != b"url " {
                self.handler_type = Some(ht);
            }
        }
        // TODO Task 2+: HandlerDescription (byte 24, string, Handler:8453) with the
        // leading-length-byte Pascal-string strip (RawConv:8457).
    }

    // ----- tag accumulation with priority-resolved dedup -----

    fn add_date(&mut self, name: &str, raw: u64, priority: Priority) {
        // %timeInfo RawConv patch (QuickTime.pm:257) + ConvertUnixTime ValueConv
        // (ExifTool.pm:6784); PrintConv ConvertDateTime is identity without `-d`,
        // so value == print. MOV: to_local = false (no -api QuickTimeUTC).
        let unix = qt::patch_time_zero(raw as i64);
        let value = TagValue::string(qt::convert_unix_time(unix, false));
        self.insert(name, value.clone(), value, priority);
    }

    fn add_duration(&mut self, name: &str, raw: u64, timescale: Option<u32>, priority: Priority) {
        // %durationInfo ValueConv `$val / $$self{TimeScale}` then PrintConv
        // ConvertDuration (QuickTime.pm:314-315). If the timescale is unset/zero
        // ExifTool passes the raw value through unchanged.
        let (value, print) = match timescale {
            Some(ts) if ts != 0 => {
                let seconds = raw as f64 / ts as f64;
                (
                    TagValue::F64(seconds),
                    TagValue::string(qt::convert_duration(seconds)),
                )
            }
            _ => (TagValue::U64(raw), TagValue::U64(raw)),
        };
        self.insert(name, value, print, priority);
    }

    fn add_scalar(&mut self, name: &str, value: TagValue, priority: Priority) {
        self.insert(name, value.clone(), value, priority);
    }

    /// Insert one tag, resolving duplicates by [`Priority`]: `First` keeps the
    /// existing value (TrackHeader Priority => 0), `Last` overwrites (default).
    fn insert(&mut self, name: &str, value: TagValue, print: TagValue, priority: Priority) {
        let entry = TagEntry {
            group: GROUP.to_string(),
            group1: GROUP.to_string(),
            name: name.to_string(),
            value,
            print,
        };
        match priority {
            Priority::First => {
                self.tags.entry(name.to_string()).or_insert(entry);
            }
            Priority::Last => {
                self.tags.insert(name.to_string(), entry);
            }
        }
    }

    fn into_entries(self) -> Vec<TagEntry> {
        self.tags.into_values().collect()
    }
}

/// Read exactly `buf.len()` bytes. Returns `Ok(Some(()))` on success,
/// `Ok(None)` on clean EOF/short read (truncated atom — stop, don't panic), and
/// propagates only genuine I/O errors.
fn read_full<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<Option<()>> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => return Ok(None),
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(Some(()))
}

#[inline]
fn be_u32(p: &[u8], off: usize) -> u32 {
    u32::from_be_bytes([p[off], p[off + 1], p[off + 2], p[off + 3]])
}

#[inline]
fn be_u64(p: &[u8], off: usize) -> u64 {
    u64::from_be_bytes([
        p[off],
        p[off + 1],
        p[off + 2],
        p[off + 3],
        p[off + 4],
        p[off + 5],
        p[off + 6],
        p[off + 7],
    ])
}

fn fourcc(t: &[u8; 4]) -> String {
    String::from_utf8_lossy(t).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn empty_reader_yields_nothing() {
        let mut r = Cursor::new(Vec::<u8>::new());
        let tags = extract_quicktime_metadata(&mut r).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn truncated_atom_header_does_not_panic() {
        // A 'moov' claiming 1000 bytes but only 4 bytes of body present.
        let mut data = Vec::new();
        data.extend_from_slice(&1000u32.to_be_bytes());
        data.extend_from_slice(b"moov");
        data.extend_from_slice(&[0, 0, 0, 0]);
        let mut r = Cursor::new(data);
        // Must not panic; returns no tags (atom extends past container → stop).
        let tags = extract_quicktime_metadata(&mut r).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn zero_size_atom_terminates() {
        // size == 0 runs to EOF; an unknown top-level atom just ends the walk.
        let mut data = Vec::new();
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(b"free");
        data.extend_from_slice(&[0xAA; 16]);
        let mut r = Cursor::new(data);
        let tags = extract_quicktime_metadata(&mut r).unwrap();
        assert!(tags.is_empty());
    }

    fn atom(fourcc: &[u8; 4], content: &[u8]) -> Vec<u8> {
        let mut a = Vec::with_capacity(8 + content.len());
        a.extend_from_slice(&((content.len() as u32) + 8).to_be_bytes());
        a.extend_from_slice(fourcc);
        a.extend_from_slice(content);
        a
    }

    /// Version-1 (64-bit date) box layouts: none of the 5 committed MOV
    /// snapshots use v1 boxes, so the review gate (2026-07-03) required this
    /// in-memory fixture to pin the shifted byte offsets (ExifTool Hook
    /// `$format = "int64u", $varSize += 4`; TrackHeader:1512).
    ///
    /// Expected values cross-checked against vendored exiftool on an identical
    /// hand-built file: CreateDate 2020:01:01 00:00:00 (raw 3660681600 =
    /// unix 1577836800 + 2082844800), Duration 10.00 s (10000/1000).
    #[test]
    fn version1_boxes_decode_64bit_dates_and_shifted_dimensions() {
        const CREATE: u64 = 3_660_681_600; // 2020:01:01 00:00:00 UTC, 1904 epoch
        const MODIFY: u64 = CREATE + 3600;

        // mvhd v1: ver/flags(4) create(8) modify(8) timescale(4) duration(8) = 32
        let mut mvhd = vec![1u8, 0, 0, 0];
        mvhd.extend_from_slice(&CREATE.to_be_bytes());
        mvhd.extend_from_slice(&MODIFY.to_be_bytes());
        mvhd.extend_from_slice(&1000u32.to_be_bytes());
        mvhd.extend_from_slice(&10_000u64.to_be_bytes());

        // tkhd v1: ver/flags(4) create(8) modify(8) trackID(4) reserved(4)
        // duration(8) reserved(8) layer/alt/vol/reserved(8) matrix(36)
        // width(4)@88 height(4)@92 = 96
        let mut tkhd = vec![1u8, 0, 0, 0];
        tkhd.extend_from_slice(&CREATE.to_be_bytes());
        tkhd.extend_from_slice(&MODIFY.to_be_bytes());
        tkhd.extend_from_slice(&1u32.to_be_bytes());
        tkhd.extend_from_slice(&[0u8; 4]);
        tkhd.extend_from_slice(&10_000u64.to_be_bytes());
        tkhd.extend_from_slice(&[0u8; 16]); // reserved + layer/alt/volume/reserved
        tkhd.extend_from_slice(&[0u8; 36]); // matrix
        tkhd.extend_from_slice(&(1920u32 << 16).to_be_bytes()); // 16.16 fixed
        tkhd.extend_from_slice(&(1080u32 << 16).to_be_bytes());
        assert_eq!(tkhd.len(), 96);

        // mdhd v1: ver/flags(4) create(8) modify(8) timescale(4) duration(8) = 32
        let mut mdhd = vec![1u8, 0, 0, 0];
        mdhd.extend_from_slice(&CREATE.to_be_bytes());
        mdhd.extend_from_slice(&MODIFY.to_be_bytes());
        mdhd.extend_from_slice(&500u32.to_be_bytes());
        mdhd.extend_from_slice(&5_000u64.to_be_bytes());

        let mdia = atom(b"mdia", &atom(b"mdhd", &mdhd));
        let trak = atom(b"trak", &[atom(b"tkhd", &tkhd), mdia].concat());
        let moov = atom(b"moov", &[atom(b"mvhd", &mvhd), trak].concat());

        let mut r = Cursor::new(moov);
        let tags = extract_quicktime_metadata(&mut r).unwrap();
        let get = |name: &str| {
            tags.iter()
                .find(|t| t.name == name)
                .unwrap_or_else(|| panic!("missing {name}"))
        };

        assert_eq!(get("CreateDate").print.to_string(), "2020:01:01 00:00:00");
        assert_eq!(get("ModifyDate").print.to_string(), "2020:01:01 01:00:00");
        assert_eq!(get("Duration").print.to_string(), "10.00 s");
        assert_eq!(
            get("TrackCreateDate").print.to_string(),
            "2020:01:01 00:00:00"
        );
        assert_eq!(get("TrackDuration").print.to_string(), "10.00 s");
        // MediaDuration divides by the SAME mdhd's timescale (500), not mvhd's.
        assert_eq!(get("MediaDuration").print.to_string(), "10.00 s");
        assert_eq!(
            get("MediaCreateDate").print.to_string(),
            "2020:01:01 00:00:00"
        );
        // v1 shifts width/height to bytes 88/92; 16.16 fixed via FixWrongFormat.
        assert_eq!(get("ImageWidth").value, TagValue::U32(1920));
        assert_eq!(get("ImageHeight").value, TagValue::U32(1080));
    }
}
