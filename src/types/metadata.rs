//! Metadata structures for EXIF data representation
//!
//! This module defines the core metadata structures including TagEntry,
//! ExifData, and TagSourceInfo that represent extracted EXIF information.

use crate::hash::ImageHashType;
use crate::types::TagValue;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One `-TAG` request, in the order it appeared on the command line.
///
/// **The first request that matches a tag decides whether that tag prints its
/// ValueConv (`#`) or its PrintConv value.** Three ExifTool steps produce that rule:
///
/// 1. `SetFoundTags` walks `REQUESTED_TAGS` in order and appends each request's
///    matches to `@foundTags`, recording the indices contributed by a `#` request in
///    `@byValue` (lib/Image/ExifTool.pm:5433-5436). A tag matched by several requests
///    therefore appears several times, in request order.
/// 2. `GetInfo` renames each by-value entry's key to `"Tag #"` and deletes the plain
///    PrintConv entry only when no non-by-value request also produced it
///    (lib/Image/ExifTool.pm:3266-3290). Both keys survive, still in request order.
/// 3. The JSON writer walks `@foundTags` and does `next if $noDups{$tok}` on the tag
///    name (exiftool:2947-2953), so the first entry wins and later ones are skipped.
///
/// Plain `-G` text output prints *both* lines, in request order, which is why the
/// later line is the one that catches the eye there - it is not the winner.
///
/// Probed against vendored ExifTool 13.59 (test-images/canon/eos_rebel_t3i.jpg):
/// `-Orientation -Orientation#` => "Rotate 270 CW"; `-Orientation# -Orientation` => 8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagRequest {
    /// The request with its leading `-` and trailing `#` removed.
    /// Examples: "Duration", "*Duration*", "QuickTime:Duration", "EXIF:all", "all"
    pub pattern: String,

    /// The request ended in `#`: print the ValueConv value for tags it matches.
    /// ExifTool: lib/Image/ExifTool.pm:5364 (`$byValue = 1 if $tag =~ s/#$//`)
    pub numeric: bool,
}

impl TagRequest {
    /// Build a request from an already-split pattern and `#` flag.
    pub fn new(pattern: impl Into<String>, numeric: bool) -> Self {
        Self {
            pattern: pattern.into(),
            numeric,
        }
    }

    /// Parse one request argument that has already had its leading `-` removed.
    ///
    /// ExifTool strips a trailing `#` from the tag portion and remembers that the tag
    /// was requested by value; everything else about the request - group prefix,
    /// wildcards, `all` - is then handled exactly as it would be without the `#`.
    /// ExifTool: lib/Image/ExifTool.pm:5364
    ///
    /// `--all` is accepted as a spelling of `-all`. Every other `--TAG` is ExifTool's
    /// *exclusion* syntax, which exif-oxide does not implement; the second `-` is kept
    /// in the pattern so such a request matches nothing rather than silently turning
    /// into the opposite request. `matches_glob_pattern` compares literally, and no
    /// tag name starts with `-`.
    pub fn parse(filter_arg: &str) -> Self {
        if filter_arg.eq_ignore_ascii_case("-all") {
            return Self::new("all", false);
        }
        match filter_arg.strip_suffix('#') {
            Some(pattern) if !pattern.is_empty() => Self::new(pattern, true),
            _ => Self::new(filter_arg, false),
        }
    }
}

/// Configuration for filtering which tags to extract and how to format them
///
/// This struct controls both tag selection (filtering) and value formatting (PrintConv vs ValueConv).
/// It enables performance optimization by extracting only requested tags and early termination
/// when simple tags (like File group tags) are requested.
///
/// # Examples
///
/// ```
/// use exif_oxide::types::{FilterOptions, TagRequest};
///
/// // Extract only MIMEType tag (performance optimized - no EXIF parsing needed)
/// let mime_only = FilterOptions::tags_only(vec!["MIMEType".to_string()]);
///
/// // The same requests the CLI would see for `-EXIF:all -Orientation#`
/// let exif_with_numeric = FilterOptions::from_requests(vec![
///     TagRequest::new("EXIF:all", false),
///     TagRequest::new("Orientation", true),
/// ]);
///
/// // Compute ImageDataHash with SHA256
/// use exif_oxide::hash::ImageHashType;
/// let hash_only = FilterOptions::image_hash_only(ImageHashType::Sha256);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FilterOptions {
    /// Specific tags to extract (case-insensitive)
    /// Examples: ["MIMEType", "Orientation", "FNumber"]
    pub requested_tags: Vec<String>,

    /// Group filters without :all suffix (case-insensitive)  
    /// Examples: ["EXIF", "File", "GPS"]
    pub requested_groups: Vec<String>,

    /// Group:all patterns (case-insensitive)
    /// Examples: ["File:all", "EXIF:all", "GPS:all"]
    pub group_all_patterns: Vec<String>,

    /// Extract all available tags (default behavior for backward compatibility)
    /// When true, all other filters are ignored
    pub extract_all: bool,

    /// Every `-TAG` request in command-line order, each carrying its `#` flag.
    ///
    /// This is ExifTool's `REQUESTED_TAGS`: the other fields on this struct are the
    /// extraction index built from it, but order is only recoverable here, and order
    /// is what decides ValueConv-vs-PrintConv when several requests match one tag.
    /// ExifTool: lib/Image/ExifTool.pm:5329, 5345 (SetFoundTags)
    pub tag_requests: Vec<TagRequest>,

    /// Glob patterns for tag/group matching (case-insensitive)
    /// Examples: ["GPS*", "*tude", "*Date*", "Canon*"]
    /// Supports prefix (*), suffix (*), and middle (*) wildcards
    pub glob_patterns: Vec<String>,

    /// Compute ImageDataHash during file processing
    ///
    /// When enabled, computes a cryptographic hash of the actual image data
    /// (excluding metadata) and outputs it as `Composite:ImageDataHash`.
    ///
    /// ExifTool equivalent: `-api requesttags=imagedatahash`
    ///
    /// This option triggers hash computation during file parsing - the hash
    /// accumulates as image data is encountered (SOS for JPEG, IDAT for PNG, etc.).
    pub compute_image_hash: bool,

    /// Hash algorithm for ImageDataHash computation
    ///
    /// ExifTool equivalent: `-api imagehashtype=MD5|SHA256|SHA512`
    ///
    /// Default: MD5 (matches ExifTool default)
    pub image_hash_type: ImageHashType,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: true, // Default to extracting all tags for backward compatibility
            tag_requests: Vec::new(),
            glob_patterns: Vec::new(),
            compute_image_hash: false, // Only compute when explicitly requested
            image_hash_type: ImageHashType::default(), // MD5, matching ExifTool default
        }
    }
}

impl FilterOptions {
    /// Create FilterOptions that extracts all tags (backward compatibility)
    pub fn extract_all() -> Self {
        Self::default()
    }

    /// Create FilterOptions for specific tags only
    pub fn tags_only(tags: Vec<String>) -> Self {
        Self {
            tag_requests: tags.iter().map(|tag| TagRequest::new(tag, false)).collect(),
            requested_tags: tags,
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            glob_patterns: Vec::new(),
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        }
    }

    /// Build FilterOptions from the ordered request list, the way the CLI does.
    ///
    /// Every request is classified *after* its `#` has been stripped, so `-EXIF:all#`
    /// selects the EXIF group exactly as `-EXIF:all` does and differs only in how the
    /// matched tags print. An empty request list means "no filters", which extracts
    /// everything - the same fallback ExifTool uses when no tags are named
    /// (lib/Image/ExifTool.pm:5340).
    ///
    /// The three pattern buckets are all matched through `request_matches_tag`, so the
    /// split between them no longer changes what is selected; it is kept because
    /// `is_file_group_only` and the compat JSON filter still read them individually.
    pub fn from_requests(tag_requests: Vec<TagRequest>) -> Self {
        let mut requested_tags = Vec::new();
        let mut group_all_patterns = Vec::new();
        let mut glob_patterns = Vec::new();
        // A bare `-all` needs no filtering at all, which lets extraction take its fast
        // path. `-all#` cannot: that path skips the numeric override in
        // `formats::extract_metadata`, so the request has to stay a real filter.
        // ExifTool: lib/Image/ExifTool.pm:5367 - `-all` matches every tag.
        let extract_all = tag_requests.is_empty()
            || tag_requests
                .iter()
                .any(|request| !request.numeric && Self::is_all_keyword(&request.pattern));

        for request in &tag_requests {
            let pattern = request.pattern.as_str();
            if Self::is_all_keyword(pattern) {
                // `-all#`: keep it as a pattern so the numeric override still runs.
                if request.numeric {
                    requested_tags.push(pattern.to_string());
                }
            } else if pattern
                .rsplit_once(':')
                .is_some_and(|(_, tag)| tag.eq_ignore_ascii_case("all"))
            {
                // `Group:all` only. `Group:*` selects the same tags but travels the
                // glob path, because `is_file_group_only` reads `group_all_patterns`
                // expecting the literal "file:all".
                group_all_patterns.push(pattern.to_string());
            } else if Self::has_wildcard(pattern) {
                glob_patterns.push(pattern.to_string());
            } else {
                requested_tags.push(pattern.to_string());
            }
        }

        // A bare `-all` subsumes every other selector.
        if extract_all {
            requested_tags.clear();
            group_all_patterns.clear();
            glob_patterns.clear();
        }

        Self {
            requested_tags,
            requested_groups: Vec::new(),
            group_all_patterns,
            extract_all,
            tag_requests,
            glob_patterns,
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        }
    }

    /// Is this request's tag portion ExifTool's "everything" keyword?
    /// ExifTool: lib/Image/ExifTool.pm:5350, 5367 (`/^(\*|all)$/i`)
    fn is_all_keyword(pattern: &str) -> bool {
        pattern == "*" || pattern.eq_ignore_ascii_case("all")
    }

    /// Create FilterOptions for specific groups
    pub fn groups_only(groups: Vec<String>) -> Self {
        Self {
            requested_tags: Vec::new(),
            requested_groups: groups,
            group_all_patterns: Vec::new(),
            extract_all: false,
            tag_requests: Vec::new(),
            glob_patterns: Vec::new(),
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        }
    }

    /// Create FilterOptions that only computes ImageDataHash
    pub fn image_hash_only(hash_type: ImageHashType) -> Self {
        Self {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            tag_requests: Vec::new(),
            glob_patterns: Vec::new(),
            compute_image_hash: true,
            image_hash_type: hash_type,
        }
    }

    /// Check if we should extract all tags (ignoring filters)
    pub fn should_extract_all(&self) -> bool {
        self.extract_all
    }

    /// Check if any specific tags or groups are requested
    pub fn has_specific_requests(&self) -> bool {
        !self.requested_tags.is_empty()
            || !self.requested_groups.is_empty()
            || !self.group_all_patterns.is_empty()
            || !self.glob_patterns.is_empty()
    }

    /// Check if a tag should be extracted based on current filters
    /// Uses case-insensitive matching to match ExifTool behavior
    ///
    /// This form only knows the tag's family-0 group. Use
    /// [`FilterOptions::should_extract_tag_in_groups`] wherever the tag's Group1 is
    /// known, so that family-1 requests like `-ExifIFD:FNumber` can match.
    pub fn should_extract_tag(&self, tag_name: &str, tag_group: &str) -> bool {
        self.should_extract_tag_in_groups(tag_name, &[tag_group])
    }

    /// Check if a tag should be extracted, given every group family it belongs to.
    ///
    /// `tag_groups` is indexed by ExifTool group family: `[group0, group1]`. ExifTool
    /// matches a bare group name against *all* families (see
    /// [`FilterOptions::group_name_matches`]), so both entries matter.
    ///
    /// Every kind of request - a plain tag name, a group-qualified name, a wildcard,
    /// `all` - runs through the same matcher, mirroring ExifTool's single request loop.
    /// ExifTool: lib/Image/ExifTool.pm:5345-5401 (SetFoundTags)
    pub fn should_extract_tag_in_groups(&self, tag_name: &str, tag_groups: &[&str]) -> bool {
        if self.extract_all {
            return true;
        }

        // `requested_groups` is an exif-oxide convenience (FilterOptions::groups_only),
        // not an ExifTool request string: a bare group name here selects the group
        // rather than a tag of that name.
        if self.requested_groups.iter().any(|requested| {
            tag_groups
                .iter()
                .any(|group| group.eq_ignore_ascii_case(requested))
        }) {
            return true;
        }

        self.requested_tags
            .iter()
            .chain(self.group_all_patterns.iter())
            .chain(self.glob_patterns.iter())
            .any(|pattern| Self::request_matches_tag(pattern, tag_name, tag_groups))
    }

    /// Match one ExifTool tag request against a tag, honouring an optional group prefix.
    ///
    /// This reproduces the body of ExifTool's request loop
    /// (lib/Image/ExifTool.pm:5345-5401) in order:
    ///
    /// 1. Split `Group:Tag` at the *last* colon (:5348, `/^(.*):(.+)/`), so the group
    ///    portion may itself name several groups.
    /// 2. Require the group portion to match (:5398-5401 via `GroupMatches`).
    /// 3. A tag portion of `*` or `all` matches every tag (:5367).
    /// 4. Otherwise delete illegal characters from the tag portion (:5378, :5386) and
    ///    match it as a wildcard pattern or as an exact case-insensitive name.
    ///
    /// `pattern` must already have had its single trailing `#` removed - ExifTool strips
    /// exactly one (:5364) and the CLI argument parsers do that when they decide whether
    /// a request is a numeric one. Stripping a second `#` here would make `-all##`
    /// select every tag, where ExifTool sterilizes it down to a request for a tag
    /// literally named "all" and returns nothing.
    ///
    /// Because the wildcards expand to `[-\w]`, they never match the `:` that separates
    /// a group from a tag name - the group is split off before matching, so `-EXIF*`
    /// returns only tags whose *name* starts with "Exif", not the whole EXIF group.
    fn request_matches_tag(pattern: &str, tag_name: &str, tag_groups: &[&str]) -> bool {
        // ExifTool: lib/Image/ExifTool.pm:5348 - `/^(.*):(.+)/` is greedy, so the split
        // happens at the last colon and the tag portion must be non-empty.
        let (group_spec, tag_spec) = match pattern.rsplit_once(':') {
            Some((group, tag)) if !tag.is_empty() => (Some(group), tag),
            _ => (None, pattern),
        };

        if let Some(group_spec) = group_spec {
            if !Self::group_spec_matches(group_spec, tag_groups) {
                return false;
            }
        }

        // ExifTool: lib/Image/ExifTool.pm:5367-5368 - "tag name of '*' or 'all' matches
        // all tags". This is checked before sterilization, so `-al.l` is *not* a request
        // for every tag.
        if tag_spec == "*" || tag_spec.eq_ignore_ascii_case("all") {
            return true;
        }

        let tag_spec = Self::sterilize_tag_spec(tag_spec);
        if Self::has_wildcard(&tag_spec) {
            Self::matches_glob_pattern(tag_name, &tag_spec)
        } else {
            tag_spec.eq_ignore_ascii_case(tag_name)
        }
    }

    /// Delete the characters ExifTool refuses to match on from a tag request.
    ///
    /// ExifTool "sterilizes" the tag portion of a request before matching:
    /// `$tag =~ tr/-_A-Za-z0-9*?//dc;` for wildcard requests
    /// (lib/Image/ExifTool.pm:5378) and `tr/-_A-Za-z0-9//dc` for plain ones (:5386).
    /// A single pass that keeps `*` and `?` covers both, because the second form only
    /// runs when the request has no wildcards left to keep.
    ///
    /// The :5386 branch is reached whenever the Duplicates option is on, which the
    /// `-j` JSON output turns on unconditionally (exiftool:949) - so the "Invalid tag
    /// name" dead end at :5396 is unreachable for JSON output, which is all exif-oxide
    /// produces.
    fn sterilize_tag_spec(tag_spec: &str) -> String {
        tag_spec
            .chars()
            .filter(|c| Self::is_tag_name_char(*c) || *c == '*' || *c == '?')
            .collect()
    }

    /// Does the group portion of a request match this tag's groups?
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5350-5360 validates the group name, then
    /// :5398-5401 hands it to `GroupMatches` (:5218-5259).
    fn group_spec_matches(group_spec: &str, tag_groups: &[&str]) -> bool {
        // ExifTool: :5350 - a group of '*' or 'all' means "any group".
        if group_spec == "*" || group_spec.eq_ignore_ascii_case("all") {
            return true;
        }
        // ExifTool: :5357-5359 - a group name outside `[-\w:]` is invalid; ExifTool
        // warns and substitutes 'invalid', which matches nothing. Note this happens
        // *instead of* sterilization: illegal characters are not silently dropped here.
        if !group_spec
            .chars()
            .all(|c| Self::is_tag_name_char(c) || c == ':')
        {
            return false;
        }
        // An empty group portion constrains nothing: `GroupMatches("")` splits into an
        // empty list and every tag falls through as a match. `-:*Duration*` therefore
        // selects the same tags as `-*Duration*`.
        if group_spec.is_empty() {
            return true;
        }
        // ExifTool: :5224 - the group portion may name several groups separated by ':'
        // (eg. "EXIF:ExifIFD"), and every one of them must match.
        group_spec
            .split(':')
            .all(|group| Self::group_name_matches(group, tag_groups))
    }

    /// Does one group name from a request match this tag's groups?
    ///
    /// A bare name matches *any* family, which is how `-ExifIFD:FNumber` (family 1) and
    /// `-EXIF:FNumber` (family 0) both work. A leading number pins the name to that
    /// family, so `-1ExifIFD:FNumber` matches but `-1EXIF:FNumber` does not.
    ///
    /// `GroupMatches` peels the family number off *before* it looks for `*`/`all`, so a
    /// family number in front of `all` is ignored entirely: `-1all:FNumber` and
    /// `-0all:FNumber` both select every group, exactly like `-all:FNumber`.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5227-5228 (family prefix, stripped first),
    /// :5241 (`*`/`all` skipped), :5243-5250 (family-pinned compare), :5252 (bare name
    /// compared against every family of `GetGroup($tag, -1)`).
    ///
    /// exif-oxide models families 0 and 1 only, so a request naming a family-2 group
    /// (`-Image:FNumber`, which ExifTool does match) finds nothing here. ExifTool's
    /// `id-` prefix (family 7, match by tag ID) is likewise not modelled; it falls
    /// through to a plain name comparison and matches nothing.
    fn group_name_matches(group: &str, tag_groups: &[&str]) -> bool {
        if group.is_empty() {
            return true;
        }

        // ExifTool: :5227 - `s/^(\d*)(id-)?//i` peels an optional family number off the
        // front of the group name, before :5241 checks the remainder for `*`/`all`.
        let digits = group.len() - group.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        let (family, name) = group.split_at(digits);

        // ExifTool: :5241 - `next if $grp eq '*' or $grp eq 'all'` skips the whole check,
        // family number and all.
        if name == "*" || name.eq_ignore_ascii_case("all") {
            return true;
        }

        if digits > 0 {
            // ExifTool: :5244 - `last unless defined $groups[$f]`, so a family we do not
            // model cannot match.
            return family
                .parse::<usize>()
                .ok()
                .and_then(|family| tag_groups.get(family))
                .is_some_and(|tag_group| tag_group.eq_ignore_ascii_case(name));
        }

        tag_groups
            .iter()
            .any(|tag_group| tag_group.eq_ignore_ascii_case(name))
    }

    /// Does this tag request contain a wildcard?
    ///
    /// ExifTool treats a requested tag name as a pattern as soon as it contains
    /// `*` or `?`.
    /// ExifTool: lib/Image/ExifTool.pm:5376 (`elsif ($tag =~ /[*?]/)`)
    pub fn has_wildcard(pattern: &str) -> bool {
        pattern.contains('*') || pattern.contains('?')
    }

    /// Check if a tag should use numeric output (ValueConv instead of PrintConv)
    ///
    /// This form only knows the tag's family-0 group; use
    /// [`FilterOptions::should_use_numeric_in_groups`] wherever Group1 is known.
    pub fn should_use_numeric(&self, tag_name: &str, tag_group: &str) -> bool {
        self.should_use_numeric_in_groups(tag_name, &[tag_group])
    }

    /// Check if a tag should use numeric output, given every group family it belongs to.
    pub fn should_use_numeric_in_groups(&self, tag_name: &str, tag_groups: &[&str]) -> bool {
        Self::numeric_request_matches(&self.tag_requests, tag_name, tag_groups)
    }

    /// Decide whether a tag prints its ValueConv value, from the ordered request list.
    ///
    /// The **first** request that matches the tag wins. ExifTool appends each
    /// request's matches to the found-tag list in request order and records which
    /// entries came from a `#` request; the JSON writer prints the first entry for a
    /// tag name and skips every later one, so a tag matched by several requests is
    /// printed the way its earliest matching request asked for.
    ///
    /// Wildcards work here exactly as they do for plain tag requests, because
    /// ExifTool strips the `#` before expanding them: `-*Duration*#` prints every tag
    /// whose name contains "Duration" as its ValueConv value. A group-qualified
    /// request such as `-QuickTime:*Duration*#` only matches tags in that group, so a
    /// request naming the wrong group is skipped and the next request decides.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5345-5437 (SetFoundTags), 3266-3290 (GetInfo),
    /// exiftool:2947-2953 (JSON `%noDups`).
    pub fn numeric_request_matches(
        tag_requests: &[TagRequest],
        tag_name: &str,
        tag_groups: &[&str],
    ) -> bool {
        tag_requests
            .iter()
            .find(|request| Self::request_matches_tag(&request.pattern, tag_name, tag_groups))
            .is_some_and(|request| request.numeric)
    }

    /// Check if a string matches an ExifTool tag-name pattern (case-insensitive).
    ///
    /// ExifTool converts a requested tag name containing `*` or `?` into a regular
    /// expression by expanding `*` to `[-\w]*` and `?` to `[-\w]`, then matches it
    /// anchored and case-insensitively against every extracted tag name:
    ///
    /// ```text
    /// $tag =~ s/\*/[-\w]*/g;
    /// $tag =~ s/\?/[-\w]/g;
    /// @matches = grep(/^$tag$/i, keys %$tagHash);
    /// ```
    ///
    /// Because the wildcards expand to `[-\w]`, they never match the `:` that
    /// separates a group from a tag name - ExifTool splits the group off the request
    /// before matching. We keep that property so patterns like `QuickTime:*Duration*`
    /// only match within the named group.
    ///
    /// Examples: "GPS*" matches "GPSLatitude", "*tude" matches "Latitude",
    /// "*Date*" matches "CreateDate", "Dur?tion" matches "Duration".
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5376-5382 (SetFoundTags)
    fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
        // ExifTool: lib/Image/ExifTool.pm:5367-5368 - a tag name of '*' matches all tags
        if pattern == "*" {
            return true;
        }

        let text: Vec<char> = text.to_lowercase().chars().collect();
        let pattern: Vec<char> = pattern.to_lowercase().chars().collect();

        // Greedy wildcard match with backtracking. `star_p`/`star_t` remember the
        // most recent `*` so it can consume one more character when the rest of the
        // pattern fails to line up.
        let (mut t, mut p) = (0usize, 0usize);
        let mut star: Option<(usize, usize)> = None;

        while t < text.len() {
            if p < pattern.len() && pattern[p] == '*' {
                star = Some((p, t));
                p += 1;
            } else if p < pattern.len()
                && ((pattern[p] == '?' && Self::is_tag_name_char(text[t]))
                    || (pattern[p] != '?' && pattern[p] == text[t]))
            {
                p += 1;
                t += 1;
            } else if let Some((star_p, star_t)) = star {
                // Let the `*` swallow one more character, but only characters that
                // ExifTool's `[-\w]*` expansion can match.
                if !Self::is_tag_name_char(text[star_t]) {
                    return false;
                }
                star = Some((star_p, star_t + 1));
                t = star_t + 1;
                p = star_p + 1;
            } else {
                return false;
            }
        }

        // Trailing `*`s can match the empty string
        while p < pattern.len() && pattern[p] == '*' {
            p += 1;
        }
        p == pattern.len()
    }

    /// Characters ExifTool's wildcards can match.
    ///
    /// ExifTool expands `*` to `[-\w]*` and `?` to `[-\w]`, and tag names are
    /// restricted to `[-A-Za-z0-9_]`.
    /// ExifTool: lib/Image/ExifTool.pm:5379-5380
    fn is_tag_name_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }

    /// Determine if only File group tags are requested (performance optimization)
    ///
    /// When this holds, `formats::extract_metadata` answers from `stat()` plus a short
    /// magic-number read instead of parsing EXIF/MakerNotes.
    ///
    /// The shortcut is only sound when no request can select a tag outside the File
    /// group, so it keys on the *group* portion of each request. A wildcard tag NAME
    /// never does: ExifTool matches it against every extracted tag name regardless of
    /// group, which is why `-File*#` also returns `EXIF:FileSource`
    /// (lib/Image/ExifTool.pm:5376-5382).
    pub fn is_file_group_only(&self) -> bool {
        if self.extract_all || !self.has_specific_requests() {
            return false;
        }

        // The same three request lists `should_extract_tag_in_groups` walks, held to
        // the stricter standard the shortcut needs.
        self.requested_groups
            .iter()
            .all(|group| group.eq_ignore_ascii_case("file"))
            && self
                .requested_tags
                .iter()
                .chain(self.group_all_patterns.iter())
                .chain(self.glob_patterns.iter())
                .all(|request| Self::is_file_group_request(request))
    }

    /// Can this request only ever select File group tags?
    ///
    /// Requests are split the way [`FilterOptions::request_matches_tag`] splits them -
    /// at the last colon, tag portion non-empty (ExifTool.pm:5348). Only the group
    /// portion can restrict a request to one group, so:
    ///
    /// - `File:anything` qualifies, whatever the tag portion is.
    /// - An unqualified request matches by name in *every* group, so it qualifies only
    ///   when the name itself is one no other group uses. A wildcard name never is:
    ///   `-File*#` returns `EXIF:FileSource` too.
    ///
    /// Anything else is conservatively refused, which costs a full parse but never
    /// returns a short answer: multi-group specs (`-File:System:FileName`), `*`/`all`
    /// groups, and family-1 names like `-System:all` all take the slow path.
    fn is_file_group_request(request: &str) -> bool {
        match request.rsplit_once(':') {
            Some((group_spec, tag_spec)) if !tag_spec.is_empty() => {
                Self::is_file_group_spec(group_spec)
            }
            _ => Self::is_file_only_tag_name(request),
        }
    }

    /// Does this group spec pin a request to the File group?
    ///
    /// `File` matches the File group in either family (ExifTool.pm:5252 compares a bare
    /// group name against every family), and `0File` pins it to family 0
    /// (ExifTool.pm:5243-5250). Only File group tags carry either.
    fn is_file_group_spec(group_spec: &str) -> bool {
        group_spec.eq_ignore_ascii_case("file") || group_spec.eq_ignore_ascii_case("0file")
    }

    /// Tag names only ever produced by `formats::extract_file_tags_only`.
    ///
    /// An exact request for one of these cannot select a tag from another group, so
    /// the shortcut answers it in full. These are ExifTool's System/File pseudo-tags:
    /// lib/Image/ExifTool.pm:1317-1517 and 9583.
    fn is_file_only_tag_name(tag: &str) -> bool {
        matches!(
            tag.to_lowercase().as_str(),
            "filename"
                | "directory"
                | "filesize"
                | "filemodifydate"
                | "fileaccessdate"
                | "fileinodechangedate"
                | "filecreatedate"
                | "filepermissions"
                | "filetype"
                | "filetypeextension"
                | "mimetype"
        )
    }
}

/// A single extracted metadata tag with both its converted value and display string.
///
/// This structure provides access to both the logical value (after ValueConv)
/// and the human-readable display string (after PrintConv), allowing consumers
/// to choose the most appropriate representation.
///
/// # Examples
///
/// ```
/// use exif_oxide::types::{TagEntry, TagValue};
///
/// // A typical EXIF tag entry
/// let entry = TagEntry {
///     group: "EXIF".to_string(),
///     group1: "ExifIFD".to_string(),  // Located in ExifIFD subdirectory
///     name: "FNumber".to_string(),
///     value: TagValue::F64(4.0),      // Post-ValueConv: 4/1 → 4.0
///     print: TagValue::String("4.0".to_string()),       // Post-PrintConv: formatted for display
/// };
///
/// assert_eq!(entry.name, "FNumber");
///
/// // A tag with units in the display string
/// let focal_entry = TagEntry {
///     group: "EXIF".to_string(),
///     group1: "ExifIFD".to_string(),
///     name: "FocalLength".to_string(),
///     value: TagValue::F64(24.0),     // Numeric value
///     print: TagValue::String("24 mm".to_string()),     // Human-readable with units
/// };
///
/// assert_eq!(focal_entry.print, TagValue::String("24 mm".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    /// Tag group name (e.g., "EXIF", "GPS", "Canon", "MakerNotes")
    ///
    /// Groups follow ExifTool's naming conventions:
    /// - Main IFDs: "EXIF", "GPS", "IFD0", "IFD1"
    /// - Manufacturer: "Canon", "Nikon", "Sony", etc.
    /// - Sub-groups: "Canon::CameraSettings", etc.
    ///
    /// This corresponds to ExifTool's Group0 (format family).
    pub group: String,

    /// ExifTool Group1 (subdirectory location)
    ///
    /// Identifies the specific IFD or subdirectory where the tag was found:
    /// - "IFD0" - Main image IFD
    /// - "ExifIFD" - EXIF subdirectory (tag 0x8769)
    /// - "GPS" - GPS subdirectory (tag 0x8825)
    /// - "InteropIFD" - Interoperability subdirectory (tag 0xa005)
    /// - "MakerNotes" - Manufacturer-specific subdirectory (tag 0x927c)
    ///
    /// This field enables ExifTool-compatible group-based tag access patterns.
    pub group1: String,

    /// Tag name without group prefix (e.g., "FNumber", "ExposureTime")
    ///
    /// Names match ExifTool's tag naming exactly for compatibility.
    pub name: String,

    /// The logical value after ValueConv processing.
    ///
    /// This is the value you get with ExifTool's -# flag:
    /// - Rational values converted to floats (4/1 → 4.0)
    /// - APEX values converted to real units
    /// - Raw value if no ValueConv exists
    ///
    /// # Examples
    ///
    /// - FNumber: `TagValue::F64(4.0)` (from rational 4/1)
    /// - ExposureTime: `TagValue::F64(0.0005)` (from rational 1/2000)
    /// - Make: `TagValue::String("Canon")` (no ValueConv needed)
    pub value: TagValue,

    /// The display value after PrintConv processing.
    ///
    /// This can be either:
    /// - A string for human-readable formatting (e.g., "1/100", "24.0 mm", "Rotate 90 CW")
    /// - A numeric value for data that should remain numeric in JSON (e.g., ISO: 100, FNumber: 4.0)
    ///
    /// PrintConv functions decide the appropriate type based on the tag's semantics:
    /// - Display-oriented tags return strings
    /// - Data-oriented tags may pass through numeric values
    ///
    /// If no PrintConv exists, this equals the original `value`.
    ///
    /// # Design Note
    ///
    /// This differs from ExifTool where PrintConv always returns strings.
    /// We chose this approach to avoid regex-based type guessing during JSON serialization.
    /// See docs/design/PRINTCONV-DESIGN-DECISIONS.md for details.
    pub print: TagValue,
}

/// Represents extracted EXIF data from an image
///
/// This matches ExifTool's JSON output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifData {
    /// Source file path
    #[serde(rename = "SourceFile")]
    pub source_file: String,

    /// Version of exif-oxide
    #[serde(rename = "ExifToolVersion", skip_serializing_if = "String::is_empty")]
    pub exif_tool_version: String,

    /// All extracted tags with both value and print representations
    #[serde(skip)]
    pub tags: Vec<TagEntry>,

    /// Legacy field for backward compatibility - will be populated during serialization
    /// TODO: Remove this once all consumers are updated to use TagEntry
    #[serde(flatten)]
    pub legacy_tags: IndexMap<String, TagValue>,

    /// Any errors encountered during processing
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,

    /// Missing implementations (only included with --show-missing)
    #[serde(
        rename = "MissingImplementations",
        skip_serializing_if = "Option::is_none"
    )]
    pub missing_implementations: Option<Vec<String>>,
}

impl ExifData {
    /// Create a new ExifData with empty tags
    pub fn new(source_file: String, exif_tool_version: String) -> Self {
        Self {
            source_file,
            exif_tool_version,
            tags: Vec::new(),
            legacy_tags: IndexMap::new(),
            errors: Vec::new(),
            missing_implementations: None,
        }
    }

    /// Get group priority for ExifTool-compatible ordering
    /// Returns lower numbers for groups that should appear first
    fn get_group_priority(tag_key: &str) -> u8 {
        if tag_key == "SourceFile" {
            return 0;
        }
        if tag_key == "ExifToolVersion" {
            return 1;
        }

        // Extract group prefix from "Group:TagName" format
        if let Some(group) = tag_key.split(':').next() {
            match group {
                "File" => 2,
                "JFIF" | "APP" | "APP0" | "APP1" | "APP2" | "APP3" | "APP4" | "APP5" | "APP6"
                | "APP7" | "APP8" | "APP9" | "APP10" | "APP11" | "APP12" | "APP13" | "APP14"
                | "APP15" => 3,
                "EXIF" => 4,
                "MakerNotes" => 5,
                "Composite" => 255, // Always last
                // Other groups (XMP, IPTC, Photoshop, PrintIM, MPF, ICC_Profile, etc.)
                _ => 50,
            }
        } else {
            // Tags without group prefix get high priority (like SourceFile, ExifToolVersion)
            10
        }
    }

    /// Convert tags to legacy format for JSON serialization
    /// This populates legacy_tags from the TagEntry vector
    ///
    /// `tag_requests` is the command-line request list in order; the first request
    /// matching a tag decides whether its ValueConv or PrintConv value is emitted.
    pub fn prepare_for_serialization(&mut self, tag_requests: Option<&[TagRequest]>) {
        use tracing::debug;

        // Preserve existing legacy_tags (like System: and Warning: tags) before clearing
        let existing_legacy_tags = self.legacy_tags.clone();
        self.legacy_tags.clear();

        // Re-add preserved legacy tags that don't come from TagEntry
        for (key, value) in existing_legacy_tags {
            if key.starts_with("System:") || key.starts_with("Warning:") {
                self.legacy_tags.insert(key, value);
            }
        }

        // Create a sorted list of (tag_key, tag_entry) pairs for ordered insertion
        let mut tag_pairs: Vec<(String, &TagEntry)> = self
            .tags
            .iter()
            .map(|entry| (format!("{}:{}", entry.group, entry.name), entry))
            .collect();

        // Sort by group priority first, then alphabetically within group
        tag_pairs.sort_by(|(key_a, _), (key_b, _)| {
            let priority_a = Self::get_group_priority(key_a);
            let priority_b = Self::get_group_priority(key_b);

            match priority_a.cmp(&priority_b) {
                std::cmp::Ordering::Equal => key_a.cmp(key_b), // Alphabetical within group
                other => other,
            }
        });

        // Insert tags in the sorted order
        for (key, entry) in tag_pairs {
            // Determine whether to use value or print field. Numeric requests support
            // wildcards (`-*Duration*#`) just like plain tag requests do, and the
            // earliest matching request decides.
            // ExifTool: lib/Image/ExifTool.pm:5345-5437, 3266-3290
            let should_use_value = tag_requests
                .map(|requests| {
                    FilterOptions::numeric_request_matches(
                        requests,
                        &entry.name,
                        &[&entry.group, &entry.group1],
                    )
                })
                .unwrap_or(false);

            if should_use_value {
                // Use value field for -# tags
                debug!("Tag {}: using numeric value {:?}", key, entry.value);
                self.legacy_tags.insert(key, entry.value.clone());
            } else {
                // Use PrintConv result directly - it already has the correct type
                // (string for display values, numeric for data values)
                debug!("Tag {}: using print value {:?}", key, entry.print);
                self.legacy_tags.insert(key, entry.print.clone());
            }
        }
    }

    /// Get all ExifIFD tags specifically
    /// ExifTool compatibility: access tags by Group1 location
    pub fn get_exif_ifd_tags(&self) -> Vec<&TagEntry> {
        self.tags
            .iter()
            .filter(|tag| tag.group1 == "ExifIFD")
            .collect()
    }

    /// Get all tags from a specific Group1 (subdirectory location)
    /// ExifTool: Group1-based filtering
    ///
    /// # Examples
    /// ```no_run
    /// use exif_oxide::formats::extract_metadata;
    ///
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false, false, None).unwrap();
    ///
    /// // Get all GPS tags
    /// let gps_tags = exif_data.get_tags_by_group1("GPS");
    ///
    /// // Get all ExifIFD tags
    /// let exif_ifd_tags = exif_data.get_tags_by_group1("ExifIFD");
    /// ```
    pub fn get_tags_by_group1(&self, group1_name: &str) -> Vec<&TagEntry> {
        self.tags
            .iter()
            .filter(|tag| tag.group1 == group1_name)
            .collect()
    }

    /// ExifTool compatibility: get tag by group-qualified name
    /// Supports both Group0 and Group1 based access
    ///
    /// # Examples
    /// ```no_run
    /// use exif_oxide::formats::extract_metadata;
    ///
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false, false, None).unwrap();
    ///
    /// // Access by Group1 (subdirectory location)
    /// let exposure_time = exif_data.get_tag_by_group("ExifIFD", "ExposureTime");
    ///
    /// // Access by Group0 (format family)
    /// let make = exif_data.get_tag_by_group("EXIF", "Make");
    /// ```
    pub fn get_tag_by_group(&self, group_name: &str, tag_name: &str) -> Option<&TagEntry> {
        self.tags.iter().find(|tag| {
            (tag.group == group_name || tag.group1 == group_name) && tag.name == tag_name
        })
    }

    /// ExifTool-style group access: EXIF:ExposureTime vs ExifIFD:ExposureTime
    /// Parses qualified tag names in "Group:TagName" format
    ///
    /// # Examples
    /// ```no_run
    /// use exif_oxide::formats::extract_metadata;
    ///
    /// let exif_data = extract_metadata(std::path::Path::new("image.jpg"), false, false, None).unwrap();
    ///
    /// let exposure_time = exif_data.get_tag_exiftool_style("ExifIFD:ExposureTime");
    /// let gps_lat = exif_data.get_tag_exiftool_style("GPS:GPSLatitude");
    /// ```
    pub fn get_tag_exiftool_style(&self, qualified_name: &str) -> Option<&TagEntry> {
        if let Some((group, name)) = qualified_name.split_once(':') {
            self.get_tag_by_group(group, name)
        } else {
            self.get_tag_by_name(qualified_name)
        }
    }

    /// Get tag by name (without group qualifier)
    /// Returns the highest priority matching tag found
    /// ExifTool behavior: EXIF tags take precedence over MakerNotes tags
    pub fn get_tag_by_name(&self, tag_name: &str) -> Option<&TagEntry> {
        let matching_tags: Vec<&TagEntry> = self
            .tags
            .iter()
            .filter(|tag| tag.name == tag_name)
            .collect();

        if matching_tags.is_empty() {
            return None;
        }

        // If only one match, return it
        if matching_tags.len() == 1 {
            return Some(matching_tags[0]);
        }

        // Multiple matches - use priority-based selection
        // ExifTool behavior: EXIF group takes precedence over MakerNotes group
        matching_tags
            .into_iter()
            .max_by_key(|tag| SourcePriority::from_namespace(&tag.group))
    }
}

/// Directory processing context for nested IFD processing
/// Matches ExifTool's $dirInfo hash structure
#[derive(Debug, Clone)]
pub struct DirectoryInfo {
    /// Directory name for debugging and PATH tracking
    pub name: String,
    /// Start offset of directory within data
    pub dir_start: usize,
    /// Length of directory data
    pub dir_len: usize,
    /// Base offset for pointer calculations (ExifTool's Base)
    pub base: u64,
    /// File position of data block (ExifTool's DataPos)
    pub data_pos: u64,
    /// Whether this directory allows reprocessing (ALLOW_REPROCESS)
    pub allow_reprocess: bool,
}

/// Data member value for tag dependencies
/// ExifTool: DataMember mechanism for inter-tag dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum DataMemberValue {
    U8(u8),
    U16(u16),
    U32(u32),
    String(String),
}

impl DataMemberValue {
    pub fn as_u16(&self) -> Option<u16> {
        match self {
            DataMemberValue::U16(v) => Some(*v),
            DataMemberValue::U8(v) => Some(*v as u16),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            DataMemberValue::U32(v) => Some(*v),
            DataMemberValue::U16(v) => Some(*v as u32),
            DataMemberValue::U8(v) => Some(*v as u32),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            DataMemberValue::U32(v) => Some(*v as usize),
            DataMemberValue::U16(v) => Some(*v as usize),
            DataMemberValue::U8(v) => Some(*v as usize),
            _ => None,
        }
    }
}

/// Source priority for tag conflict resolution
/// Higher numbers take precedence over lower numbers
/// ExifTool behavior: Main EXIF tags override MakerNote tags with same ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourcePriority {
    /// Unknown or unrecognized source (lowest priority)
    Unknown = 10,
    /// MakerNote tags (manufacturer-specific data)
    MakerNotes = 50,
    /// GPS IFD tags
    Gps = 80,
    /// Main EXIF tags (highest priority)
    /// ExifTool: IFD0, IFD1, ExifIFD, etc.
    Exif = 100,
}

impl SourcePriority {
    /// Get priority for a namespace string
    /// Matches ExifTool's group hierarchy behavior
    pub fn from_namespace(namespace: &str) -> Self {
        match namespace {
            "EXIF" | "IFD0" | "IFD1" | "ExifIFD" | "SubIFD" => SourcePriority::Exif,
            "GPS" => SourcePriority::Gps,
            "MakerNotes" => SourcePriority::MakerNotes,
            _ => SourcePriority::Unknown,
        }
    }
}

/// Enhanced tag source information for conflict resolution
/// Tracks where each tag came from and its processing context
#[derive(Debug, Clone)]
pub struct TagSourceInfo {
    /// Namespace/group for the tag (e.g., "EXIF", "MakerNotes", "GPS")
    /// ExifTool: Group 0 in tag name "Group:TagName"
    pub namespace: String,
    /// Specific IFD or table name (e.g., "IFD0", "ExifIFD", "Canon::Main")
    /// ExifTool: Directory path for debugging and processing context
    pub ifd_name: String,
    /// Source priority for conflict resolution
    /// ExifTool: Main EXIF tags take precedence over MakerNote tags
    pub priority: SourcePriority,
    /// Processor name that handled this tag
    /// ExifTool: PROCESS_PROC information
    pub processor_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_glob_pattern() {
        // Test prefix wildcard
        assert!(FilterOptions::matches_glob_pattern("GPSAltitude", "GPS*"));
        assert!(FilterOptions::matches_glob_pattern("GPSLatitude", "GPS*"));
        assert!(!FilterOptions::matches_glob_pattern("Altitude", "GPS*"));

        // Test suffix wildcard
        assert!(FilterOptions::matches_glob_pattern("GPSLatitude", "*tude"));
        assert!(FilterOptions::matches_glob_pattern("Altitude", "*tude"));
        assert!(!FilterOptions::matches_glob_pattern("GPSAltitu", "*tude"));

        // Test middle wildcard
        assert!(FilterOptions::matches_glob_pattern("CreateDate", "*Date*"));
        assert!(FilterOptions::matches_glob_pattern(
            "DateTimeOriginal",
            "*Date*"
        ));
        assert!(!FilterOptions::matches_glob_pattern("CreateTime", "*Date*"));

        // Test case insensitive
        assert!(FilterOptions::matches_glob_pattern("gpsaltitude", "GPS*"));
        assert!(FilterOptions::matches_glob_pattern("GPSAltitude", "gps*"));
    }

    /// ExifTool: lib/Image/ExifTool.pm:5376-5382 (SetFoundTags) translates a
    /// requested tag name containing `*` or `?` into `[-\w]*` / `[-\w]` and
    /// matches it case-insensitively against the whole tag name.
    #[test]
    fn test_matches_glob_pattern_exiftool_semantics() {
        // '?' matches exactly one word character
        assert!(FilterOptions::matches_glob_pattern("Duration", "Dur?tion"));
        assert!(!FilterOptions::matches_glob_pattern(
            "Duration",
            "Duration?"
        ));
        assert!(!FilterOptions::matches_glob_pattern("Durtion", "Dur?tion"));

        // More than one '*' in a pattern
        assert!(FilterOptions::matches_glob_pattern(
            "SubSecTimeOriginal",
            "Sub*Time*"
        ));
        assert!(!FilterOptions::matches_glob_pattern(
            "SubSecDateOriginal",
            "Sub*Time*"
        ));

        // '*' expands to [-\w]* so it never crosses the group separator
        assert!(!FilterOptions::matches_glob_pattern(
            "QuickTime:Duration",
            "Quick*Duration"
        ));
        // ...but an explicit group prefix in the pattern still matches
        assert!(FilterOptions::matches_glob_pattern(
            "QuickTime:Duration",
            "QuickTime:*Duration*"
        ));
    }

    /// ExifTool: lib/Image/ExifTool.pm:5364-5382 - the `#` suffix is stripped from
    /// the tag portion of the request and the remainder is matched with wildcards,
    /// so `-*Duration*#` requests numeric output for every matching tag.
    #[test]
    fn test_should_use_numeric_with_glob_pattern() {
        let filter_opts = FilterOptions::from_requests(vec![TagRequest::new("*Duration*", true)]);

        assert_eq!(filter_opts.glob_patterns, vec!["*Duration*"]);
        assert!(filter_opts.should_use_numeric("Duration", "QuickTime"));
        assert!(filter_opts.should_use_numeric("TrackDuration", "QuickTime"));
        assert!(!filter_opts.should_use_numeric("ImageWidth", "QuickTime"));
    }

    /// ExifTool matches requested tag names case-insensitively (`/^$tag$/i`).
    /// ExifTool: lib/Image/ExifTool.pm:5382
    #[test]
    fn test_should_use_numeric_is_case_insensitive() {
        let filter_opts = FilterOptions::from_requests(vec![TagRequest::new("orientation", true)]);

        assert!(filter_opts.should_use_numeric("Orientation", "EXIF"));
    }

    /// A group-qualified numeric request only applies to that group.
    /// ExifTool: lib/Image/ExifTool.pm:5348-5360, 5398-5401 (group is split off the
    /// request and used to filter the wildcard matches).
    #[test]
    fn test_should_use_numeric_with_group_qualified_pattern() {
        let filter_opts =
            FilterOptions::from_requests(vec![TagRequest::new("QuickTime:*Duration*", true)]);

        assert!(filter_opts.should_use_numeric("Duration", "QuickTime"));
        assert!(!filter_opts.should_use_numeric("Duration", "EXIF"));
    }

    /// The `#` is stripped before the request is classified, so `-Group:all#` picks
    /// the group-all path rather than being mistaken for a tag named "EXIF:all".
    /// ExifTool: lib/Image/ExifTool.pm:5364 (`$byValue = 1 if $tag =~ s/#$//`)
    #[test]
    fn test_tag_request_parse_strips_numeric_suffix() {
        assert_eq!(
            TagRequest::parse("Duration"),
            TagRequest::new("Duration", false)
        );
        assert_eq!(
            TagRequest::parse("Duration#"),
            TagRequest::new("Duration", true)
        );
        assert_eq!(
            TagRequest::parse("*Duration*#"),
            TagRequest::new("*Duration*", true)
        );
        assert_eq!(
            TagRequest::parse("EXIF:all#"),
            TagRequest::new("EXIF:all", true)
        );
        // A lone '#' is not a numeric marker for an empty request
        assert_eq!(TagRequest::parse("#"), TagRequest::new("#", false));
    }

    /// The first request that matches a tag decides how it is printed.
    ///
    /// Probed against vendored ExifTool 13.59 with test-images/apple/IMG_3755.MOV:
    ///
    /// ```text
    /// exiftool -j -Duration "-*Duration*#"  => "Duration": "2.96 s", "TrackDuration": 2.965
    /// exiftool -j "-*Duration*#" -Duration  => "Duration": 2.965
    /// ```
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5345-5437 (SetFoundTags appends each request's
    /// matches in order), 3266-3290 (GetInfo renames by-value entries), exiftool:2947-2953
    /// (the JSON writer keeps the first entry per tag name).
    #[test]
    fn test_should_use_numeric_honours_request_order() {
        let print_first = FilterOptions::from_requests(vec![
            TagRequest::new("Duration", false),
            TagRequest::new("*Duration*", true),
        ]);
        assert!(!print_first.should_use_numeric("Duration", "QuickTime"));
        assert!(print_first.should_use_numeric("TrackDuration", "QuickTime"));

        let numeric_first = FilterOptions::from_requests(vec![
            TagRequest::new("*Duration*", true),
            TagRequest::new("Duration", false),
        ]);
        assert!(numeric_first.should_use_numeric("Duration", "QuickTime"));
        assert!(numeric_first.should_use_numeric("TrackDuration", "QuickTime"));
    }

    /// A request naming a group the tag is not in never matches, so the decision
    /// falls through to the next request.
    ///
    /// Probed (ExifTool 13.59, IMG_3755.MOV):
    /// `exiftool -j "-EXIF:Duration#" -Duration`      => `"Duration": "2.96 s"`
    /// `exiftool -j "-QuickTime:Duration#" -Duration` => `"Duration": 2.965`
    /// `exiftool -j -Duration "-QuickTime:Duration#"` => `"Duration": "2.96 s"`
    #[test]
    fn test_should_use_numeric_skips_requests_for_other_groups() {
        let wrong_group = FilterOptions::from_requests(vec![
            TagRequest::new("EXIF:Duration", true),
            TagRequest::new("Duration", false),
        ]);
        assert!(!wrong_group.should_use_numeric("Duration", "QuickTime"));

        let right_group = FilterOptions::from_requests(vec![
            TagRequest::new("QuickTime:Duration", true),
            TagRequest::new("Duration", false),
        ]);
        assert!(right_group.should_use_numeric("Duration", "QuickTime"));

        let bare_first = FilterOptions::from_requests(vec![
            TagRequest::new("Duration", false),
            TagRequest::new("QuickTime:Duration", true),
        ]);
        assert!(!bare_first.should_use_numeric("Duration", "QuickTime"));
    }

    /// `all` (and `*`) match every tag, so an `-all` request takes its turn in the
    /// order like any other request.
    ///
    /// Probed (ExifTool 13.59, test-images/canon/eos_5d_mark_iii.jpg):
    /// `exiftool -j -all -Orientation#`         => `"Orientation": "Horizontal (normal)"`
    /// `exiftool -j -Orientation# -all`         => `"Orientation": 1`
    /// `exiftool -j "-EXIF:all" -Orientation#`  => `"Orientation": "Horizontal (normal)"`
    /// `exiftool -j -ALL` matches `-all` exactly (351 output lines both ways).
    /// ExifTool: lib/Image/ExifTool.pm:5350, 5367 (`/^(\*|all)$/i`)
    #[test]
    fn test_all_keyword_participates_in_request_order() {
        let all_first = FilterOptions::from_requests(vec![
            TagRequest::new("all", false),
            TagRequest::new("Orientation", true),
        ]);
        assert!(all_first.extract_all);
        assert!(!all_first.should_use_numeric("Orientation", "EXIF"));

        let numeric_first = FilterOptions::from_requests(vec![
            TagRequest::new("Orientation", true),
            TagRequest::new("all", false),
        ]);
        assert!(numeric_first.extract_all);
        assert!(numeric_first.should_use_numeric("Orientation", "EXIF"));

        let group_all_first = FilterOptions::from_requests(vec![
            TagRequest::new("EXIF:all", false),
            TagRequest::new("Orientation", true),
        ]);
        assert!(!group_all_first.should_use_numeric("Orientation", "EXIF"));
        assert!(
            group_all_first.should_use_numeric("Orientation", "MakerNotes"),
            "EXIF:all does not match a MakerNotes tag, so -Orientation# decides"
        );
    }

    /// `prepare_for_serialization` is the JSON-output path used by the CLI and by
    /// `extract_metadata_json`; it must honour the same wildcard semantics.
    /// ExifTool: lib/Image/ExifTool.pm:5376-5382
    #[test]
    fn test_prepare_for_serialization_numeric_glob_pattern() {
        let mut data = ExifData::new("test.mov".to_string(), "0.1.0-oxide".to_string());
        data.tags = vec![
            TagEntry {
                group: "QuickTime".to_string(),
                group1: "QuickTime".to_string(),
                name: "Duration".to_string(),
                value: TagValue::F64(2.965),
                print: TagValue::String("2.96 s".to_string()),
            },
            TagEntry {
                group: "QuickTime".to_string(),
                group1: "QuickTime".to_string(),
                name: "ImageWidth".to_string(),
                value: TagValue::U32(1920),
                print: TagValue::String("1920 px".to_string()),
            },
        ];

        data.prepare_for_serialization(Some(&[TagRequest::new("*Duration*", true)]));

        assert_eq!(
            data.legacy_tags.get("QuickTime:Duration"),
            Some(&TagValue::F64(2.965)),
            "-*Duration*# must select the ValueConv result"
        );
        assert_eq!(
            data.legacy_tags.get("QuickTime:ImageWidth"),
            Some(&TagValue::String("1920 px".to_string())),
            "non-matching tags must keep their PrintConv result"
        );
    }

    /// The JSON path must honour request order, not just the presence of a `#`.
    ///
    /// Probed against vendored ExifTool 13.59 with test-images/apple/IMG_3755.MOV:
    ///
    /// ```text
    /// exiftool -j -G -Duration "-*Duration*#"
    ///   => "QuickTime:Duration": "2.96 s", "QuickTime:TrackDuration": 2.965
    /// exiftool -j -G "-*Duration*#" -Duration
    ///   => "QuickTime:Duration": 2.965,   "QuickTime:TrackDuration": 2.965
    /// ```
    #[test]
    fn test_prepare_for_serialization_honours_request_order() {
        let quicktime_tags = || {
            vec![
                TagEntry {
                    group: "QuickTime".to_string(),
                    group1: "QuickTime".to_string(),
                    name: "Duration".to_string(),
                    value: TagValue::F64(2.965),
                    print: TagValue::String("2.96 s".to_string()),
                },
                TagEntry {
                    group: "QuickTime".to_string(),
                    group1: "QuickTime".to_string(),
                    name: "TrackDuration".to_string(),
                    value: TagValue::F64(2.965),
                    print: TagValue::String("2.96 s".to_string()),
                },
            ]
        };

        let mut print_first = ExifData::new("test.mov".to_string(), "0.1.0-oxide".to_string());
        print_first.tags = quicktime_tags();
        print_first.prepare_for_serialization(Some(&[
            TagRequest::new("Duration", false),
            TagRequest::new("*Duration*", true),
        ]));
        assert_eq!(
            print_first.legacy_tags.get("QuickTime:Duration"),
            Some(&TagValue::String("2.96 s".to_string())),
            "-Duration came first, so Duration keeps its PrintConv result"
        );
        assert_eq!(
            print_first.legacy_tags.get("QuickTime:TrackDuration"),
            Some(&TagValue::F64(2.965)),
            "TrackDuration is only matched by the numeric wildcard"
        );

        let mut numeric_first = ExifData::new("test.mov".to_string(), "0.1.0-oxide".to_string());
        numeric_first.tags = quicktime_tags();
        numeric_first.prepare_for_serialization(Some(&[
            TagRequest::new("*Duration*", true),
            TagRequest::new("Duration", false),
        ]));
        assert_eq!(
            numeric_first.legacy_tags.get("QuickTime:Duration"),
            Some(&TagValue::F64(2.965)),
            "-*Duration*# came first, so Duration prints its ValueConv result"
        );
    }

    #[test]
    fn test_should_extract_tag_with_glob_patterns() {
        let filter_opts = FilterOptions {
            requested_tags: Vec::new(),
            requested_groups: Vec::new(),
            group_all_patterns: Vec::new(),
            extract_all: false,
            tag_requests: Vec::new(),
            glob_patterns: vec!["GPS*".to_string()],
            compute_image_hash: false,
            image_hash_type: ImageHashType::default(),
        };

        // Should match GPS tags
        assert!(filter_opts.should_extract_tag("GPSAltitude", "GPS"));
        assert!(filter_opts.should_extract_tag("GPSLatitude", "GPS"));
        assert!(filter_opts.should_extract_tag("GPSVersionID", "EXIF"));

        // Should not match non-GPS tags
        assert!(!filter_opts.should_extract_tag("Make", "EXIF"));
        // An unqualified pattern is matched against the tag NAME only, never against
        // "Group:TagName" - ExifTool splits the group off the request before matching,
        // and its wildcards expand to [-\w] which cannot match the ':' separator.
        // ExifTool: lib/Image/ExifTool.pm:5348-5382
        // Verified: `exiftool -j -G "-QuickTime*" IMG_3755.MOV` returns no tags even
        // though every tag in the file is in the QuickTime group, and
        // `exiftool -j -G "-EXIF*" Ricoh2.jpg` returns only Exif*-named tags.
        assert!(!filter_opts.should_extract_tag("Altitude", "GPS"));
        assert!(!filter_opts.should_extract_tag("Altitude", "EXIF")); // Different group
    }

    /// A wildcard is never allowed to expand into the group portion of a qualified
    /// request: ExifTool splits the group off first and rejects a group name holding
    /// anything outside `[-\w:]`.
    /// ExifTool: lib/Image/ExifTool.pm:5348-5359
    /// Verified: `exiftool -j -G "-Quick*:Duration#" IMG_3755.MOV` warns
    /// "Invalid group name 'Quick*'" and returns no tags, while `-*:Duration#` and
    /// `-QuickTime:*Duration*#` both return the numeric Duration.
    #[test]
    fn test_wildcard_never_expands_into_group_prefix() {
        let invalid_group =
            FilterOptions::from_requests(vec![TagRequest::new("Quick*:Duration", true)]);
        assert!(!invalid_group.should_extract_tag("Duration", "QuickTime"));
        assert!(!invalid_group.should_use_numeric("Duration", "QuickTime"));

        // A group portion of exactly '*' is valid and selects any group
        let any_group = FilterOptions::from_requests(vec![TagRequest::new("*:Duration", false)]);
        assert!(any_group.should_extract_tag("Duration", "QuickTime"));
    }

    /// An empty group portion imposes no group constraint.
    /// ExifTool: lib/Image/ExifTool.pm:5348 splits `:*Duration*` into group "" and tag
    /// "*Duration*", and `GroupMatches("")` constrains nothing.
    /// Verified: `exiftool -j -G "-:*Duration*#" IMG_3755.MOV` warns
    /// `Invalid TAG name: ":*Duration*#"` (exiftool:1445) but still returns every
    /// *Duration* tag numerically; `-:Duration` likewise returns just Duration, so the
    /// request is honoured rather than dropped.
    #[test]
    fn test_empty_group_prefix_imposes_no_group_constraint() {
        let filter_opts = FilterOptions::from_requests(vec![TagRequest::new(":*Duration*", true)]);

        assert!(filter_opts.should_extract_tag("TrackDuration", "QuickTime"));
        assert!(filter_opts.should_use_numeric("TrackDuration", "QuickTime"));
        assert!(!filter_opts.should_extract_tag("ImageWidth", "QuickTime"));
    }

    /// A group-qualified pattern still selects the whole group.
    /// Verified: `exiftool -j -G "-EXIF:*" Ricoh2.jpg` returns every EXIF tag.
    /// ExifTool: lib/Image/ExifTool.pm:5348-5350, 5367-5373
    #[test]
    fn test_should_extract_tag_with_group_qualified_glob() {
        let filter_opts = FilterOptions {
            extract_all: false,
            glob_patterns: vec!["EXIF:*".to_string()],
            ..Default::default()
        };

        assert!(filter_opts.should_extract_tag("Make", "EXIF"));
        assert!(filter_opts.should_extract_tag("GPSLatitude", "EXIF"));
        assert!(!filter_opts.should_extract_tag("MIMEType", "File"));
    }

    /// A bare group name matches *any* group family, so a family-1 (subdirectory)
    /// name selects tags the same way a family-0 name does.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5398-5401 passes the group portion to
    /// `GroupMatches`, which at :5238 expands the tag to every family via
    /// `GetGroup($tag, -1)` and at :5252 accepts a match in any of them.
    ///
    /// Verified on canon/eos_rebel_t3i.jpg, where `exiftool -j -G1 -FNumber -Make`
    /// reports `ExifIFD:FNumber` and `IFD0:Make`:
    ///   `exiftool -j -G "-ExifIFD:FNum?er#"` => `"EXIF:FNumber": 4`
    ///   `exiftool -j -G "-ExifIFD:Make"`     => no tags
    ///   `exiftool -j -G "-IFD0:Make"`        => `"EXIF:Make": "Canon"`
    #[test]
    fn test_family1_group_name_matches() {
        let filter_opts =
            FilterOptions::from_requests(vec![TagRequest::new("ExifIFD:FNum?er", true)]);

        assert!(filter_opts.should_extract_tag_in_groups("FNumber", &["EXIF", "ExifIFD"]));
        assert!(filter_opts.should_use_numeric_in_groups("FNumber", &["EXIF", "ExifIFD"]));
        // Same tag name, different subdirectory
        assert!(!filter_opts.should_extract_tag_in_groups("FNumber", &["MakerNotes", "Canon"]));
        // The family-0 name still works
        assert!(FilterOptions {
            extract_all: false,
            requested_tags: vec!["EXIF:FNumber".to_string()],
            ..Default::default()
        }
        .should_extract_tag_in_groups("FNumber", &["EXIF", "ExifIFD"]));
    }

    /// A number in front of a group name pins it to that family.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5227-5228 peels the family number off, :5243-5250
    /// compares only that family, and :5244 rejects families the tag does not have.
    ///
    /// Verified on canon/eos_rebel_t3i.jpg:
    ///   `exiftool -j -G "-1ExifIFD:FNumber"` => `"EXIF:FNumber": 4.0`
    ///   `exiftool -j -G "-0EXIF:FNumber"`    => `"EXIF:FNumber": 4.0`
    ///   `exiftool -j -G "-1EXIF:FNumber"`    => no tags (family 1 is ExifIFD)
    #[test]
    fn test_family_numbered_group_prefix() {
        let groups = ["EXIF", "ExifIFD"];
        for (request, expected) in [
            ("1ExifIFD:FNumber", true),
            ("0EXIF:FNumber", true),
            ("1EXIF:FNumber", false),
            ("0ExifIFD:FNumber", false),
        ] {
            let filter_opts = FilterOptions {
                extract_all: false,
                requested_tags: vec![request.to_string()],
                ..Default::default()
            };
            assert_eq!(
                filter_opts.should_extract_tag_in_groups("FNumber", &groups),
                expected,
                "-{request} against groups {groups:?}"
            );
        }
    }

    /// The group portion may name several groups; every one must match.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5348 splits at the *last* colon, so
    /// "EXIF:ExifIFD:FNumber" has the group portion "EXIF:ExifIFD"; `GroupMatches`
    /// then splits that on ':' (:5224) and requires all parts to match (:5255).
    ///
    /// Verified on canon/eos_rebel_t3i.jpg:
    ///   `exiftool -j -G "-EXIF:ExifIFD:FNumber"` => `"EXIF:FNumber": 4.0`
    ///   `exiftool -j -G "-EXIF:IFD0:FNumber"`    => no tags
    #[test]
    fn test_multi_family_group_spec() {
        let both = FilterOptions {
            extract_all: false,
            requested_tags: vec!["EXIF:ExifIFD:FNumber".to_string()],
            ..Default::default()
        };
        assert!(both.should_extract_tag_in_groups("FNumber", &["EXIF", "ExifIFD"]));

        let contradictory = FilterOptions {
            extract_all: false,
            requested_tags: vec!["EXIF:IFD0:FNumber".to_string()],
            ..Default::default()
        };
        assert!(!contradictory.should_extract_tag_in_groups("FNumber", &["EXIF", "ExifIFD"]));
    }

    /// A group portion of `all` means "any group", exactly like `*` - and a family
    /// number in front of `all` is ignored, because `GroupMatches` strips the number
    /// before it tests for `all`.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5350 (`$group =~ /^(\*|all)$/i`), :5227 (prefix
    /// stripped) and :5241 (`next if $grp eq '*' or $grp eq 'all'`).
    /// Verified on canon/eos_rebel_t3i.jpg: `-all:FNumber`, `-*:FNumber`,
    /// `-1all:FNumber` and `-0all:FNumber` all return EXIF:FNumber and
    /// MakerNotes:FNumber, while `-1*:FNumber` and `-0*:FNumber` return nothing
    /// ("1*" fails the `^[-\w:]*$` group-name check at :5357).
    #[test]
    fn test_group_all_means_any_group() {
        for request in ["all:FNumber", "*:FNumber", "1all:FNumber", "0all:FNumber"] {
            let filter_opts = FilterOptions {
                extract_all: false,
                requested_tags: vec![request.to_string()],
                ..Default::default()
            };
            assert!(filter_opts.should_extract_tag_in_groups("FNumber", &["EXIF", "ExifIFD"]));
            assert!(filter_opts.should_extract_tag_in_groups("FNumber", &["MakerNotes", "Canon"]));
            assert!(!filter_opts.should_extract_tag_in_groups("Orientation", &["EXIF", "IFD0"]));
        }
    }

    /// Exactly one trailing `#` is stripped from a request, by the argument parser.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5364 (`$tag =~ s/#$//`) removes one `#`; what is
    /// left goes through the ordinary branches. `all#` therefore reaches :5386, is
    /// sterilized to "all", and asks for a tag *named* "all" rather than for every tag.
    ///
    /// Verified on canon/eos_rebel_t3i.jpg:
    ///   `exiftool -j -G "-all##"` => no tags (warns `Invalid TAG name: "all##"`)
    ///   `exiftool -j -G "-*##"`   => every tag ("*#" keeps its wildcard after
    ///                                sterilization, so it still matches everything)
    #[test]
    fn test_only_one_numeric_suffix_is_stripped() {
        // What the argument parser hands us for `-all##`
        let doubled = FilterOptions {
            extract_all: false,
            requested_tags: vec!["all#".to_string()],
            ..Default::default()
        };
        assert!(!doubled.should_extract_tag("Orientation", "EXIF"));
        assert!(doubled.should_extract_tag("all", "EXIF"));

        // ...and for `-*##`
        let doubled_star = FilterOptions {
            extract_all: false,
            glob_patterns: vec!["*#".to_string()],
            ..Default::default()
        };
        assert!(doubled_star.should_extract_tag("Orientation", "EXIF"));
    }

    /// A tag name of `*` or `all` (case-insensitive) matches every tag.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5367-5368
    /// (`elsif ($tag =~ /^(\*|all)$/i)` - "tag name of '*' or 'all' matches all tags").
    /// Verified: `exiftool -j -G "-all#"`, `"-*#"` and `"-ALL#"` on
    /// canon/eos_rebel_t3i.jpg all return the same 329 output lines as an
    /// unfiltered run, with `"EXIF:Orientation": 8` instead of "Rotate 270 CW".
    #[test]
    fn test_all_and_star_match_every_tag() {
        for request in ["all", "ALL", "*"] {
            let filter_opts = FilterOptions::from_requests(vec![TagRequest::new(request, true)]);

            assert!(
                filter_opts.should_extract_tag("Orientation", "EXIF"),
                "-{request}# must select EXIF:Orientation"
            );
            assert!(
                filter_opts.should_extract_tag("MIMEType", "File"),
                "-{request}# must select File:MIMEType"
            );
            assert!(
                filter_opts.should_use_numeric("Orientation", "EXIF"),
                "-{request}# must select the ValueConv result"
            );
        }
    }

    /// `Group:all` restricts the "all tags" request to that group.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5348-5350 splits the group off, :5367 expands
    /// `all`, :5398-5401 filters the matches by group.
    /// Verified: `exiftool -j -G "-EXIF:all#" canon/eos_rebel_t3i.jpg` returns 52 EXIF
    /// tags numerically and no File tags.
    #[test]
    fn test_group_qualified_all_restricts_to_that_group() {
        let filter_opts = FilterOptions {
            extract_all: false,
            group_all_patterns: vec!["EXIF:all".to_string()],
            ..Default::default()
        };

        assert!(filter_opts.should_extract_tag("Orientation", "EXIF"));
        assert!(!filter_opts.should_extract_tag("MIMEType", "File"));
    }

    /// Characters outside `[-\w*?]` are deleted from the tag portion of a request.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5378 (`$tag =~ tr/-_A-Za-z0-9*?//dc;`) for
    /// wildcard requests, :5386 (`tr/-_A-Za-z0-9//dc`) for plain ones. `-j` enables
    /// Duplicates (exiftool:949), so the JSON CLI always reaches one of those two
    /// branches rather than the "Invalid tag name" branch at :5396.
    ///
    /// Verified on canon/eos_rebel_t3i.jpg - each warns `Invalid TAG name`
    /// (exiftool:1445) and then matches anyway:
    ///   `exiftool -j -G "-F*Num.ber"` => EXIF:FNumber, MakerNotes:FlashGuideNumber,
    ///                                    MakerNotes:FNumber, Composite:FileNumber
    ///   `exiftool -j -G "-FNum.ber"`  => EXIF:FNumber, MakerNotes:FNumber
    ///   `exiftool -j -G "-*Duration*.#"` on canon/eos_500d.mov => every *Duration* tag
    #[test]
    fn test_illegal_characters_are_sterilized() {
        let wildcard = FilterOptions {
            extract_all: false,
            glob_patterns: vec!["F*Num.ber".to_string()],
            ..Default::default()
        };
        assert!(wildcard.should_extract_tag("FNumber", "EXIF"));
        assert!(wildcard.should_extract_tag("FlashGuideNumber", "MakerNotes"));
        assert!(!wildcard.should_extract_tag("Orientation", "EXIF"));

        let plain = FilterOptions {
            extract_all: false,
            requested_tags: vec!["FNum.ber".to_string()],
            ..Default::default()
        };
        assert!(plain.should_extract_tag("FNumber", "EXIF"));
        assert!(!plain.should_extract_tag("FlashGuideNumber", "MakerNotes"));

        let duration = FilterOptions {
            extract_all: false,
            glob_patterns: vec!["*Duration*.".to_string()],
            ..Default::default()
        };
        assert!(duration.should_extract_tag("TrackDuration", "QuickTime"));
        assert!(!duration.should_extract_tag("ImageWidth", "QuickTime"));
    }

    /// An illegal character in the *group* portion is not sterilized: ExifTool warns
    /// and substitutes the group name 'invalid', which matches nothing.
    ///
    /// ExifTool: lib/Image/ExifTool.pm:5357-5359
    /// Verified: `exiftool -j -G "-EX.IF:FNumber" canon/eos_rebel_t3i.jpg` prints
    /// "Warning: Invalid group name 'EX.IF'" and returns no tags.
    #[test]
    fn test_illegal_group_name_matches_nothing() {
        let filter_opts = FilterOptions {
            extract_all: false,
            requested_tags: vec!["EX.IF:FNumber".to_string()],
            ..Default::default()
        };

        assert!(!filter_opts.should_extract_tag("FNumber", "EXIF"));
    }

    /// A wildcard *name* pattern never restricts the request to the File group,
    /// so it must not trigger the stat-only shortcut.
    ///
    /// ExifTool matches a wildcard request against every extracted tag name
    /// regardless of group (lib/Image/ExifTool.pm:5376-5382), so `File*` also picks
    /// up EXIF:FileSource:
    ///
    /// ```console
    /// $ third-party/exiftool/exiftool -j -G "-File*#" test-images/nikon/d3500.jpg
    /// {
    ///   "File:FileName": "d3500.jpg",
    ///   ...
    ///   "File:FilePermissions": 100664,
    ///   "File:FileType": "JPEG",
    ///   "File:FileTypeExtension": "JPG",
    ///   "EXIF:FileSource": 3
    /// }
    /// ```
    /// (vendored ExifTool 13.59, probed 2026-08-30)
    #[test]
    fn test_is_file_group_only_rejects_bare_name_patterns() {
        for pattern in ["GPS*", "File*", "file*", "MIMEType*", "*", "File?ype"] {
            let filter = FilterOptions {
                extract_all: false,
                glob_patterns: vec![pattern.to_string()],
                ..Default::default()
            };
            assert!(
                !filter.is_file_group_only(),
                "name pattern {pattern:?} can match tags outside the File group"
            );
        }
    }

    /// The stat-only shortcut stays available for requests whose *group* portion
    /// pins them to the File group, which is where its performance value lies.
    ///
    /// ```console
    /// $ third-party/exiftool/exiftool -j -G "-File:all" test-images/nikon/d3500.jpg
    /// { "File:FileName": ..., "File:MIMEType": "image/jpeg", ... }   # File group only
    /// ```
    /// (vendored ExifTool 13.59, probed 2026-08-30)
    #[test]
    fn test_is_file_group_only_allows_group_qualified_requests() {
        let group_all = FilterOptions {
            extract_all: false,
            group_all_patterns: vec!["File:all".to_string()],
            ..Default::default()
        };
        assert!(group_all.is_file_group_only());

        let group_only = FilterOptions::groups_only(vec!["File".to_string()]);
        assert!(group_only.is_file_group_only());

        for pattern in ["File:*", "file:*", "File:File*", "File:MIMEType"] {
            let filter = FilterOptions {
                extract_all: false,
                glob_patterns: vec![pattern.to_string()],
                ..Default::default()
            };
            assert!(
                filter.is_file_group_only(),
                "group-qualified pattern {pattern:?} should keep the File-only shortcut"
            );
        }

        // A different group, or a wildcard group, reaches beyond File.
        for pattern in ["EXIF:*", "*:File*", "*:*"] {
            let filter = FilterOptions {
                extract_all: false,
                glob_patterns: vec![pattern.to_string()],
                ..Default::default()
            };
            assert!(
                !filter.is_file_group_only(),
                "pattern {pattern:?} is not restricted to the File group"
            );
        }
    }

    /// Exact tag-name requests keep the shortcut: these names are only ever
    /// produced by the File-group emitters in `formats::extract_file_tags_only`.
    #[test]
    fn test_is_file_group_only_allows_exact_file_tag_names() {
        let filter = FilterOptions::tags_only(vec!["MIMEType".to_string(), "FileType".to_string()]);
        assert!(filter.is_file_group_only());

        let mixed =
            FilterOptions::tags_only(vec!["MIMEType".to_string(), "Orientation".to_string()]);
        assert!(!mixed.is_file_group_only());
    }
}

impl TagSourceInfo {
    /// Create new tag source info
    pub fn new(namespace: String, ifd_name: String, processor_name: String) -> Self {
        let priority = SourcePriority::from_namespace(&namespace);
        Self {
            namespace,
            ifd_name,
            priority,
            processor_name,
        }
    }

    /// Get the full tag name with namespace prefix
    /// ExifTool format: "Group:TagName"
    pub fn format_tag_name(&self, tag_name: &str) -> String {
        format!("{}:{}", self.namespace, tag_name)
    }

    /// Get ExifTool Group1 value based on IFD name
    /// ExifTool: Groups => { 1 => 'ExifIFD' } specification
    pub fn get_group1(&self) -> String {
        match self.ifd_name.as_str() {
            "ExifIFD" => "ExifIFD".to_string(),
            "GPS" => "GPS".to_string(),
            "InteropIFD" => "InteropIFD".to_string(),
            "MakerNotes" => "MakerNotes".to_string(),
            "IFD1" => "IFD1".to_string(),
            "KyoceraRaw" => "KyoceraRaw".to_string(),
            // Canon MakerNote subdirectory processing
            // ExifTool: MakerNotes.pm MakerNoteCanon -> Canon.pm Main table
            // The directory name becomes Group1 per ExifTool's SetGroup logic
            name if name.starts_with("Canon") => "Canon".to_string(),
            // Other manufacturer MakerNote subdirectories follow the same pattern
            name if name.starts_with("Nikon") => "Nikon".to_string(),
            name if name.starts_with("Sony") => "Sony".to_string(),
            name if name.starts_with("Olympus") => "Olympus".to_string(),
            name if name.starts_with("Panasonic") => "Panasonic".to_string(),
            name if name.starts_with("Pentax") => "Pentax".to_string(),
            name if name.starts_with("Fujifilm") => "Fujifilm".to_string(),
            // Default to IFD0 for main IFD and unknown IFDs
            _ => "IFD0".to_string(),
        }
    }

    /// Get ExifTool Group1 value with tag-specific overrides for correct context assignment
    /// ExifTool: Certain tags belong to specific contexts regardless of processing order
    /// Fixes issue where Canon MakerNotes processing steals ExifIFD tags like ColorSpace
    pub fn get_group1_with_tag_override(&self, tag_id: u16) -> String {
        // ExifIFD-specific tags should always have group1="ExifIFD" regardless of processing context
        // ExifTool: These tags are defined in Exif.pm ExifIFD table, not manufacturer tables
        match tag_id {
            // Core ExifIFD tags that should never be assigned to manufacturer context
            0x9000 => "ExifIFD".to_string(), // ExifVersion - Always in ExifIFD
            0xA000 => "ExifIFD".to_string(), // FlashpixVersion - Always in ExifIFD
            0xA001 => "ExifIFD".to_string(), // ColorSpace - Always in ExifIFD
            0xA002 => "ExifIFD".to_string(), // ExifImageWidth - Always in ExifIFD
            0xA003 => "ExifIFD".to_string(), // ExifImageHeight - Always in ExifIFD
            0xA005 => "ExifIFD".to_string(), // InteropIFD pointer - Always in ExifIFD
            // GPS IFD pointer should always have GPS group1
            0x8825 => "GPS".to_string(), // GPSInfo - Always GPS context
            // For all other tags, use normal context-based assignment
            _ => self.get_group1(),
        }
    }
}

/// Temporary placeholder for ProcessorDispatch during Phase 5 cleanup
/// TODO: Remove this once trait-based dispatch is fully integrated
#[derive(Debug, Clone, Default)]
pub struct ProcessorDispatch {
    pub subdirectory_overrides: HashMap<u16, String>,
    pub parameters: HashMap<String, String>,
}

impl ProcessorDispatch {
    pub fn with_table_processor(_processor: String) -> Self {
        Self::default()
    }
}
