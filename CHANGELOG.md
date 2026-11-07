# Changelog

## [Unreleased]

### Added

- `pcba fab BOARD.kicad_pcb`: assembly drawing in `fab/` beside the board: an SVG page per side with references
  fitted inside each part or outside with a leader, DNP parts crossed out, test points and fiducials marked, a drill
  map, tables (fitted parts, DNP, test points, drills) as JSON, the test points as CSV, and a Typst document that
  puts it all on A4. Settings in `pcba.toml` beside the board (`--show-settings` prints the defaults).

### Changed

- A Cargo workspace: the library `pcba` and the CLI `pcba_cli` (binary still `pcba`).
