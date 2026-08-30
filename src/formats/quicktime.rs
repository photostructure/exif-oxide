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
//! Scope through TPP Task 3: mvhd/tkhd/mdhd dates, durations and dimensions;
//! hdlr/stsd descriptions; meta/keys/ilst Apple metadata; direct UserData Model;
//! and observable HandlerType/MatrixStructure/GPSCoordinates/LensModel state for
//! Task 4. GPS ISO6709 conversion, Rotation composites, and embedded XMP remain
//! later tasks.

use std::io::{Read, Seek, SeekFrom};

use indexmap::IndexMap;
use tracing::{trace, warn};

use crate::generated::QuickTime_pm::{
    audio_keys_tags::QUICK_TIME_AUDIOKEYS_TAGS_BY_NAME, handler_tags::QUICK_TIME_HANDLER_TAGS,
    item_list_tags::QUICK_TIME_ITEMLIST_TAGS_BY_NAME, keys_tags::QUICK_TIME_KEYS_TAGS_BY_NAME,
    user_data_tags::QUICK_TIME_USERDATA_TAGS_BY_NAME,
    video_keys_tags::QUICK_TIME_VIDEOKEYS_TAGS_BY_NAME,
    visual_sample_desc_tags::QUICK_TIME_VISUALSAMPLEDESC_TAGS,
};
use crate::implementations::quicktime as qt;
use crate::types::{PrintConv, Result, TagEntry, TagInfo, TagValue, ValueConv};

/// Guard against pathologically deep / cyclic atom nesting (fuzz target, Task 5).
const MAX_DEPTH: u32 = 16;

/// ExifTool starts skipping ordinary atoms above 32 MiB (ProcessMOV:10167-10191).
/// We stream containers and only buffer supported leaf values, but retain the
/// same threshold for a corrupt known leaf claiming an unreasonable size.
const LARGE_ATOM_THRESHOLD: u64 = 0x0200_0000;

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
    /// `minf` → QuickTime::MediaInfo (QuickTime.pm:7289): `stbl`, `hdlr` (data handler).
    MediaInfo,
    /// `stbl` → QuickTime::SampleTable (QuickTime.pm:7365): `stsd` (CompressorName).
    SampleTable,
    /// `meta` → QuickTime::Meta (QuickTime.pm:2810): `hdlr`, `keys`, `ilst`.
    /// A BARE container inside moov/trak (Movie:1218 has no `Start`), so there is
    /// no leading version/flags to skip — its content starts at the first child atom.
    Meta,
    /// `ilst` children: direct FourCC entries or 1-based numeric Keys indices.
    ItemList,
    /// One ItemList entry's child atoms; only `data` is decoded.
    ItemData,
    /// `udta` → QuickTime::UserData (QuickTime.pm:1585): manufacturer atoms whose
    /// 4-byte ID keys the UserData table directly (e.g. Canon `CNMN` → Model).
    UserData,
}

/// How a duplicate tag across tracks resolves. TrackHeader dates/duration/
/// dimensions explicitly carry `Priority => 0` so the FIRST track wins;
/// MatrixStructure, MediaHeader and Handler use the default LAST-wins priority
/// (ExifTool FoundTag:9536-9588).
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
        media_type: None,
        current_keys: Vec::new(),
        current_item: None,
        video_matrix: None,
        pending_matrix: None,
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
    /// `$$self{HandlerType}` from the most recent non-`alis`/`url ` hdlr. This
    /// drives stsd dispatch and is reset when leaving `minf` (ProcessMOV:10370).
    handler_type: Option<[u8; 4]>,
    /// `$$self{MediaType}` — the current track's media handler type, set ONLY by a
    /// `trak/mdia/hdlr` (Handler idx 8, whose PATH parent is `Media`; QuickTime.pm:8413).
    /// Drives `keys` table selection (Meta:2867-2878). The `stsd` gate uses the
    /// separate HandlerType state. `meta`/`minf` data handlers do not touch
    /// MediaType (their parent isn't `Media`), matching ExifTool.
    media_type: Option<[u8; 4]>,
    /// Resolved tag for each 1-based index of the most recent `keys` atom, so the
    /// following sibling `ilst` can map its numeric item IDs back to tags
    /// (ProcessKeys:9857 stores `KeysCount.index`; ilst re-keys via
    /// `KeysCount . '.' . unpack('N',$tag)` at ProcessMOV:10132 — within one `meta`
    /// the index is simply local).
    current_keys: Vec<Option<&'static TagInfo>>,
    /// TagInfo for the ItemList entry currently being streamed.
    current_item: Option<&'static TagInfo>,
    /// The first `vide` track's tkhd MatrixStructure (ValueConv-applied string),
    /// captured for Task 4's Rotation (CalcRotation:8797 uses the video track matrix).
    video_matrix: Option<String>,
    /// tkhd MatrixStructure of the track currently being walked, pending pairing
    /// with that track's media handler type (hdlr follows tkhd inside a `trak`).
    pending_matrix: Option<String>,
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
            // Main:552 and UserData:1687 declare `Start => 4`: these `meta`
            // atoms are FullBoxes and their child stream follows version/flags.
            (Container::TopLevel, b"meta") => {
                self.process_meta_fullbox(content_start, content_len, atom_end, depth)?;
            }
            (Container::Movie, b"mvhd") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_mvhd(&buf);
            }
            (Container::Movie, b"trak") => {
                // New track: MediaType is scoped to the track and reset at its end
                // (ProcessMOV:10596); HandlerType is independently scoped by minf.
                let outer_media_type = self.media_type.take();
                let outer_handler_type = self.handler_type.take();
                self.pending_matrix = None;
                self.process(Container::Track, content_start, atom_end, depth + 1)?;
                self.media_type = outer_media_type;
                self.handler_type = outer_handler_type;
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
            // trak/mdia/hdlr: the *media* handler — sets `$$self{MediaType}` (its
            // PATH parent is `Media`, QuickTime.pm:8413) and yields HandlerDescription.
            (Container::Media, b"hdlr") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_hdlr(&buf, true);
            }
            (Container::Media, b"minf") => {
                self.process(Container::MediaInfo, content_start, atom_end, depth + 1)?;
                // QuickTime.pm:10370 resets HandlerType when exiting minf.
                self.handler_type = None;
            }
            // trak/mdia/minf/hdlr: the *data* handler (alis) — yields
            // HandlerDescription "Core Media Data Handler" but must NOT set MediaType.
            (Container::MediaInfo, b"hdlr") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_hdlr(&buf, false);
            }
            (Container::MediaInfo, b"stbl") => {
                self.process(Container::SampleTable, content_start, atom_end, depth + 1)?;
            }

            // ----- meta / keys / ilst (ProcessKeys:9779, ItemList data boxes:10380) -----
            // `meta` is a bare container inside moov or trak (Movie:1218 / Track has
            // no Start): no version/flags to skip — recurse straight into its children.
            (Container::Movie | Container::Track, b"meta") => {
                self.process_meta(content_start, atom_end, depth + 1)?;
            }
            // meta/hdlr is the 'mdta' metadata handler; decode its (usually empty)
            // HandlerDescription but leave MediaType alone (parent is `Meta`).
            (Container::Meta, b"hdlr") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_hdlr(&buf, false);
            }
            (Container::Meta, b"keys") => {
                let buf = self.read_content(content_start, content_len)?;
                self.parse_keys(&buf);
            }
            (Container::Meta, b"ilst") => {
                self.process(Container::ItemList, content_start, atom_end, depth + 1)?;
            }
            (Container::ItemList, _) => {
                if let Some(info) = self.resolve_ilst_item(atom_type) {
                    let outer_item = self.current_item.replace(info);
                    self.process(Container::ItemData, content_start, atom_end, depth + 1)?;
                    self.current_item = outer_item;
                }
            }
            (Container::ItemData, b"data") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_item_data(&buf)?;
            }

            // ----- udta (UserData:1585): Canon CNMN => Model, CNCV/CNFV strings.
            //   XMP_ / CNTH etc. are SubDirectories (format "unknown") and are left
            //   for Task 5 / out of scope — decode_userdata_atom skips them. -----
            (Container::Movie | Container::Track, b"udta") => {
                self.process(Container::UserData, content_start, atom_end, depth + 1)?;
            }
            (Container::UserData, b"meta") => {
                self.process_meta_fullbox(content_start, content_len, atom_end, depth)?;
            }
            (Container::UserData, _) => {
                self.decode_userdata_atom(atom_type, content_start, content_len)?;
            }

            // ----- stsd (SampleTable:7365 → ProcessSampleDesc:9629 →
            //   VisualSampleDesc:7585): CompressorName idx 25, only when the track's
            //   media handler is 'vide' (SampleTable stsd Condition:7380). -----
            (Container::SampleTable, b"stsd") => {
                let buf = self.read_content(content_start, content_len)?;
                self.decode_stsd(&buf);
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

    /// Read a leaf-atom / small-container payload into memory. Called for header
    /// boxes and the keys/ilst/stsd/udta-child metadata boxes, never for `mdat`.
    fn read_content(&mut self, start: u64, len: u64) -> Result<Vec<u8>> {
        if len > LARGE_ATOM_THRESHOLD {
            // ProcessMOV:10167-10191 skips ordinary known atoms above 32 MiB.
            warn!("quicktime: skipping metadata atom larger than 32 MiB");
            return Ok(Vec::new());
        }
        let len = len as usize;
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

    /// Enter one bare `meta` directory with a fresh ProcessKeys registry. ExifTool
    /// increments KeysCount per directory; a local vector is equivalent for the
    /// 1-based ilst indices and prevents sibling meta directories leaking keys.
    fn process_meta(&mut self, start: u64, end: u64, depth: u32) -> Result<()> {
        let outer_keys = std::mem::take(&mut self.current_keys);
        self.process(Container::Meta, start, end, depth)?;
        self.current_keys = outer_keys;
        Ok(())
    }

    /// Main:552 and UserData:1687 `meta` atoms are FullBoxes (`Start => 4`).
    fn process_meta_fullbox(
        &mut self,
        content_start: u64,
        content_len: u64,
        atom_end: u64,
        depth: u32,
    ) -> Result<()> {
        if content_len < 4 {
            return Ok(());
        }
        self.process_meta(content_start + 4, atom_end, depth + 1)
    }

    /// ProcessKeys (QuickTime.pm:9779-9877): read entries after version/count,
    /// strip the mdta Apple domain, resolve through Keys then ItemList/UserData,
    /// and store each result at its 1-based index.
    fn parse_keys(&mut self, p: &[u8]) {
        self.current_keys.clear();
        if p.len() < 8 {
            return;
        }
        let mut pos = 8usize;
        while pos + 4 < p.len() {
            let len = be_u32(p, pos) as usize;
            let Some(end) = pos.checked_add(len) else {
                break;
            };
            if len < 8 || end > p.len() {
                break;
            }
            let namespace = &p[pos + 4..pos + 8];
            let mut full = &p[pos + 8..end];
            if let Some(nul) = full.iter().position(|byte| *byte == 0) {
                full = &full[..nul];
            }
            let short = if namespace == b"mdta" {
                full.strip_prefix(b"com.apple.quicktime.")
                    .or_else(|| full.strip_prefix(b"com."))
                    .unwrap_or(full)
            } else {
                full
            };
            let info = self.resolve_key_name(short).or_else(|| {
                (short != full)
                    .then(|| self.resolve_key_name(full))
                    .flatten()
            });
            self.current_keys.push(info);
            pos = end;
        }
    }

    fn resolve_key_name(&self, key: &[u8]) -> Option<&'static TagInfo> {
        let primary = match self.media_type.as_ref() {
            Some(b"soun") => &*QUICK_TIME_AUDIOKEYS_TAGS_BY_NAME,
            Some(b"vide") => &*QUICK_TIME_VIDEOKEYS_TAGS_BY_NAME,
            _ => &*QUICK_TIME_KEYS_TAGS_BY_NAME,
        };
        lookup_key_tables(primary, key).or_else(|| {
            reversed_copyright_fourcc(key)
                .and_then(|reversed| lookup_key_tables(primary, &reversed))
        })
    }

    fn resolve_ilst_item(&self, atom_type: &[u8; 4]) -> Option<&'static TagInfo> {
        // ProcessMOV:10130-10135 first tries the direct ItemList ID, then the
        // ProcessKeys-generated 1-based numeric ID.
        QUICK_TIME_ITEMLIST_TAGS_BY_NAME
            .get(atom_type.as_slice())
            .or_else(|| {
                let index = u32::from_be_bytes(*atom_type);
                index
                    .checked_sub(1)
                    .and_then(|zero_based| self.current_keys.get(zero_based as usize))
                    .copied()
                    .flatten()
            })
    }

    /// Decode one ItemList `data` FullBox payload. The atom header itself was
    /// consumed by the walker, leaving flags/country/language plus value bytes.
    fn decode_item_data(&mut self, p: &[u8]) -> Result<()> {
        let Some(info) = self.current_item.cloned() else {
            return Ok(());
        };
        if !is_task3_item(info.name) || p.len() < 8 {
            return Ok(());
        }
        let flags = be_u32(p, 0);
        // QuickTime.pm:10393: 0x1 and 0x4 are UTF-8. Other encodings/formats are
        // deliberately deferred; warn and skip instead of guessing.
        if flags != 0x1 && flags != 0x4 {
            warn!(
                "quicktime: skipping unsupported ilst data format 0x{:x} for {}",
                flags, info.name
            );
            return Ok(());
        }
        let mut bytes = &p[8..];
        if bytes.last() == Some(&0) {
            bytes = &bytes[..bytes.len() - 1];
        }
        let raw = TagValue::string(String::from_utf8_lossy(bytes));
        self.add_from_tag_info(&info, raw, Priority::Last)
    }

    /// UserData direct string atoms needed by Task 3 (notably Canon CNMN Model).
    /// Subdirectories such as XMP_/CNTH remain untouched for Task 5.
    fn decode_userdata_atom(
        &mut self,
        atom_type: &[u8; 4],
        content_start: u64,
        content_len: u64,
    ) -> Result<()> {
        let Some(info) = QUICK_TIME_USERDATA_TAGS_BY_NAME.get(atom_type.as_slice()) else {
            return Ok(());
        };
        if info.name != "Model" || !info.format.starts_with("string") {
            return Ok(());
        }
        let buf = self.read_content(content_start, content_len)?;
        let Some(value) = nul_terminated_string(&buf) else {
            return Ok(());
        };
        self.add_from_tag_info(info, TagValue::string(value), Priority::Last)
    }

    /// ProcessSampleDesc (QuickTime.pm:9629): stsd FullBox header/count followed
    /// by bounded sample entries. VisualSampleDesc idx 25 is byte offset 50.
    fn decode_stsd(&mut self, p: &[u8]) {
        if self.handler_type.as_ref() != Some(b"vide") || p.len() < 8 {
            return;
        }
        let count = be_u32(p, 4) as usize;
        let mut pos = 8usize;
        for _ in 0..count {
            if pos + 8 > p.len() {
                break;
            }
            let size = be_u32(p, pos) as usize;
            let Some(end) = pos.checked_add(size) else {
                break;
            };
            if size < 8 || end > p.len() {
                break;
            }
            if size >= 82 {
                if let Some(name) = strip_length_prefixed_string(&p[pos + 50..pos + 82]) {
                    let tag_name =
                        table_name(&QUICK_TIME_VISUALSAMPLEDESC_TAGS, 25, "CompressorName");
                    self.add_scalar(tag_name, TagValue::string(name), Priority::Last);
                }
            }
            pos = end;
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

    /// tkhd → QuickTime::TrackHeader (QuickTime.pm:1493). Dates, duration and
    /// dimensions are `Priority => 0` (FIRST track wins). Yields TrackCreateDate (idx 1),
    /// TrackModifyDate (idx 2), TrackDuration (idx 5, uses the *movie* TimeScale),
    /// ImageWidth (idx 19), ImageHeight (idx 20). MatrixStructure (idx 10) is
    /// emitted and paired with the video track for Task 4's Rotation.
    fn decode_tkhd(&mut self, p: &[u8]) {
        let Some(version) = p.first().copied() else {
            return;
        };
        // MatrixStructure (idx 10, fixed32s[9]) sits at byte 40 (v0) / 52 (v1),
        // just before ImageWidth/Height. See offsets below.
        let (create, modify, duration, matrix_off, width, height) = if version == 0 {
            if p.len() < 84 {
                return;
            }
            (
                be_u32(p, 4) as u64,
                be_u32(p, 8) as u64,
                be_u32(p, 20) as u64,
                40usize,
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
                52usize,
                be_u32(p, 88),
                be_u32(p, 92),
            )
        };
        // MatrixStructure ValueConv (TrackHeader:1561-1565): fixed32s[9] with the
        // right column (idx 2,5,8) further divided by 0x4000. Held pending pairing
        // with this track's media handler type (for Task 4's Rotation).
        self.pending_matrix = matrix_structure(&p[matrix_off..matrix_off + 36]);
        if let Some(matrix) = self.pending_matrix.clone() {
            // MatrixStructure idx 10 has no Priority override, so the public tag
            // is LAST-wins. `video_matrix` below independently captures the first
            // video-track matrix for CalcRotation (QuickTime.pm:8797).
            self.add_scalar("MatrixStructure", TagValue::string(matrix), Priority::Last);
        }
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

    /// hdlr → QuickTime::Handler (QuickTime.pm:8391), a ProcessBinaryData table with
    /// the default int8u FORMAT (so tag indices are byte offsets). Yields
    /// HandlerType (idx 8) and HandlerDescription (idx 24, "string", Pascal/C RawConv
    /// at Handler:8457). `set_media_type` is true only for the `trak/mdia/hdlr` media
    /// handler (QuickTime.pm:8413) — the `minf`/`meta` data/metadata handlers must
    /// not touch `$$self{MediaType}`.
    fn decode_hdlr(&mut self, p: &[u8], set_media_type: bool) {
        if p.len() >= 12 {
            let mut ht = [0u8; 4];
            ht.copy_from_slice(&p[8..12]);
            // Handler:8412 updates HandlerType except for data-reference handlers.
            if &ht != b"alis" && &ht != b"url " {
                self.handler_type = Some(ht);
                // Pair the pending tkhd matrix with the first video track (Rotation).
                if set_media_type && &ht == b"vide" && self.video_matrix.is_none() {
                    self.video_matrix = self.pending_matrix.clone();
                }
            }
            // MediaType has a separate PATH condition and is set even for
            // alis/url (Handler:8413). It persists until the enclosing trak ends.
            if set_media_type {
                self.media_type = Some(ht);
            }
            let raw = TagValue::string(String::from_utf8_lossy(&ht));
            if let Some(info) = QUICK_TIME_HANDLER_TAGS.get(&8).cloned() {
                // Handler conversion failures are impossible for this generated
                // simple lookup; preserve extraction's best-effort contract.
                let _ = self.add_from_tag_info(&info, raw, Priority::Last);
            }
        }
        // HandlerDescription (idx 24): "string" runs to the atom end; the Pascal/C
        // strip drops a leading length byte (< 0x20 and < len) and `length $val ?
        // $val : undef` suppresses empty values (e.g. the 'mdta' meta handler).
        if let Some(desc) = p.get(24..).and_then(strip_length_prefixed_string) {
            let name = table_name(&QUICK_TIME_HANDLER_TAGS, 24, "HandlerDescription");
            self.add_scalar(name, TagValue::string(desc), Priority::Last);
        }
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

    /// Apply the generated tag table's direct conversions at the parse site.
    /// Expression placeholders stay raw (GPS conversion is Task 4); registered
    /// Functions such as `%iso8601Date` run without duplicating table knowledge.
    fn add_from_tag_info(
        &mut self,
        info: &TagInfo,
        raw: TagValue,
        priority: Priority,
    ) -> Result<()> {
        let value = match &info.value_conv {
            Some(ValueConv::Function(function)) => function(&raw, None)?,
            Some(ValueConv::Numeric(factor)) => &raw * *factor,
            _ => raw,
        };
        let print = match &info.print_conv {
            Some(PrintConv::Simple(lookup)) => lookup
                .get(&value.to_string())
                .map(|text| TagValue::string(*text))
                .unwrap_or_else(|| value.clone()),
            Some(PrintConv::Function(function)) => function(&value, None),
            _ => value.clone(),
        };
        self.insert(info.name, value, print, priority);
        Ok(())
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

fn table_name<'a>(
    table: &'a std::collections::HashMap<u16, TagInfo>,
    index: u16,
    fallback: &'a str,
) -> &'a str {
    table.get(&index).map(|info| info.name).unwrap_or(fallback)
}

fn lookup_key_tables(
    primary: &'static std::collections::HashMap<&'static [u8], TagInfo>,
    key: &[u8],
) -> Option<&'static TagInfo> {
    primary
        .get(key)
        .or_else(|| QUICK_TIME_ITEMLIST_TAGS_BY_NAME.get(key))
        .or_else(|| QUICK_TIME_USERDATA_TAGS_BY_NAME.get(key))
}

/// ProcessKeys:9806-9810 tolerates samples containing a reversed ItemList or
/// UserData ID when the bytes match `/^\w{3}\xa9$/`.
fn reversed_copyright_fourcc(key: &[u8]) -> Option<[u8; 4]> {
    if key.len() == 4
        && key[..3]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        && key[3] == 0xa9
    {
        Some([key[3], key[2], key[1], key[0]])
    } else {
        None
    }
}

fn is_task3_item(name: &str) -> bool {
    matches!(
        name,
        "Make" | "Model" | "Software" | "CreationDate" | "GPSCoordinates" | "LensModel"
    )
}

/// Handler:8454-8460 / VisualSampleDesc:7642-7647: strings are sometimes
/// Pascal and sometimes C. A leading control byte smaller than the buffer is a
/// length; otherwise the first NUL terminates the C string.
fn strip_length_prefixed_string(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    let value = if bytes[0] < 0x20 && usize::from(bytes[0]) < bytes.len() {
        &bytes[1..1 + usize::from(bytes[0])]
    } else {
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        &bytes[..end]
    };
    (!value.is_empty()).then(|| String::from_utf8_lossy(value).into_owned())
}

fn nul_terminated_string(bytes: &[u8]) -> Option<String> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    (end != 0).then(|| String::from_utf8_lossy(&bytes[..end]).into_owned())
}

/// TrackHeader:1561-1566 MatrixStructure ValueConv. ProcessBinaryData first
/// decodes each fixed32s as 16.16, then ExifTool divides right-column elements
/// 2/5/8 by another 0x4000 because those source words are fixed 2.30.
fn matrix_structure(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 36 {
        return None;
    }
    let mut values = Vec::with_capacity(9);
    for index in 0..9 {
        let off = index * 4;
        let raw = i32::from_be_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]]);
        let mut value = f64::from(raw) / 65_536.0;
        if matches!(index, 2 | 5 | 8) {
            value /= 16_384.0;
        }
        values.push(value.to_string());
    }
    Some(values.join(" "))
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

    fn keys(names: &[&[u8]]) -> Vec<u8> {
        let mut payload = vec![0u8; 4]; // version/flags
        payload.extend_from_slice(&(names.len() as u32).to_be_bytes());
        for name in names {
            payload.extend_from_slice(&((name.len() + 8) as u32).to_be_bytes());
            payload.extend_from_slice(b"mdta");
            payload.extend_from_slice(name);
        }
        atom(b"keys", &payload)
    }

    fn data(value: &[u8]) -> Vec<u8> {
        let mut payload = 1u32.to_be_bytes().to_vec(); // flags 0x1 = UTF-8
        payload.extend_from_slice(&[0u8; 4]); // country + language
        payload.extend_from_slice(value);
        atom(b"data", &payload)
    }

    fn ilst_item(index: u32, values: &[&[u8]]) -> Vec<u8> {
        let mut payload = Vec::new();
        for value in values {
            payload.extend_from_slice(&data(value));
        }
        atom(&index.to_be_bytes(), &payload)
    }

    fn hdlr(handler_type: &[u8; 4], description: &[u8]) -> Vec<u8> {
        let mut payload = vec![0u8; 24];
        payload[8..12].copy_from_slice(handler_type);
        payload.extend_from_slice(description);
        atom(b"hdlr", &payload)
    }

    fn stsd(compressor_name: &[u8]) -> Vec<u8> {
        let mut entry = vec![0u8; 82];
        let entry_len = entry.len() as u32;
        entry[0..4].copy_from_slice(&entry_len.to_be_bytes());
        entry[4..8].copy_from_slice(b"avc1");
        let copy_len = compressor_name.len().min(32);
        entry[50..50 + copy_len].copy_from_slice(&compressor_name[..copy_len]);

        let mut payload = vec![0u8; 4]; // version/flags
        payload.extend_from_slice(&1u32.to_be_bytes());
        payload.extend_from_slice(&entry);
        atom(b"stsd", &payload)
    }

    fn find<'a>(tags: &'a [TagEntry], name: &str) -> Option<&'a TagEntry> {
        tags.iter().find(|tag| tag.name == name)
    }

    #[test]
    fn keys_are_one_based_and_multiple_data_values_are_processed() {
        let meta = atom(
            b"meta",
            &[
                keys(&[b"com.apple.quicktime.make", b"com.apple.quicktime.model"]),
                atom(
                    b"ilst",
                    &[ilst_item(1, &[b"Apple"]), ilst_item(2, &[b"17.0", b"18.5"])].concat(),
                ),
            ]
            .concat(),
        );
        let mut r = Cursor::new(atom(b"moov", &meta));
        let tags = extract_quicktime_metadata(&mut r).unwrap();

        assert_eq!(
            find(&tags, "Make").unwrap().value,
            TagValue::string("Apple")
        );
        assert_eq!(
            find(&tags, "Model").unwrap().value,
            TagValue::string("18.5")
        );
    }

    #[test]
    fn keys_state_is_scoped_to_one_meta_directory() {
        let first = atom(
            b"meta",
            &[
                keys(&[b"com.apple.quicktime.make"]),
                atom(b"ilst", &ilst_item(1, &[b"Apple"])),
            ]
            .concat(),
        );
        // No keys atom here: index 1 must not reuse the preceding meta's Make.
        let second = atom(b"meta", &atom(b"ilst", &ilst_item(1, &[b"Bogus"])));
        let mut r = Cursor::new(atom(b"moov", &[first, second].concat()));
        let tags = extract_quicktime_metadata(&mut r).unwrap();

        assert_eq!(
            find(&tags, "Make").unwrap().value,
            TagValue::string("Apple")
        );
    }

    #[test]
    fn top_level_and_userdata_meta_skip_fullbox_header() {
        let children = [
            keys(&[b"com.apple.quicktime.make"]),
            atom(b"ilst", &ilst_item(1, &[b"Apple"])),
        ]
        .concat();
        let mut fullbox = vec![0u8; 4];
        fullbox.extend_from_slice(&children);

        let mut top = Cursor::new(atom(b"meta", &fullbox));
        let top_tags = extract_quicktime_metadata(&mut top).unwrap();
        assert_eq!(
            find(&top_tags, "Make").unwrap().value,
            TagValue::string("Apple")
        );

        let udta_meta = atom(b"udta", &atom(b"meta", &fullbox));
        let mut nested = Cursor::new(atom(b"moov", &udta_meta));
        let nested_tags = extract_quicktime_metadata(&mut nested).unwrap();
        assert_eq!(
            find(&nested_tags, "Make").unwrap().value,
            TagValue::string("Apple")
        );
    }

    #[test]
    fn length_prefixed_string_handles_pascal_c_and_empty_forms() {
        assert_eq!(
            strip_length_prefixed_string(b"\x05H.264ignored"),
            Some("H.264".to_string())
        );
        assert_eq!(
            strip_length_prefixed_string(b"AVC Coding\0ignored"),
            Some("AVC Coding".to_string())
        );
        assert_eq!(strip_length_prefixed_string(b"\0ignored"), None);
    }

    #[test]
    fn stsd_compressor_name_is_gated_by_handler_type() {
        fn file(handler: &[u8; 4]) -> Vec<u8> {
            let stbl = atom(b"stbl", &stsd(b"\x05H.264"));
            let minf = atom(b"minf", &stbl);
            let mdia = atom(b"mdia", &[hdlr(handler, b""), minf].concat());
            atom(b"moov", &atom(b"trak", &mdia))
        }

        let mut video = Cursor::new(file(b"vide"));
        let video_tags = extract_quicktime_metadata(&mut video).unwrap();
        assert_eq!(
            find(&video_tags, "CompressorName").unwrap().value,
            TagValue::string("H.264")
        );

        let mut audio = Cursor::new(file(b"soun"));
        let audio_tags = extract_quicktime_metadata(&mut audio).unwrap();
        assert!(find(&audio_tags, "CompressorName").is_none());
    }

    #[test]
    fn media_type_selects_video_keys_after_handler_type_changes() {
        // Handler:8412/8413 keep separate state: the media hdlr sets MediaType=vide;
        // the later meta hdlr changes HandlerType=mdta but must not change MediaType.
        let media = atom(b"mdia", &hdlr(b"vide", b"Video Handler\0"));
        let track_meta = atom(
            b"meta",
            &[
                hdlr(b"mdta", b""),
                keys(&[b"camera.lens_model"]),
                atom(b"ilst", &ilst_item(1, &[b"Test Lens"])),
            ]
            .concat(),
        );
        let track = atom(b"trak", &[media, track_meta].concat());
        let mut r = Cursor::new(atom(b"moov", &track));
        let tags = extract_quicktime_metadata(&mut r).unwrap();

        assert_eq!(
            find(&tags, "LensModel").unwrap().value,
            TagValue::string("Test Lens")
        );
        assert_eq!(
            find(&tags, "HandlerType").unwrap().value,
            TagValue::string("mdta")
        );
    }

    #[test]
    fn userdata_canon_model_is_nul_terminated() {
        let udta = atom(b"udta", &atom(b"CNMN", b"Canon EOS 500D\0ignored"));
        let mut r = Cursor::new(atom(b"moov", &udta));
        let tags = extract_quicktime_metadata(&mut r).unwrap();

        assert_eq!(
            find(&tags, "Model").unwrap().value,
            TagValue::string("Canon EOS 500D")
        );
    }

    #[test]
    fn matrix_and_handler_type_are_observable_prerequisites() {
        let matrix: [i32; 9] = [0, 65_536, 0, -65_536, 0, 0, 1440 * 65_536, 0, 1 << 30];
        let mut tkhd = vec![0u8; 84];
        for (index, value) in matrix.into_iter().enumerate() {
            let off = 40 + index * 4;
            tkhd[off..off + 4].copy_from_slice(&value.to_be_bytes());
        }
        let mdia = atom(b"mdia", &hdlr(b"vide", b"Video Handler\0"));
        let trak = atom(b"trak", &[atom(b"tkhd", &tkhd), mdia].concat());
        let mut r = Cursor::new(atom(b"moov", &trak));
        let tags = extract_quicktime_metadata(&mut r).unwrap();

        assert_eq!(
            find(&tags, "MatrixStructure").unwrap().value,
            TagValue::string("0 1 0 -1 0 0 1440 0 1")
        );
        assert_eq!(
            find(&tags, "HandlerType").unwrap().value,
            TagValue::string("vide")
        );
    }

    #[test]
    fn public_matrix_is_last_wins_but_rotation_keeps_first_video_matrix() {
        fn track(matrix: [i32; 9], handler: &[u8; 4]) -> Vec<u8> {
            let mut tkhd = vec![0u8; 84];
            for (index, value) in matrix.into_iter().enumerate() {
                let off = 40 + index * 4;
                tkhd[off..off + 4].copy_from_slice(&value.to_be_bytes());
            }
            let mdia = atom(b"mdia", &hdlr(handler, b""));
            atom(b"trak", &[atom(b"tkhd", &tkhd), mdia].concat())
        }

        let rotated = [0, 65_536, 0, -65_536, 0, 0, 1440 * 65_536, 0, 1 << 30];
        let identity = [65_536, 0, 0, 0, 65_536, 0, 0, 0, 1 << 30];
        let moov = atom(
            b"moov",
            &[track(rotated, b"vide"), track(identity, b"soun")].concat(),
        );
        let mut reader = Cursor::new(moov);
        let file_end = reader.seek(SeekFrom::End(0)).unwrap();
        let mut walker = Walker {
            reader: &mut reader,
            tags: IndexMap::new(),
            time_scale: None,
            handler_type: None,
            media_type: None,
            current_keys: Vec::new(),
            current_item: None,
            video_matrix: None,
            pending_matrix: None,
        };
        walker.process(Container::TopLevel, 0, file_end, 0).unwrap();

        // CalcRotation:8797 needs the first vide track, independent of the
        // ordinary tag's default duplicate handling.
        assert_eq!(
            walker.video_matrix.as_deref(),
            Some("0 1 0 -1 0 0 1440 0 1")
        );
        // TrackHeader MatrixStructure has default priority, so the public tag is
        // overwritten by the later audio track (ExifTool FoundTag:9536-9588).
        assert_eq!(
            walker.tags.get("MatrixStructure").unwrap().value,
            TagValue::string("1 0 0 0 1 0 0 0 1")
        );
    }

    #[test]
    fn malformed_keys_ilst_and_data_boxes_do_not_panic() {
        let mut bad_keys = vec![0u8; 8];
        bad_keys.extend_from_slice(&7u32.to_be_bytes()); // entry shorter than header
        bad_keys.extend_from_slice(b"mdta");
        let malformed_ilst = atom(
            b"ilst",
            &[
                // Item claims to end beyond ilst.
                [0, 0, 1, 0, 0, 0, 0, 1].to_vec(),
                // Valid item shell with truncated data header.
                ilst_item(1, &[b""]),
            ]
            .concat(),
        );
        let meta = atom(
            b"meta",
            &[atom(b"keys", &bad_keys), malformed_ilst].concat(),
        );
        let mut r = Cursor::new(atom(b"moov", &meta));

        let tags = extract_quicktime_metadata(&mut r).unwrap();
        assert!(tags.is_empty());

        // A known key whose nested data atom claims to extend past its item.
        let mut truncated_data = 100u32.to_be_bytes().to_vec();
        truncated_data.extend_from_slice(b"data");
        truncated_data.extend_from_slice(&[0u8; 8]);
        let bad_item = atom(&1u32.to_be_bytes(), &truncated_data);
        let meta = atom(
            b"meta",
            &[
                keys(&[b"com.apple.quicktime.make"]),
                atom(b"ilst", &bad_item),
            ]
            .concat(),
        );
        let mut r = Cursor::new(atom(b"moov", &meta));
        let tags = extract_quicktime_metadata(&mut r).unwrap();
        assert!(tags.is_empty());
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
