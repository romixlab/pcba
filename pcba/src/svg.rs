//! SVG pages: an assembly view per side (the back mirrored, as seen from below) and a drill map.

use crate::board::{Board, Hole, Part, PartKind};
use crate::geom::{Rect, arc_points, pt};
use crate::place::Label;
use crate::settings::FabSettings;
use erc_model::layout::{Geom, Shape, Side};
use std::fmt::Write;

const INK: &str = "#1a1a1a";
const BODY: &str = "#555555";
const DNP: &str = "#9a9a9a";
const PROBE: &str = "#1f5fbf";
const FID: &str = "#b5651d";

/// Coordinates of one page: the back side is mirrored left to right
#[derive(Clone, Copy)]
struct View {
    mirror: bool,
}

impl View {
    fn p(&self, p: (f64, f64)) -> (f64, f64) {
        if self.mirror { (-p.0, p.1) } else { p }
    }
    fn rect(&self, r: &Rect) -> Rect {
        let (a, b) = (self.p((r.x0, r.y0)), self.p((r.x1, r.y1)));
        Rect::new(a.0, a.1, b.0, b.1)
    }
}

fn f(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

fn esc(t: &str) -> String {
    t.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn path_d(g: &Geom, v: View) -> String {
    let mut d = String::new();
    let mv = |d: &mut String, p: (f64, f64), first: bool| {
        let p = v.p(p);
        let _ = write!(d, "{}{} {} ", if first { "M" } else { "L" }, f(p.0), f(p.1));
    };
    match g {
        Geom::Line(a, b) => {
            mv(&mut d, pt(*a), true);
            mv(&mut d, pt(*b), false);
        }
        Geom::Rect(a, b) => {
            let (a, b) = (pt(*a), pt(*b));
            for (i, p) in [a, (b.0, a.1), b, (a.0, b.1)].into_iter().enumerate() {
                mv(&mut d, p, i == 0);
            }
            d.push('Z');
        }
        Geom::Circle { center, radius } => {
            let (c, r) = (v.p(pt(*center)), *radius as f64 / 1e6);
            let _ = write!(
                d,
                "M{} {} a{r} {r} 0 1 0 {} 0 a{r} {r} 0 1 0 {} 0Z",
                f(c.0 - r),
                f(c.1),
                f(2.0 * r),
                f(-2.0 * r),
                r = f(r)
            );
        }
        Geom::Arc { start, mid, end } => {
            for (i, p) in arc_points(pt(*start), pt(*mid), pt(*end), 24)
                .into_iter()
                .enumerate()
            {
                mv(&mut d, p, i == 0);
            }
        }
        Geom::Poly(ps) => {
            for (i, p) in ps.iter().enumerate() {
                mv(&mut d, pt(*p), i == 0);
            }
            d.push('Z');
        }
        Geom::Bezier(ps) => {
            let q: Vec<(f64, f64)> = ps.iter().map(|p| v.p(pt(*p))).collect();
            let _ = write!(
                d,
                "M{} {} C{} {} {} {} {} {}",
                f(q[0].0),
                f(q[0].1),
                f(q[1].0),
                f(q[1].1),
                f(q[2].0),
                f(q[2].1),
                f(q[3].0),
                f(q[3].1)
            );
        }
    }
    d.trim_end().to_string()
}

fn shapes(out: &mut String, shapes: &[Shape], v: View, stroke: &str, width: f64, dash: bool) {
    for s in shapes {
        let fill = if s.filled { stroke } else { "none" };
        let _ = writeln!(
            out,
            r#"<path d="{}" fill="{fill}" stroke="{stroke}" stroke-width="{}"{} stroke-linecap="round" stroke-linejoin="round"/>"#,
            path_d(&s.geom, v),
            f(width),
            if dash {
                r#" stroke-dasharray="0.4 0.3""#
            } else {
                ""
            }
        );
    }
}

fn header(out: &mut String, area: &Rect, title: &str) {
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}mm" height="{h}mm" viewBox="{x} {y} {w} {h}">"#,
        x = f(area.x0),
        y = f(area.y0),
        w = f(area.w()),
        h = f(area.h())
    );
    let _ = writeln!(out, "<title>{}</title>", esc(title));
    let _ = writeln!(
        out,
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="white"/>"#,
        f(area.x0),
        f(area.y0),
        f(area.w()),
        f(area.h())
    );
}

fn text(out: &mut String, l: &Label, v: View, color: &str) {
    let (x, y) = v.p(l.at);
    let rot = if l.vertical {
        format!(r#" transform="rotate(-90 {} {})""#, f(x), f(y))
    } else {
        String::new()
    };
    let _ = writeln!(
        out,
        r#"<text x="{}" y="{}" font-family="DejaVu Sans Mono, monospace" font-size="{}" fill="{color}" text-anchor="middle" dominant-baseline="central"{rot}>{}</text>"#,
        f(x),
        f(y),
        f(l.size),
        esc(&l.text)
    );
}

/// The assembly page of one side
pub fn assembly(
    board: &Board,
    side: Side,
    parts: &[&Part],
    labels: &[Label],
    s: &FabSettings,
) -> String {
    let v = View {
        mirror: side == Side::Back,
    };
    let mut area = v.rect(&board.bbox);
    for l in labels {
        area = area.union(&v.rect(&l.bbox));
    }
    let area = area.grow(s.margin);
    let mut out = String::new();
    let side_name = if side == Side::Front {
        "top"
    } else {
        "bottom, seen from below"
    };
    header(
        &mut out,
        &area,
        &format!("{} assembly, {side_name}", board.name),
    );
    shapes(&mut out, &board.outline, v, INK, s.outline_width, false);
    for p in parts {
        match p.kind {
            PartKind::Component | PartKind::MountingHole => {
                let (color, dash) = if p.dnp { (DNP, true) } else { (BODY, false) };
                if p.body.is_empty() {
                    let r = v.rect(&p.bbox);
                    let _ = writeln!(
                        out,
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{color}" stroke-width="{}"/>"#,
                        f(r.x0),
                        f(r.y0),
                        f(r.w()),
                        f(r.h()),
                        f(s.body_width)
                    );
                } else {
                    shapes(&mut out, &p.body, v, color, s.body_width, dash);
                }
                if p.dnp {
                    let r = v.rect(&p.bbox);
                    let _ = writeln!(
                        out,
                        r#"<path d="M{} {} L{} {} M{} {} L{} {}" stroke="{DNP}" stroke-width="{}"/>"#,
                        f(r.x0),
                        f(r.y0),
                        f(r.x1),
                        f(r.y1),
                        f(r.x0),
                        f(r.y1),
                        f(r.x1),
                        f(r.y0),
                        f(s.thin_width)
                    );
                }
            }
            PartKind::TestPoint => {
                let (c, _) = p.probe.clone().unwrap_or((p.at, None));
                let (x, y) = v.p(c);
                let r = s.testpoint_radius;
                let _ = writeln!(
                    out,
                    r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="{PROBE}" stroke-width="{}"/><path d="M{} {} L{} {} M{} {} L{} {}" stroke="{PROBE}" stroke-width="{}"/>"#,
                    f(x),
                    f(y),
                    f(r),
                    f(s.thin_width * 2.0),
                    f(x - r),
                    f(y),
                    f(x + r),
                    f(y),
                    f(x),
                    f(y - r),
                    f(x),
                    f(y + r),
                    f(s.thin_width)
                );
            }
            PartKind::Fiducial => {
                let (x, y) = v.p(p.bbox.center());
                let r = p.bbox.w().max(p.bbox.h()) / 2.0;
                let _ = writeln!(
                    out,
                    r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="{FID}" stroke-width="{}"/><circle cx="{}" cy="{}" r="{}" fill="{FID}"/>"#,
                    f(x),
                    f(y),
                    f(r),
                    f(s.thin_width * 2.0),
                    f(x),
                    f(y),
                    f(r * 0.35)
                );
            }
        }
    }
    for l in labels {
        if let Some((a, b)) = l.leader {
            let (a, b) = (v.p(a), v.p(b));
            let _ = writeln!(
                out,
                r#"<path d="M{} {} L{} {}" stroke="{BODY}" stroke-width="{}"/><circle cx="{}" cy="{}" r="{}" fill="{BODY}"/>"#,
                f(a.0),
                f(a.1),
                f(b.0),
                f(b.1),
                f(s.thin_width),
                f(b.0),
                f(b.1),
                f(s.thin_width * 2.0)
            );
        }
        text(&mut out, l, v, if l.dnp { DNP } else { INK });
    }
    out.push_str("</svg>\n");
    out
}

/// A drill class: one symbol per plating, diameter and slot length
#[derive(Clone, Debug, PartialEq)]
pub struct DrillClass {
    pub plated: bool,
    pub diameter: f64,
    pub slot: Option<f64>,
    pub count: usize,
    /// Symbol index (0: circle, 1: square, 2: triangle, 3: diamond, 4: plus, 5: cross, then combined)
    pub symbol: usize,
}

pub fn drill_classes(holes: &[Hole]) -> Vec<DrillClass> {
    let mut classes: Vec<DrillClass> = Vec::new();
    for h in holes {
        let key = |c: &DrillClass| {
            c.plated == h.plated
                && (c.diameter - h.diameter).abs() < 1e-4
                && c.slot.map(|x| (x * 1e4).round()) == h.slot.map(|x| (x * 1e4).round())
        };
        match classes.iter_mut().find(|c| key(c)) {
            Some(c) => c.count += 1,
            None => classes.push(DrillClass {
                plated: h.plated,
                diameter: h.diameter,
                slot: h.slot,
                count: 1,
                symbol: 0,
            }),
        }
    }
    classes.sort_by(|a, b| {
        (!a.plated, a.diameter, a.slot.unwrap_or(0.0))
            .partial_cmp(&(!b.plated, b.diameter, b.slot.unwrap_or(0.0)))
            .unwrap()
    });
    for (i, c) in classes.iter_mut().enumerate() {
        c.symbol = i;
    }
    classes
}

/// SVG fragment of drill symbol `i` centred at `(x, y)` (a size `r` half-width)
pub fn drill_symbol(i: usize, x: f64, y: f64, r: f64, w: f64) -> String {
    let base = i % 6;
    let ring = i / 6;
    let st = format!(r#"fill="none" stroke="{INK}" stroke-width="{}""#, f(w));
    let mut s = match base {
        0 => format!(r#"<circle cx="{}" cy="{}" r="{}" {st}/>"#, f(x), f(y), f(r)),
        1 => format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" {st}/>"#,
            f(x - r),
            f(y - r),
            f(2.0 * r),
            f(2.0 * r)
        ),
        2 => format!(
            r#"<path d="M{} {} L{} {} L{} {}Z" {st}/>"#,
            f(x),
            f(y - r),
            f(x + r),
            f(y + r * 0.8),
            f(x - r),
            f(y + r * 0.8)
        ),
        3 => format!(
            r#"<path d="M{} {} L{} {} L{} {} L{} {}Z" {st}/>"#,
            f(x),
            f(y - r),
            f(x + r),
            f(y),
            f(x),
            f(y + r),
            f(x - r),
            f(y)
        ),
        4 => format!(
            r#"<path d="M{} {} L{} {} M{} {} L{} {}" {st}/>"#,
            f(x - r),
            f(y),
            f(x + r),
            f(y),
            f(x),
            f(y - r),
            f(x),
            f(y + r)
        ),
        _ => format!(
            r#"<path d="M{} {} L{} {} M{} {} L{} {}" {st}/>"#,
            f(x - r),
            f(y - r),
            f(x + r),
            f(y + r),
            f(x - r),
            f(y + r),
            f(x + r),
            f(y - r)
        ),
    };
    for k in 0..ring {
        let rr = r * (1.35 + 0.35 * k as f64);
        let _ = write!(
            s,
            r#"<circle cx="{}" cy="{}" r="{}" {st}/>"#,
            f(x),
            f(y),
            f(rr)
        );
    }
    s
}

/// The drill map: outline and a symbol per hole, seen from the top
pub fn drill(board: &Board, classes: &[DrillClass], s: &FabSettings) -> String {
    let v = View { mirror: false };
    let area = board.bbox.grow(s.margin);
    let mut out = String::new();
    header(&mut out, &area, &format!("{} drill map, top", board.name));
    shapes(&mut out, &board.outline, v, INK, s.outline_width, false);
    let r = s.drill_symbol / 2.0;
    for h in &board.holes {
        let c = classes
            .iter()
            .find(|c| {
                c.plated == h.plated
                    && (c.diameter - h.diameter).abs() < 1e-4
                    && c.slot.map(|x| (x * 1e4).round()) == h.slot.map(|x| (x * 1e4).round())
            })
            .expect("every hole has its class");
        out.push_str(&drill_symbol(
            c.symbol,
            h.at.0,
            h.at.1,
            r,
            s.thin_width * 2.0,
        ));
        out.push('\n');
    }
    out.push_str("</svg>\n");
    out
}
