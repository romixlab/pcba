//! Millimetre geometry: boxes, shape bounds, overlap tests.

use erc_model::layout::{Geom, Pt};

/// Nanometres to millimetres
pub fn mm(nm: i64) -> f64 {
    nm as f64 / 1e6
}

pub fn pt(p: Pt) -> (f64, f64) {
    (mm(p.x), mm(p.y))
}

/// An axis-aligned box, mm; empty when x0 > x1
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Rect {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
        Rect {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }
    pub fn empty() -> Rect {
        Rect {
            x0: f64::INFINITY,
            y0: f64::INFINITY,
            x1: f64::NEG_INFINITY,
            y1: f64::NEG_INFINITY,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.x0 > self.x1 || self.y0 > self.y1
    }
    pub fn around(c: (f64, f64), w: f64, h: f64) -> Rect {
        Rect::new(c.0 - w / 2.0, c.1 - h / 2.0, c.0 + w / 2.0, c.1 + h / 2.0)
    }
    pub fn union(&self, o: &Rect) -> Rect {
        if self.is_empty() {
            return *o;
        }
        if o.is_empty() {
            return *self;
        }
        Rect {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }
    pub fn add(&self, p: (f64, f64)) -> Rect {
        self.union(&Rect {
            x0: p.0,
            y0: p.1,
            x1: p.0,
            y1: p.1,
        })
    }
    pub fn grow(&self, d: f64) -> Rect {
        Rect {
            x0: self.x0 - d,
            y0: self.y0 - d,
            x1: self.x1 + d,
            y1: self.y1 + d,
        }
    }
    pub fn w(&self) -> f64 {
        self.x1 - self.x0
    }
    pub fn h(&self) -> f64 {
        self.y1 - self.y0
    }
    pub fn center(&self) -> (f64, f64) {
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }
    pub fn overlaps(&self, o: &Rect) -> bool {
        !(self.is_empty() || o.is_empty())
            && self.x0 < o.x1
            && o.x0 < self.x1
            && self.y0 < o.y1
            && o.y0 < self.y1
    }
    /// The point of the box nearest to `p`
    pub fn nearest(&self, p: (f64, f64)) -> (f64, f64) {
        (p.0.clamp(self.x0, self.x1), p.1.clamp(self.y0, self.y1))
    }
}

/// Bounds of a shape's geometry, mm (an arc by its three points and the circle through them)
pub fn shape_bounds(g: &Geom) -> Rect {
    let mut r = Rect::empty();
    match g {
        Geom::Line(a, b) | Geom::Rect(a, b) => r = r.add(pt(*a)).add(pt(*b)),
        Geom::Circle { center, radius } => {
            let (c, rad) = (pt(*center), mm(*radius));
            r = Rect::new(c.0 - rad, c.1 - rad, c.0 + rad, c.1 + rad);
        }
        Geom::Arc { start, mid, end } => {
            for p in arc_points(pt(*start), pt(*mid), pt(*end), 24) {
                r = r.add(p);
            }
        }
        Geom::Poly(ps) => {
            for p in ps {
                r = r.add(pt(*p));
            }
        }
        Geom::Bezier(ps) => {
            for p in ps {
                r = r.add(pt(*p));
            }
        }
    }
    r
}

/// Points along the arc through `a`, `m`, `b` (a straight line when they are collinear)
pub fn arc_points(a: (f64, f64), m: (f64, f64), b: (f64, f64), n: usize) -> Vec<(f64, f64)> {
    let d = 2.0 * (a.0 * (m.1 - b.1) + m.0 * (b.1 - a.1) + b.0 * (a.1 - m.1));
    if d.abs() < 1e-12 {
        return vec![a, b];
    }
    let sq = |p: (f64, f64)| p.0 * p.0 + p.1 * p.1;
    let cx = (sq(a) * (m.1 - b.1) + sq(m) * (b.1 - a.1) + sq(b) * (a.1 - m.1)) / d;
    let cy = (sq(a) * (b.0 - m.0) + sq(m) * (a.0 - b.0) + sq(b) * (m.0 - a.0)) / d;
    let r = ((a.0 - cx).powi(2) + (a.1 - cy).powi(2)).sqrt();
    let ang = |p: (f64, f64)| (p.1 - cy).atan2(p.0 - cx);
    let (t0, tm, mut t1) = (ang(a), ang(m), ang(b));
    // go from a to b through m
    let tau = std::f64::consts::TAU;
    let norm = |t: f64| ((t % tau) + tau) % tau;
    let ccw_m = norm(tm - t0) < norm(t1 - t0);
    if ccw_m {
        t1 = t0 + norm(t1 - t0);
    } else {
        t1 = t0 - norm(t0 - t1);
    }
    (0..=n)
        .map(|i| {
            let t = t0 + (t1 - t0) * i as f64 / n as f64;
            (cx + r * t.cos(), cy + r * t.sin())
        })
        .collect()
}
