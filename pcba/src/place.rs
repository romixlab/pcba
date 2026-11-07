//! Where each reference goes: inside its body at the largest size that fits, else outside with a leader.
//!
//! A reference outside goes to the nearest spot on rings around its part that overlaps no body, no other label,
//! no hole, test point or fiducial symbol and does not straddle the board outline. Among those, a spot whose
//! leader crosses no other label, leader or part body and whose text sits on no leader wins over a nearer one
//! that does (when every spot has some: the fewest label and leader crossings, then the fewest bodies); at
//! the same distance a spot inside the board wins over one outside it.

use crate::board::{Board, Part, PartKind};
use crate::geom::{Rect, Seg, circle_hits_rect, inside, seg_hits_rect, segments, segs_cross};
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
    pub leader: Option<Seg>,
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

/// What a reference placed outside must keep clear of besides bodies and other labels
#[derive(Clone, Debug, Default)]
pub struct Obstacles {
    /// The board outline as straight pieces
    pub outline: Vec<Seg>,
    /// Holes, test point and fiducial symbols: centre and radius, mm
    pub circles: Vec<((f64, f64), f64)>,
}

impl Obstacles {
    /// The outline and holes of the board and the symbols of the parts on one side
    pub fn new(board: &Board, parts: &[&Part], s: &FabSettings) -> Obstacles {
        let outline = board
            .outline
            .iter()
            .flat_map(|sh| segments(&sh.geom))
            .collect();
        let mut circles: Vec<_> = board
            .holes
            .iter()
            .map(|h| (h.at, h.slot.unwrap_or(h.diameter).max(h.diameter) / 2.0))
            .collect();
        for p in parts {
            match p.kind {
                PartKind::TestPoint => {
                    let c = p.probe.as_ref().map(|(at, _)| *at).unwrap_or(p.at);
                    circles.push((c, s.testpoint_radius));
                }
                PartKind::Fiducial => {
                    circles.push((p.bbox.center(), p.bbox.w().max(p.bbox.h()) / 2.0));
                }
                _ => {}
            }
        }
        Obstacles { outline, circles }
    }

    /// Whether a point is on the board (always, when the board has no outline)
    pub fn on_board(&self, p: (f64, f64)) -> bool {
        self.outline.is_empty() || inside(&self.outline, p)
    }

    /// Whether a text box straddles the outline line, with `gap` to spare
    pub fn on_edge(&self, b: &Rect, s: &FabSettings, gap: f64) -> bool {
        let r = b.grow(s.outline_width / 2.0 + gap);
        self.outline.iter().any(|&g| seg_hits_rect(g, &r))
    }

    /// Whether a text box covers a hole or a symbol, with `gap` to spare
    pub fn on_symbol(&self, b: &Rect, gap: f64) -> bool {
        self.circles
            .iter()
            .any(|&(c, r)| circle_hits_rect(c, r + gap, b))
    }
}

/// How often a leader with its text in `b` meets the labels and leaders placed so far, and how many bodies of
/// other parts it runs through (`own` is the index of the leader's part in `bodies`)
fn crossings(
    b: &Rect,
    leader: Seg,
    labels: &[Label],
    bodies: &[Rect],
    own: usize,
) -> (usize, usize) {
    let crossed = labels
        .iter()
        .map(|o| {
            let mut n = seg_hits_rect(leader, &o.bbox) as usize;
            if let Some(l) = o.leader {
                n += segs_cross(leader, l) as usize + seg_hits_rect(l, b) as usize;
            }
            n
        })
        .sum();
    let through = bodies
        .iter()
        .enumerate()
        .filter(|&(j, r)| j != own && seg_hits_rect(leader, r))
        .count();
    (crossed, through)
}

fn leader_of(p: &Part, b: &Rect) -> Seg {
    let to = if p.kind == PartKind::TestPoint {
        p.probe
            .as_ref()
            .map(|(at, _)| *at)
            .unwrap_or(p.bbox.center())
    } else {
        p.bbox.nearest(b.center())
    };
    (b.nearest(to), to)
}

/// Labels for the parts of one side of `board`; `stats` counts inside and outside placements and what is left
/// overlapping or crossing
pub fn place(board: &Board, parts: &[&Part], s: &FabSettings) -> (Vec<Label>, PlaceStats) {
    let obstacles = Obstacles::new(board, parts, s);
    let mut labels = Vec::new();
    let mut stats = PlaceStats::default();
    let mut outside = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        let n = p.reference.chars().count();
        let (size, vertical) = fit(n, p.bbox.w(), p.bbox.h(), s);
        if size >= s.min_text && p.kind != PartKind::TestPoint {
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
    // a second pass places first the references that crossed something; it is kept when it leaves less
    let mut done = callouts(parts, &outside, labels.clone(), &obstacles, s);
    if !done.crossed.is_empty() || !done.collided.is_empty() {
        let mut order: Vec<usize> = done.collided.clone();
        order.extend(&done.crossed);
        let rest: Vec<usize> = outside
            .iter()
            .copied()
            .filter(|i| !order.contains(i))
            .collect();
        order.extend(rest);
        let second = callouts(parts, &order, labels, &obstacles, s);
        if second.score() < done.score() {
            done = second;
        }
    }
    stats.outside = outside.len();
    stats.collisions = done.collided.len();
    stats.crossings = done.crossed.len();
    (done.labels, stats)
}

/// The result of one pass of placing references outside their parts
struct Pass {
    labels: Vec<Label>,
    /// Parts whose reference found no free spot
    collided: Vec<usize>,
    /// Parts whose reference crosses something
    crossed: Vec<usize>,
    /// Crossings in all
    cost: usize,
}

impl Pass {
    fn score(&self) -> (usize, usize, usize) {
        (self.collided.len(), self.crossed.len(), self.cost)
    }
}

/// Places the references of `parts[order]` outside their parts, in that order, around the labels in `labels`
fn callouts(
    parts: &[&Part],
    order: &[usize],
    labels: Vec<Label>,
    obstacles: &Obstacles,
    s: &FabSettings,
) -> Pass {
    let bodies: Vec<Rect> = parts.iter().map(|p| p.bbox).collect();
    let mut pass = Pass {
        labels,
        collided: Vec::new(),
        crossed: Vec::new(),
        cost: 0,
    };
    let labels = &mut pass.labels;
    for &i in order {
        let p = parts[i];
        let n = p.reference.chars().count();
        let size = s.callout_text;
        let (tw, th) = (n as f64 * s.char_width * size, size);
        let c = p.bbox.center();
        // (label and leader crossings, bodies crossed, ring, off the board, direction): the smallest wins
        type Key = (usize, usize, usize, bool, usize);
        let mut best: Option<(Key, (f64, f64), Rect)> = None;
        for step in 0.. {
            let d = s.callout_gap + step as f64 * size * 0.5;
            if d > s.callout_reach || best.as_ref().is_some_and(|(k, ..)| k.0 + k.1 == 0) {
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
                    || labels.iter().any(|o| o.bbox.overlaps(&clear))
                    || obstacles.on_edge(&b, s, s.edge_gap)
                    || obstacles.on_symbol(&b, s.edge_gap)
                {
                    continue;
                }
                let (crossed, through) = crossings(&b, leader_of(p, &b), labels, &bodies, i);
                let key = (crossed, through, step, !obstacles.on_board(at), k);
                if best.as_ref().is_none_or(|(bk, ..)| key < *bk) {
                    best = Some((key, at, b));
                }
            }
        }
        let (at, b) = match best {
            Some(((crossed, through, ..), at, b)) => {
                if crossed + through > 0 {
                    pass.crossed.push(i);
                    pass.cost += crossed + through;
                }
                (at, b)
            }
            None => {
                pass.collided.push(i);
                let at = (c.0, p.bbox.y0 - s.callout_gap - th / 2.0);
                (at, Rect::around(at, tw, th))
            }
        };
        labels.push(Label {
            text: p.reference.clone(),
            at,
            size,
            vertical: false,
            leader: Some(leader_of(p, &b)),
            dnp: p.dnp,
            bbox: b,
        });
    }
    pass
}

/// How the references were placed on a side
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlaceStats {
    pub inside: usize,
    pub outside: usize,
    /// References that found no free spot within reach and overlap something
    pub collisions: usize,
    /// References outside whose leader or text crosses another label, leader or part body (no crossing-free spot
    /// in reach)
    pub crossings: usize,
}

/// What is wrong with a finished page: each count is a pair or a label
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Audit {
    /// Pairs of labels whose boxes overlap
    pub label_overlaps: usize,
    /// Labels whose box touches the board outline line
    pub on_edge: usize,
    /// Labels outside their part whose box covers a hole, test point or fiducial symbol
    pub on_symbol: usize,
    /// Labels outside their part whose box overlaps a part body
    pub on_body: usize,
    /// Leaders that cross another label's box, and pairs of leaders that cross
    pub leader_crossings: usize,
    /// Leaders that cross the body of a part other than their own
    pub leader_through_body: usize,
}

/// Checks placed labels against each other and the board: what a reader would see as clutter
pub fn audit(board: &Board, parts: &[&Part], labels: &[Label], s: &FabSettings) -> Audit {
    let obstacles = Obstacles::new(board, parts, s);
    let mut a = Audit::default();
    for (i, l) in labels.iter().enumerate() {
        for o in &labels[i + 1..] {
            a.label_overlaps += l.bbox.overlaps(&o.bbox) as usize;
            if let (Some(x), Some(y)) = (l.leader, o.leader) {
                a.leader_crossings += segs_cross(x, y) as usize;
            }
        }
        a.on_edge += obstacles.on_edge(&l.bbox, s, 0.0) as usize;
        if let Some(leader) = l.leader {
            a.on_symbol += obstacles.on_symbol(&l.bbox, 0.0) as usize;
            a.on_body += parts.iter().any(|p| p.bbox.overlaps(&l.bbox)) as usize;
            // the part a leader points at is the one whose box holds its far end
            a.leader_through_body += parts
                .iter()
                .filter(|p| {
                    let r = p.bbox.grow(1e-9);
                    !(r.x0..=r.x1).contains(&leader.1.0) || !(r.y0..=r.y1).contains(&leader.1.1)
                })
                .any(|p| seg_hits_rect(leader, &p.bbox))
                as usize;
            a.leader_crossings += labels
                .iter()
                .enumerate()
                .filter(|&(j, o)| j != i && seg_hits_rect(leader, &o.bbox))
                .count();
        }
    }
    a
}
