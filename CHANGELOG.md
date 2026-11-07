# Changelog

## [Unreleased]

### Added

- `pcba fab BOARD.kicad_pcb`: assembly drawing in `fab/` beside the board: an SVG page per side with references
  fitted inside each part or outside with a leader, DNP parts crossed out, test points and fiducials marked, a drill
  map, tables (fitted parts, DNP, test points, drills) as JSON, the test points as CSV, and a Typst document that
  puts it all on A4. Settings in `pcba.toml` beside the board (`--show-settings` prints the defaults).
- `pcba fab`: references placed outside their part no longer sit on the board outline, holes (also those of parts
  on the other side), test point or fiducial symbols; the new setting `edge_gap` (0.15 mm) is the gap they keep. At
  the same distance a spot on the board wins over one off it, and a spot whose leader crosses no other label,
  leader or part body wins over a nearer one that does; a second pass places the references that still crossed
  something first and is kept when it leaves fewer crossings. `pcba fab` names how many still cross something
  (`PlaceStats::crossings`), and `place::audit` counts what a reader would see as clutter on a placed page.

### Changed

- better_erc pinned at `a1dadec`, whose board reader fills each footprint's pads (IMP-14); pcba's own grouping of
  the pads by footprint is gone.
- A Cargo workspace: the library `pcba` and the CLI `pcba_cli` (binary still `pcba`).
- `place::place` takes the board too (its outline and holes): `place(&board, &parts, &settings)`.
