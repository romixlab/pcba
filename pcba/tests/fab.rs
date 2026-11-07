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
    let (labels, stats) = pcba::place::place(&b, &parts, &st);
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

// FAB-5: references placed outside keep clear of the outline, holes and symbols, and their leaders cross nothing

const SMD: &str = r#"(pad "1" smd rect (at -0.5 0) (size 0.5 0.5) (layers "F.Cu" "F.Mask")) (pad "2" smd rect (at 0.5 0) (size 0.5 0.5) (layers "F.Cu" "F.Mask"))"#;
const TP: &str = r#"(pad "1" smd circle (at 0 0) (size 1 1) (property pad_prop_testpoint) (layers "F.Cu" "F.Mask") (net "N"))"#;

fn resistor(reference: &str, at: (f64, f64)) -> String {
    fp(
        reference,
        "Resistor_SMD:R_0402",
        at,
        (1.0, 0.5),
        "(attr smd)",
        SMD,
    )
}

fn test_point(reference: &str, at: (f64, f64)) -> String {
    fp(
        reference,
        "VHRD_TestPoint:TestPoint_Pad_D1.0mm",
        at,
        (1.0, 1.0),
        "(attr smd exclude_from_bom)",
        TP,
    )
}

/// A 40 x 30 mm board with these footprints
fn board_of(fps: &[String]) -> String {
    format!(
        r#"(kicad_pcb (version 20260206) (generator "pcbnew")
  (gr_rect (start 0 0) (end 40 30) (stroke (width 0.1) (type solid)) (fill no) (layer "Edge.Cuts"))
  {}
)"#,
        fps.join("\n  ")
    )
}

/// Places the references of `text`'s board and checks the page for clutter
fn placed(text: &str) -> (Board, Vec<pcba::place::Label>, pcba::place::Audit) {
    let st = FabSettings::default();
    let b = Board::read(text, "t", &st).unwrap();
    let parts: Vec<_> = b.parts.iter().collect();
    let (labels, stats) = pcba::place::place(&b, &parts, &st);
    assert_eq!(stats.collisions, 0);
    let audit = pcba::place::audit(&b, &parts, &labels, &st);
    (b, labels, audit)
}

#[test]
fn a_reference_beside_the_board_edge_does_not_cross_it() {
    // tight against the left and the top edge: the nearest spots straddle the outline
    let (_, labels, audit) = placed(&board_of(&[
        resistor("R1", (0.9, 15.0)),
        resistor("R2", (20.0, 0.6)),
        test_point("TP1", (39.3, 29.3)),
        // on the edge line itself: spots above and below are as near, the one on the board wins
        resistor("R3", (30.0, 0.0)),
    ]));
    assert_eq!(audit.on_edge, 0, "{labels:#?}");
    for l in &labels {
        let c = l.bbox.center();
        assert!(
            c.0 > 0.0 && c.0 < 40.0 && c.1 > 0.0 && c.1 < 30.0,
            "{} should stay on the board when a spot inside is as near: {:?}",
            l.text,
            l.bbox
        );
    }
}

#[test]
fn a_reference_goes_off_the_board_when_no_spot_inside_is_near() {
    // a strip 1.6 mm high, filled with parts: the label of R1 has no room on it
    let mut fps = vec![resistor("R1", (10.0, 0.8))];
    for (i, x) in [4.0, 6.0, 8.0, 12.0, 14.0, 16.0].into_iter().enumerate() {
        fps.push(fp(
            &format!("U{i}"),
            "Package_SO:SOIC-8",
            (x, 0.8),
            (1.8, 1.4),
            "(attr smd)",
            SMD,
        ));
    }
    let text = format!(
        r#"(kicad_pcb (version 20260206) (generator "pcbnew")
  (gr_rect (start 0 0) (end 20 1.6) (stroke (width 0.1) (type solid)) (fill no) (layer "Edge.Cuts"))
  {}
)"#,
        fps.join("\n  ")
    );
    let (_, labels, audit) = placed(&text);
    assert_eq!(audit.on_edge, 0);
    let r1 = labels.iter().find(|l| l.text == "R1").unwrap();
    assert!(r1.bbox.y1 < 0.0 || r1.bbox.y0 > 1.6, "{:?}", r1.bbox);
}

#[test]
fn a_reference_keeps_off_a_mounting_hole_and_test_point_symbols() {
    // the hole's footprint is on the top; on the bottom page only its drill keeps the labels off
    let hole = r#"(pad "" np_thru_hole circle (at 0 0) (size 3.2 3.2) (drill 3.2) (layers "*.Cu" "*.Mask"))"#;
    let back = |s: String| s.replace("\"F.", "\"B.");
    let text = board_of(&[
        fp(
            "H1",
            "MountingHole:MountingHole_3.2mm",
            (10.0, 10.0),
            (3.2, 3.2),
            "(attr exclude_from_bom)",
            hole,
        ),
        back(resistor("R1", (10.0, 12.1))),
        back(resistor("R2", (12.4, 10.0))),
        back(test_point("TP1", (13.0, 7.6))),
        back(test_point("TP2", (14.4, 7.6))),
    ]);
    let st = FabSettings::default();
    let b = Board::read(&text, "t", &st).unwrap();
    let parts: Vec<_> = b.parts.iter().filter(|p| p.reference != "H1").collect();
    assert_eq!(parts.len(), 4);
    let (labels, stats) = pcba::place::place(&b, &parts, &st);
    assert_eq!(stats.collisions, 0);
    let audit = pcba::place::audit(&b, &parts, &labels, &st);
    assert_eq!(audit.on_symbol, 0, "{labels:#?}");
    assert_eq!(audit.label_overlaps, 0);
}

#[test]
fn leaders_in_a_dense_cluster_cross_no_label_and_no_leader() {
    let mut fps = Vec::new();
    for row in 0..4 {
        for col in 0..5 {
            let r = format!("R{}", row * 5 + col + 1);
            fps.push(resistor(
                &r,
                (15.0 + col as f64 * 1.6, 12.0 + row as f64 * 1.2),
            ));
        }
    }
    for k in 0..4 {
        fps.push(test_point(
            &format!("TP{}", k + 1),
            (14.0 + k as f64 * 2.4, 17.5),
        ));
    }
    let text = board_of(&fps);
    let (_, labels, audit) = placed(&text);
    assert_eq!(audit.label_overlaps, 0);
    assert_eq!(audit.on_body, 0);
    assert_eq!(audit.leader_crossings, 0, "{labels:#?}");
    // the same board gives the same page
    let (_, again, _) = placed(&text);
    let at = |ls: &[pcba::place::Label]| {
        ls.iter()
            .map(|l| (l.text.clone(), l.at))
            .collect::<Vec<_>>()
    };
    assert_eq!(at(&labels), at(&again));
}

#[test]
fn test_point_leaders_still_point_at_the_pad() {
    let (b, labels, _) = placed(&board_of(&[test_point("TP1", (20.0, 0.8))]));
    let l = labels.iter().find(|l| l.text == "TP1").unwrap();
    let (pad, _) = b.parts[0].probe.clone().unwrap();
    assert_eq!(l.leader.unwrap().1, pad);
}
