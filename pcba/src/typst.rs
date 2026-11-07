//! The Typst document that puts the pages and tables on paper.

use crate::settings::FabSettings;
use crate::svg::{DrillClass, drill_symbol};
use crate::tables::{BomRow, Tables};
use erc_model::layout::Side;
use std::fmt::Write;

fn s(t: &str) -> String {
    format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\""))
}

fn bom_table(out: &mut String, title: &str, rows: &[BomRow]) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "== {title}\n#table(columns: (auto, 1fr, auto, auto, auto), align: (right, left, left, left, left), inset: 3pt,"
    );
    out.push_str("  table.header[*Qty*][*References*][*Value*][*Footprint*][*MPN*],\n");
    for r in rows {
        let fp = r.footprint.split(':').next_back().unwrap_or(&r.footprint);
        let _ = writeln!(
            out,
            "  [{}], [#{}], [#{}], [#{}], [#{}],",
            r.qty,
            s(&r.references.join(", ")),
            s(&r.value),
            s(fp),
            s(r.mpn.as_deref().unwrap_or(""))
        );
    }
    out.push_str(")\n\n");
}

/// The document: one page per assembly side and the drill map, then the tables
pub fn document(
    board: &str,
    pages: &[(String, Side)],
    drill: Option<&str>,
    t: &Tables,
    classes: &[DrillClass],
    st: &FabSettings,
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// Assembly drawing of {board}, made by `pcba fab`; build with typst or `tpm doc build`."
    );
    out.push_str("#set page(paper: \"a4\", flipped: true, margin: 10mm)\n");
    out.push_str("#set text(font: (\"DejaVu Sans\",), size: 8pt)\n");
    out.push_str("#show heading: set text(size: 10pt)\n\n");
    for (file, side) in pages {
        let what = if *side == Side::Front {
            "Assembly, top"
        } else {
            "Assembly, bottom (seen from below)"
        };
        let _ = writeln!(
            out,
            "= {board}: {what}\n#align(center + horizon, image({}, width: 100%, height: 92%, fit: \"contain\"))\n#pagebreak()\n",
            s(file)
        );
    }
    if let Some(d) = drill {
        let _ = writeln!(
            out,
            "= {board}: Drill map, top\n#grid(columns: (1fr, auto), gutter: 6mm,\n  image({}, width: 100%, height: 88%, fit: \"contain\"),",
            s(d)
        );
        out.push_str("  table(columns: 5, inset: 3pt, align: (center, right, right, left, right),\n    table.header[*Symbol*][*Drill*][*Slot*][*Plating*][*Count*],\n");
        for c in classes {
            let r = 1.5;
            let sym = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="4mm" height="4mm" viewBox="-2 -2 4 4">{}</svg>"#,
                drill_symbol(c.symbol, 0.0, 0.0, r, st.thin_width * 3.0)
            );
            let _ = writeln!(
                out,
                "    image(bytes({})), [{:.2} mm], [{}], [{}], [{}],",
                s(&sym),
                c.diameter,
                c.slot.map(|x| format!("{x:.2} mm")).unwrap_or_default(),
                if c.plated { "plated" } else { "unplated" },
                c.count
            );
        }
        out.push_str("  ),\n)\n#pagebreak()\n\n");
    }
    bom_table(&mut out, "Fitted parts", &t.bom);
    bom_table(&mut out, "Not fitted (DNP)", &t.dnp);
    if !t.test_points.is_empty() {
        out.push_str("== Test points\n#table(columns: 5, inset: 3pt, align: (left, left, left, right, right),\n  table.header[*Ref*][*Net*][*Side*][*X mm*][*Y mm*],\n");
        for r in &t.test_points {
            let _ = writeln!(
                out,
                "  [{}], [#{}], [{}], [{:.3}], [{:.3}],",
                r.reference,
                s(r.net.as_deref().unwrap_or("")),
                r.side,
                r.x,
                r.y
            );
        }
        out.push_str(")\n");
    }
    out
}
