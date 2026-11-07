//! The board as the assembly drawing sees it: outline, parts with their bodies and kinds, holes.
//! Read through better_erc's KiCad reader (`erc_kicad::pcb`).

use crate::geom::{Rect, mm, shape_bounds};
use crate::settings::FabSettings;
use erc_model::layout::{Footprint, Geom, Layout, Pad, Shape, Side};

/// What a footprint is on the drawing
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PartKind {
    Component,
    TestPoint,
    Fiducial,
    MountingHole,
}

/// A part to draw
#[derive(Clone, Debug)]
pub struct Part {
    pub reference: String,
    pub value: String,
    /// `lib:name` of the footprint
    pub footprint: String,
    pub mpn: Option<String>,
    pub side: Side,
    pub kind: PartKind,
    pub dnp: bool,
    pub exclude_from_bom: bool,
    /// Centre of the footprint (mm)
    pub at: (f64, f64),
    /// Body drawing: the fab layer of its side, else the courtyard, else nothing (pads give the box)
    pub body: Vec<Shape>,
    /// Box of the body (or of the pads), mm
    pub bbox: Rect,
    /// Test point: its pad centre and net
    pub probe: Option<((f64, f64), Option<String>)>,
}

/// A drilled hole (vias are not part of the drawing)
#[derive(Clone, Debug)]
pub struct Hole {
    pub at: (f64, f64),
    /// Drill diameter, mm; for a slot its width
    pub diameter: f64,
    /// Slot length, mm, when the drill is oval
    pub slot: Option<f64>,
    pub plated: bool,
    pub reference: String,
}

/// The board
#[derive(Clone, Debug)]
pub struct Board {
    pub name: String,
    pub outline: Vec<Shape>,
    pub bbox: Rect,
    pub parts: Vec<Part>,
    pub holes: Vec<Hole>,
    /// Origin of test point coordinates: the auxiliary origin when set, else the board's lower left corner (mm)
    pub origin: (f64, f64),
    /// Footprints outside the board outline, left out of the drawing
    pub off_board: Vec<String>,
}

fn layer_of(side: Side, front: &str, back: &str) -> String {
    match side {
        Side::Front => front.to_string(),
        Side::Back => back.to_string(),
    }
}

fn kind(fp: &Footprint, pads: &[&Pad]) -> PartKind {
    let props: Vec<&str> = pads.iter().filter_map(|p| p.property.as_deref()).collect();
    let lib = fp.lib_id.to_ascii_lowercase();
    if props.iter().any(|p| p.starts_with("pad_prop_fiducial")) || lib.contains("fiducial") {
        PartKind::Fiducial
    } else if props.contains(&"pad_prop_testpoint") || lib.contains("testpoint") {
        PartKind::TestPoint
    } else if lib.contains("mountinghole") {
        PartKind::MountingHole
    } else {
        PartKind::Component
    }
}

impl Board {
    /// Reads a `.kicad_pcb`; `name` is what pages and tables call the board
    pub fn read(text: &str, name: &str, settings: &FabSettings) -> Result<Board, String> {
        let (layout, _notes) = erc_kicad::pcb::read(text, name).map_err(|e| format!("{e:?}"))?;
        Ok(Board::from_layout(&layout, name, settings))
    }

    pub fn from_layout(layout: &Layout, name: &str, settings: &FabSettings) -> Board {
        let outline: Vec<Shape> = layout
            .graphics
            .iter()
            .filter(|s| s.layer == "Edge.Cuts")
            .cloned()
            .collect();
        let mut bbox = Rect::empty();
        for s in &outline {
            bbox = bbox.union(&shape_bounds(&s.geom));
        }
        let mut parts = Vec::new();
        let mut holes = Vec::new();
        // the reader links each pad to its footprint (Footprint::pads stays empty)
        let mut pads_of: Vec<Vec<&Pad>> = vec![Vec::new(); layout.footprints.len()];
        for pad in &layout.pads {
            pads_of[pad.footprint.index()].push(pad);
        }
        for (fi, fp) in layout.footprints.iter().enumerate() {
            let fp_pads = &pads_of[fi];
            for pad in fp_pads {
                let Some(drill) = pad.drill else { continue };
                if pad.kind != "thru_hole" && pad.kind != "np_thru_hole" {
                    continue;
                }
                let (a, b) = (mm(drill.x), mm(if drill.y > 0 { drill.y } else { drill.x }));
                holes.push(Hole {
                    at: (mm(pad.at.x), mm(pad.at.y)),
                    diameter: a.min(b),
                    slot: ((a - b).abs() > 1e-6).then(|| a.max(b)),
                    plated: pad.kind == "thru_hole",
                    reference: fp.reference.clone(),
                });
            }
            if fp_pads.is_empty()
                || settings
                    .skip_libraries
                    .iter()
                    .any(|p| fp.lib_id.starts_with(p.as_str()))
            {
                continue;
            }
            let fab = layer_of(fp.side, "F.Fab", "B.Fab");
            let crt = layer_of(fp.side, "F.CrtYd", "B.CrtYd");
            let mut body: Vec<Shape> = fp
                .graphics
                .iter()
                .filter(|s| s.layer == fab)
                .cloned()
                .collect();
            if body.is_empty() {
                body = fp
                    .graphics
                    .iter()
                    .filter(|s| s.layer == crt)
                    .cloned()
                    .collect();
            }
            let mut bb = Rect::empty();
            for s in &body {
                bb = bb.union(&shape_bounds(&s.geom));
            }
            if bb.is_empty() {
                for p in fp_pads {
                    let (w, h) = (mm(p.size.x) / 2.0, mm(p.size.y) / 2.0);
                    let r = w.max(h);
                    bb = bb.union(&Rect::new(
                        mm(p.at.x) - r,
                        mm(p.at.y) - r,
                        mm(p.at.x) + r,
                        mm(p.at.y) + r,
                    ));
                }
            }
            let kind = kind(fp, fp_pads);
            let probe = (kind == PartKind::TestPoint)
                .then(|| fp_pads.first())
                .flatten()
                .map(|p| ((mm(p.at.x), mm(p.at.y)), p.net_name.clone()));
            let mpn = fp
                .fields
                .iter()
                .find(|f| f.name.eq_ignore_ascii_case("mpn1") || f.name.eq_ignore_ascii_case("mpn"))
                .and_then(|f| f.value.get().cloned())
                .filter(|v| !v.is_empty());
            parts.push(Part {
                reference: fp.reference.clone(),
                value: fp.value.clone(),
                footprint: fp.lib_id.clone(),
                mpn,
                side: fp.side,
                kind,
                dnp: fp.attrs.dnp,
                exclude_from_bom: fp.attrs.exclude_from_bom,
                at: (mm(fp.at.x), mm(fp.at.y)),
                body,
                bbox: bb,
                probe,
            });
        }
        let origin = layout
            .setup
            .iter()
            .find(|(k, _)| k == "aux_axis_origin")
            .and_then(|(_, v)| {
                let mut it = v.split_whitespace().filter_map(|t| t.parse::<f64>().ok());
                Some((it.next()?, it.next()?))
            })
            .filter(|&(x, y)| x != 0.0 || y != 0.0)
            .unwrap_or((bbox.x0, bbox.y1));
        // parts parked beside the board (unplaced, spares) are not part of the assembly
        let mut off_board = Vec::new();
        if bbox.is_empty() {
            for p in &parts {
                bbox = bbox.union(&p.bbox);
            }
        } else {
            let inside = |at: (f64, f64)| {
                at.0 >= bbox.x0 && at.0 <= bbox.x1 && at.1 >= bbox.y0 && at.1 <= bbox.y1
            };
            off_board = parts
                .iter()
                .filter(|p| !inside(p.at))
                .map(|p| p.reference.clone())
                .collect();
            parts.retain(|p| inside(p.at));
            holes.retain(|h| inside(h.at));
        }
        off_board.sort_by(|a, b| crate::tables::natural(a, b));
        Board {
            name: name.to_string(),
            outline,
            bbox,
            parts,
            holes,
            origin,
            off_board,
        }
    }
}

/// Whether a shape is a closed outline (a fill or a rectangle, circle or polygon)
pub fn closed(s: &Shape) -> bool {
    matches!(s.geom, Geom::Rect(..) | Geom::Circle { .. } | Geom::Poly(_))
}
