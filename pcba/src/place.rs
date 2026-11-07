//! Where each reference goes: inside its body at the largest size that fits, else outside with a leader at the
//! nearest spot that collides with no body and no other label.

use crate::board::Part;
use crate::geom::Rect;
use crate::settings::FabSettings;

/// A reference on the page
#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    /// Centre of the text, mm
    pub at: (f64, f64),
    /// Text height, mm
    pub size: f64,
    /// Turned 90° to run along a tall body
    pub vertical: bool,
    /// Leader from the label's box to the part, when the label is outside
    pub leader: Option<((f64, f64), (f64, f64))>,
    pub dnp: bool,
    /// Box the text takes, mm
    pub bbox: Rect,
}

/// The largest text height for `n` characters in a `w` x `h` box, horizontal and turned
fn fit(n: usize, w: f64, h: f64, s: &FabSettings) -> (f64, bool) {
    let n = n.max(1) as f64;
    let flat = (w * s.text_fill / (n * s.char_width))
        .min(h * s.text_fill)
        .min(s.max_text);
    let turned = (h * s.text_fill / (n * s.char_width))
        .min(w * s.text_fill)
        .min(s.max_text);
    if turned > flat * 1.15 {
        (turned, true)
    } else {
        (flat, false)
    }
}

fn text_box(at: (f64, f64), n: usize, size: f64, vertical: bool, s: &FabSettings) -> Rect {
    let (w, h) = (n as f64 * s.char_width * size, size);
    if vertical {
        Rect::around(at, h, w)
    } else {
        Rect::around(at, w, h)
    }
}

/// Labels for the parts of one side; `stats` counts inside and outside placements and collisions left
pub fn place(parts: &[&Part], s: &FabSettings) -> (Vec<Label>, PlaceStats) {
    let mut labels = Vec::new();
    let mut stats = PlaceStats::default();
    let bodies: Vec<Rect> = parts.iter().map(|p| p.bbox).collect();
    let mut outside = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        let n = p.reference.chars().count();
        let (size, vertical) = fit(n, p.bbox.w(), p.bbox.h(), s);
        if size >= s.min_text && p.kind != crate::board::PartKind::TestPoint {
            let at = p.bbox.center();
            labels.push(Label {
                text: p.reference.clone(),
                at,
                size,
                vertical,
                leader: None,
                dnp: p.dnp,
                bbox: text_box(at, n, size, vertical, s),
            });
            stats.inside += 1;
        } else {
            outside.push(i);
        }
    }
    // smallest parts first: they have the fewest good spots
    outside.sort_by(|&a, &b| {
        let area = |i: usize| parts[i].bbox.w() * parts[i].bbox.h();
        area(a).total_cmp(&area(b))
    });
    let mut taken: Vec<Rect> = labels.iter().map(|l| l.bbox).collect();
    for i in outside {
        let p = parts[i];
        let n = p.reference.chars().count();
        let size = s.callout_text;
        let (tw, th) = (n as f64 * s.char_width * size, size);
        let c = p.bbox.center();
        let mut best = None;
        'search: for step in 0.. {
            let d = s.callout_gap + step as f64 * size * 0.5;
            if d > s.callout_reach {
                break;
            }
            for k in 0..16 {
                let a = std::f64::consts::TAU * k as f64 / 16.0 - std::f64::consts::FRAC_PI_2;
                let (dx, dy) = (a.cos(), a.sin());
                // just outside the part's box in this direction, then `d` further
                let rx = p.bbox.w() / 2.0 + tw / 2.0;
                let ry = p.bbox.h() / 2.0 + th / 2.0;
                let reach = if dx.abs() * ry > dy.abs() * rx {
                    rx / dx.abs()
                } else {
                    ry / dy.abs()
                };
                let at = (c.0 + dx * (reach + d), c.1 + dy * (reach + d));
                let b = Rect::around(at, tw, th);
                let clear = b.grow(s.callout_gap * 0.5);
                if bodies.iter().any(|o| o.overlaps(&clear))
                    || taken.iter().any(|o| o.overlaps(&clear))
                {
                    continue;
                }
                best = Some((at, b));
                break 'search;
            }
        }
        let (at, b, collided) = match best {
            Some((at, b)) => (at, b, false),
            None => {
                let at = (c.0, p.bbox.y0 - s.callout_gap - th / 2.0);
                (at, Rect::around(at, tw, th), true)
            }
        };
        stats.outside += 1;
        stats.collisions += collided as usize;
        taken.push(b);
        let to = if p.kind == crate::board::PartKind::TestPoint {
            p.probe.as_ref().map(|(at, _)| *at).unwrap_or(c)
        } else {
            p.bbox.nearest(b.center())
        };
        labels.push(Label {
            text: p.reference.clone(),
            at,
            size,
            vertical: false,
            leader: Some((b.nearest(to), to)),
            dnp: p.dnp,
            bbox: b,
        });
    }
    (labels, stats)
}

/// How the references were placed on a side
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlaceStats {
    pub inside: usize,
    pub outside: usize,
    /// References that found no free spot within reach and overlap something
    pub collisions: usize,
}
