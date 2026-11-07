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

/// A straight piece of a line, mm
pub type Seg = ((f64, f64), (f64, f64));

/// The shape as straight pieces (arcs, circles and curves as short chords); closed shapes come back closed
pub fn segments(g: &Geom) -> Vec<Seg> {
    let chain = |ps: &[(f64, f64)]| ps.windows(2).map(|w| (w[0], w[1])).collect::<Vec<_>>();
    match g {
        Geom::Line(a, b) => vec![(pt(*a), pt(*b))],
        Geom::Rect(a, b) => {
            let (a, b) = (pt(*a), pt(*b));
            chain(&[a, (b.0, a.1), b, (a.0, b.1), a])
        }
        Geom::Circle { center, radius } => {
            let (c, r) = (pt(*center), mm(*radius));
            let ps: Vec<_> = (0..=48)
                .map(|i| {
                    let t = std::f64::consts::TAU * i as f64 / 48.0;
                    (c.0 + r * t.cos(), c.1 + r * t.sin())
                })
                .collect();
            chain(&ps)
        }
        Geom::Arc { start, mid, end } => chain(&arc_points(pt(*start), pt(*mid), pt(*end), 24)),
        Geom::Poly(ps) => {
            let mut ps: Vec<_> = ps.iter().map(|p| pt(*p)).collect();
            if let Some(&first) = ps.first() {
                ps.push(first);
            }
            chain(&ps)
        }
        Geom::Bezier(ps) => {
            let [p0, p1, p2, p3] = ps.map(pt);
            let at = |t: f64| {
                let u = 1.0 - t;
                let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                (
                    a * p0.0 + b * p1.0 + c * p2.0 + d * p3.0,
                    a * p0.1 + b * p1.1 + c * p2.1 + d * p3.1,
                )
            };
            chain(&(0..=24).map(|i| at(i as f64 / 24.0)).collect::<Vec<_>>())
        }
    }
}

/// Whether the segment touches the inside of the box
pub fn seg_hits_rect(s: Seg, r: &Rect) -> bool {
    if r.is_empty() {
        return false;
    }
    // Liang-Barsky clipping of the segment against the box
    let ((x0, y0), (x1, y1)) = s;
    let (dx, dy) = (x1 - x0, y1 - y0);
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [
        (-dx, x0 - r.x0),
        (dx, r.x1 - x0),
        (-dy, y0 - r.y0),
        (dy, r.y1 - y0),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// Whether two segments cross (touching at an end does not count)
pub fn segs_cross(a: Seg, b: Seg) -> bool {
    let orient = |p: (f64, f64), q: (f64, f64), r: (f64, f64)| {
        (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0)
    };
    let (d1, d2) = (orient(b.0, b.1, a.0), orient(b.0, b.1, a.1));
    let (d3, d4) = (orient(a.0, a.1, b.0), orient(a.0, a.1, b.1));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

/// Whether a circle comes closer than its radius to the box
pub fn circle_hits_rect(c: (f64, f64), r: f64, b: &Rect) -> bool {
    let n = b.nearest(c);
    (n.0 - c.0).powi(2) + (n.1 - c.1).powi(2) < r * r
}

/// Whether `p` is inside the closed outline made of `segs` (even-odd rule, so cut-outs count as outside)
pub fn inside(segs: &[Seg], p: (f64, f64)) -> bool {
    let mut odd = false;
    for &((x0, y0), (x1, y1)) in segs {
        if (y0 > p.1) != (y1 > p.1) {
            let x = x0 + (p.1 - y0) / (y1 - y0) * (x1 - x0);
            if x > p.0 {
                odd = !odd;
            }
        }
    }
    odd
}
