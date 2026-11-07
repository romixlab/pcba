# Features

- ✅ NEW-1 board-folder: `pcba new` creates a board's folder structure (simplified).
- ✅ FAB-1 assembly-pages: `pcba fab` draws one SVG page per side (the back mirrored, seen from below): board
  outline, part bodies from the fab layer (courtyard as fallback), references fitted inside their body (turned along
  tall bodies) or outside with a leader at a free spot, DNP parts dashed, crossed and greyed, test points and
  fiducials with their own symbols. Parts outside the board outline are left out and named.
- ✅ FAB-2 drill-map: drill page with one symbol per plating, diameter and slot; vias left out.
- ✅ FAB-3 fab-tables: BOM of fitted parts grouped by value, footprint and MPN; DNP list; test points with net and
  coordinates from the auxiliary origin (CSV too); drill table; all as JSON and in a Typst document.
- 📋 FAB-4 variants: DNP per assembly variant (KiCad 10 variants or mx3 once its BOM tooling is reworked); today the
  footprint's own `dnp` attribute decides.
- ✅ FAB-5 outline-aware-labels: references outside their part keep clear of the board outline line, holes, test
  point and fiducial symbols (`edge_gap`), take a spot on the board over one off it at the same distance, and
  take a spot whose leader crosses no other label, leader or part body over a nearer one that does.
- 📋 FAB-6 detail-views: zoomed detail views for clusters too dense for leaders.
