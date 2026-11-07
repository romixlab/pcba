# Working on pcba

Opinionated PCB tooling for KiCad boards. A Cargo workspace:

- `pcba` (library): the board as the assembly drawing sees it (`board.rs`, read through better_erc's `erc_kicad::pcb`
  reader), reference placement (`place.rs`), SVG pages (`svg.rs`), tables (`tables.rs`), the Typst document
  (`typst.rs`) and every tunable number in `settings.rs` (`FabSettings`, a board may override any field in a
  `pcba.toml` beside it).
- `pcba_cli` (binary `pcba`): `pcba new` (board folder structure) and `pcba fab` (assembly drawing).
- `pcbapy/`: a Python Gerber renderer (pygerber), separate from the Rust crates.

KiCad files are read with better_erc's reader (git dependency in the workspace `Cargo.toml`, by release tag, or by
commit while a needed fix waits for its release); move the pin there when a reader fix lands in better_erc. Dependencies are pure Rust with `default-features = false`.

## Commands

```sh
just test      # cargo nextest on the workspace
just lint      # fmt check, clippy on the library
just install   # builds the deploy profile and puts `pcba` on PATH
```

Tests write their own boards (`pcba/tests/fab.rs`); no board files from hw or vendors are committed here.

## Tracking

[FEATURES.md](FEATURES.md) holds what exists with stable ids (`FAB-1`); [CHANGELOG.md](CHANGELOG.md) gets an entry
under `## [Unreleased]` for every visible change. Planning notes live in tpm (P2490 `design/pcba-fab.md`), not here.
