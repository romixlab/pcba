//! The tables of the drawing: BOM of the fitted parts, DNP, test points, drills.

use crate::board::{Board, PartKind};
use crate::svg::DrillClass;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct BomRow {
    pub qty: usize,
    pub references: Vec<String>,
    pub value: String,
    pub footprint: String,
    pub mpn: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TestPointRow {
    pub reference: String,
    pub net: Option<String>,
    pub side: &'static str,
    /// From the board's origin, y up, mm
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DrillRow {
    pub symbol: usize,
    pub diameter: f64,
    pub slot: Option<f64>,
    pub plated: bool,
    pub count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Tables {
    pub bom: Vec<BomRow>,
    pub dnp: Vec<BomRow>,
    pub test_points: Vec<TestPointRow>,
    pub drills: Vec<DrillRow>,
}

/// `R2` before `R10`
pub fn natural(a: &str, b: &str) -> std::cmp::Ordering {
    let split = |s: &str| {
        let i = s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len());
        (
            s[..i].to_string(),
            s[i..].parse::<u64>().unwrap_or(0),
            s.to_string(),
        )
    };
    split(a).cmp(&split(b))
}

fn group(rows: impl Iterator<Item = (String, String, String, Option<String>)>) -> Vec<BomRow> {
    let mut out: Vec<BomRow> = Vec::new();
    for (r, value, footprint, mpn) in rows {
        match out
            .iter_mut()
            .find(|x| x.value == value && x.footprint == footprint && x.mpn == mpn)
        {
            Some(x) => x.references.push(r),
            None => out.push(BomRow {
                qty: 0,
                references: vec![r],
                value,
                footprint,
                mpn,
            }),
        }
    }
    for x in &mut out {
        x.references.sort_by(|a, b| natural(a, b));
        x.qty = x.references.len();
    }
    out.sort_by(|a, b| natural(&a.references[0], &b.references[0]));
    out
}

pub fn tables(board: &Board, drills: &[DrillClass]) -> Tables {
    let component = |p: &&crate::board::Part| p.kind == PartKind::Component && !p.exclude_from_bom;
    let row = |p: &crate::board::Part| {
        (
            p.reference.clone(),
            p.value.clone(),
            p.footprint.clone(),
            p.mpn.clone(),
        )
    };
    let bom = group(
        board
            .parts
            .iter()
            .filter(component)
            .filter(|p| !p.dnp)
            .map(row),
    );
    let dnp = group(board.parts.iter().filter(|p| p.dnp).map(row));
    let mut test_points: Vec<TestPointRow> = board
        .parts
        .iter()
        .filter(|p| p.kind == PartKind::TestPoint)
        .map(|p| {
            let (at, net) = p.probe.clone().unwrap_or((p.at, None));
            TestPointRow {
                reference: p.reference.clone(),
                net,
                side: if p.side == erc_model::layout::Side::Front {
                    "top"
                } else {
                    "bottom"
                },
                x: ((at.0 - board.origin.0) * 1000.0).round() / 1000.0,
                y: ((board.origin.1 - at.1) * 1000.0).round() / 1000.0,
            }
        })
        .collect();
    test_points.sort_by(|a, b| natural(&a.reference, &b.reference));
    let drills = drills
        .iter()
        .map(|c| DrillRow {
            symbol: c.symbol,
            diameter: c.diameter,
            slot: c.slot,
            plated: c.plated,
            count: c.count,
        })
        .collect();
    Tables {
        bom,
        dnp,
        test_points,
        drills,
    }
}

/// The test point table as CSV (for fixtures and flying probe)
pub fn test_points_csv(t: &Tables) -> String {
    let mut s = String::from("reference,net,side,x_mm,y_mm\n");
    for r in &t.test_points {
        s.push_str(&format!(
            "{},{},{},{},{}\n",
            r.reference,
            r.net.as_deref().unwrap_or(""),
            r.side,
            r.x,
            r.y
        ));
    }
    s
}
