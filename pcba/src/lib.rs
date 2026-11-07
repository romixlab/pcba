//! PCB assembly outputs from KiCad boards.
//!
//! [`fab`] makes the assembly drawing of a `.kicad_pcb`: a page per side with every reference fitted inside its
//! part (or outside with a leader where it cannot fit), DNP parts crossed out, test points and fiducials with
//! their own symbols, a drill map, and the tables (BOM of fitted parts, DNP, test points, drills). The pages are
//! SVG; a Typst file puts them and the tables on paper (`tpm doc build` stamps and files the PDF).

pub mod board;
pub mod geom;
pub mod place;
pub mod settings;
pub mod svg;
pub mod tables;
pub mod typst;

use board::{Board, Part};
use erc_model::layout::Side;
use place::PlaceStats;
use settings::FabSettings;

/// One output file
#[derive(Clone, Debug)]
pub struct Output {
    /// File name, without folder
    pub name: String,
    pub text: String,
}

/// What `fab` made
#[derive(Clone, Debug)]
pub struct FabResult {
    pub files: Vec<Output>,
    pub front: Option<PlaceStats>,
    pub back: Option<PlaceStats>,
    pub tables: tables::Tables,
}

/// The assembly drawing of a board; `stem` prefixes every file name (`b153a` gives `b153a_assembly_top.svg`...)
pub fn fab(board: &Board, stem: &str, settings: &FabSettings) -> FabResult {
    let mut files = Vec::new();
    let mut stats = [None, None];
    let mut pages = Vec::new();
    for (i, side) in [Side::Front, Side::Back].into_iter().enumerate() {
        let parts: Vec<&Part> = board.parts.iter().filter(|p| p.side == side).collect();
        if parts.is_empty() {
            continue;
        }
        let (labels, st) = place::place(board, &parts, settings);
        stats[i] = Some(st);
        let name = format!(
            "{stem}_assembly_{}.svg",
            if side == Side::Front { "top" } else { "bottom" }
        );
        files.push(Output {
            name: name.clone(),
            text: svg::assembly(board, side, &parts, &labels, settings),
        });
        pages.push((name, side));
    }
    let classes = svg::drill_classes(&board.holes);
    let drill_name = (!board.holes.is_empty()).then(|| format!("{stem}_drill.svg"));
    if let Some(n) = &drill_name {
        files.push(Output {
            name: n.clone(),
            text: svg::drill(board, &classes, settings),
        });
    }
    let tables = tables::tables(board, &classes);
    files.push(Output {
        name: format!("{stem}_testpoints.csv"),
        text: tables::test_points_csv(&tables),
    });
    files.push(Output {
        name: format!("{stem}_fab.json"),
        text: serde_json::to_string_pretty(&tables).expect("tables serialize"),
    });
    files.push(Output {
        name: format!("{stem}_assembly.typ"),
        text: typst::document(
            &board.name,
            &pages,
            drill_name.as_deref(),
            &tables,
            &classes,
            settings,
        ),
    });
    FabResult {
        files,
        front: stats[0],
        back: stats[1],
        tables,
    }
}
