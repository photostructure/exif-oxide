# TODO

Current priorities live in [docs/MILESTONES.md](docs/MILESTONES.md) and the
active TPPs in [_todo/](_todo/). Completed TPPs move to [_done/](_done/).
Deferred work gets a dated rationale note in [_paused/](_paused/) instead of
sitting stale in `_todo/` — see `_paused/WRITE-SUPPORT.md` for the write
support deferral.

This file previously referenced P13-olympus, P15-ifd-parsing, and
P17-string-formatting TPPs that no longer exist. If any item below is picked
back up, write a fresh TPP per [docs/TPP-GUIDE.md](docs/TPP-GUIDE.md) rather
than reviving old checkbox lists from chat logs.

## Backlog (no TPP yet)

- Olympus/Canon/Nikon/Sony/Fuji required-tags work — no active TPP, needs
  re-triage; a prior Olympus session left MakerNotes conditional dispatch
  partially wired (check git history around `process_tag_0x927c_subdirectory`
  before restarting from scratch)
- `BinaryDataTag::from_legacy` (`src/types/binary_data.rs:713`) — still has
  live callers in Canon binary data; open question of whether remaining
  consumers should be migrated off the legacy constructor
- Expression parser: function calls and string concatenation support — no
  active TPP
