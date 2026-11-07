//! A small board written here (no board files in the repo): outline, a large part, a tiny one, a DNP part, a test
//! point, an unplated hole and a part parked outside the board.

use pcba::board::{Board, PartKind};
use pcba::settings::FabSettings;

fn fp(
    reference: &str,
    lib: &str,
    at: (f64, f64),
    fab: (f64, f64),
    attr: &str,
    pads: &str,
) -> String {
    format!(
        r#"(footprint "{lib}" (layer "F.Cu") (at {} {}) {attr}
  (property "Reference" "{reference}" (at 0 0 0) (layer "F.SilkS"))
  (property "Value" "V_{reference}" (at 0 0 0) (layer "F.Fab"))
  (fp_rect (start {} {}) (end {} {}) (stroke (width 0.1) (type solid)) (fill no) (layer "F.Fab"))
  {pads})"#,
        at.0,
        at.1,
        -fab.0 / 2.0,
        -fab.1 / 2.0,
        fab.0 / 2.0,
        fab.1 / 2.0
    )
}

fn board() -> String {
    let smd = r#"(pad "1" smd rect (at -0.5 0) (size 0.5 0.5) (layers "F.Cu" "F.Mask")) (pad "2" smd rect (at 0.5 0) (size 0.5 0.5) (layers "F.Cu" "F.Mask"))"#;
    let tp = r#"(pad "1" smd circle (at 0 0) (size 1 1) (property pad_prop_testpoint) (layers "F.Cu" "F.Mask") (net "VBUS"))"#;
    let hole = r#"(pad "" np_thru_hole circle (at 0 0) (size 3.2 3.2) (drill 3.2) (layers "*.Cu" "*.Mask"))"#;
    format!(
        r#"(kicad_pcb (version 20260206) (generator "pcbnew")
  (gr_rect (start 0 0) (end 40 30) (stroke (width 0.1) (type solid)) (fill no) (layer "Edge.Cuts"))
  {}
  {}
  {}
  {}
  {}
  {}
)"#,
        fp(
            "U1",
            "Package_SO:SOIC-8",
            (10.0, 10.0),
            (5.0, 4.0),
            "(attr smd)",
            smd
        ),
        fp(
            "R1",
            "Resistor_SMD:R_0402",
            (20.0, 10.0),
            (1.0, 0.5),
            "(attr smd)",
            smd
        ),
        fp(
            "R2",
            "Resistor_SMD:R_0402",
            (25.0, 10.0),
            (1.0, 0.5),
            "(attr smd dnp)",
            smd
        ),
        fp(
            "TP1",
            "VHRD_TestPoint:TestPoint_Pad_D1.0mm",
            (30.0, 20.0),
            (1.0, 1.0),
            "(attr smd exclude_from_bom)",
            tp
        ),
        fp(
            "H1",
            "MountingHole:MountingHole_3.2mm",
            (5.0, 25.0),
            (3.2, 3.2),
            "(attr exclude_from_bom)",
            hole
        ),
        fp(
            "C9",
            "Capacitor_SMD:C_0402",
            (60.0, 10.0),
            (1.0, 0.5),
            "(attr smd)",
            smd
        ),
    )
}

#[test]
fn the_board_reads_into_parts_kinds_and_holes() {
    let b = Board::read(&board(), "t", &FabSettings::default()).unwrap();
    let kind = |r: &str| b.parts.iter().find(|p| p.reference == r).map(|p| p.kind);
    assert_eq!(kind("U1"), Some(PartKind::Component));
    assert_eq!(kind("TP1"), Some(PartKind::TestPoint));
    assert_eq!(kind("H1"), Some(PartKind::MountingHole));
    assert_eq!(b.off_board, vec!["C9".to_string()]);
    assert!(b.parts.iter().find(|p| p.reference == "R2").unwrap().dnp);
    assert_eq!(b.holes.len(), 1);
    assert!(!b.holes[0].plated);
}

#[test]
fn references_fit_inside_large_bodies_and_go_outside_small_ones() {
    let st = FabSettings::default();
    let b = Board::read(&board(), "t", &st).unwrap();
    let parts: Vec<_> = b.parts.iter().collect();
    let (labels, stats) = pcba::place::place(&parts, &st);
    let l = |r: &str| labels.iter().find(|l| l.text == r).unwrap();
    assert!(l("U1").leader.is_none(), "U1 fits inside its 5 x 4 mm body");
    assert!(
        l("R1").leader.is_some(),
        "R1's 1 x 0.5 mm body is too small"
    );
    assert_eq!(stats.collisions, 0);
    for (i, a) in labels.iter().enumerate() {
        for c in &labels[i + 1..] {
            assert!(
                !a.bbox.overlaps(&c.bbox),
                "{} and {} overlap",
                a.text,
                c.text
            );
        }
    }
}

#[test]
fn tables_split_fitted_dnp_test_points_and_drills() {
    let st = FabSettings::default();
    let b = Board::read(&board(), "t", &st).unwrap();
    let res = pcba::fab(&b, "t", &st);
    let t = &res.tables;
    let refs: Vec<_> = t.bom.iter().flat_map(|r| r.references.clone()).collect();
    assert_eq!(refs, ["R1", "U1"]);
    assert_eq!(t.dnp[0].references, ["R2"]);
    assert_eq!(t.test_points[0].reference, "TP1");
    assert_eq!(t.test_points[0].net.as_deref(), Some("VBUS"));
    // origin is the board's lower left corner, y up
    assert!((t.test_points[0].x - 30.0).abs() < 1e-6 && (t.test_points[0].y - 10.0).abs() < 1e-6);
    assert_eq!(t.drills.len(), 1);
    let names: Vec<_> = res.files.iter().map(|f| f.name.as_str()).collect();
    assert!(
        names.contains(&"t_assembly_top.svg")
            && names.contains(&"t_drill.svg")
            && names.contains(&"t_assembly.typ")
    );
    assert!(
        !names.contains(&"t_assembly_bottom.svg"),
        "no parts on the back"
    );
}
