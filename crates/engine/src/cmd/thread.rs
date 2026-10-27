//! THREAD: ISO 6410 simplified screw threads — an external thread (a screw or bolt) and an
//! internal thread (a nut) drawn side by side from a diameter and a pitch.

use cadcraft_doc::{EntityKind, Lineweight};
use cadcraft_geom::{PolyVertex, Vec2};
use serde_json::{Value, json};

use super::helpers::{line, lwpoly, rect_vertices};
use super::*;
use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step};

/// The 60° inclusive thread flank: the half-angle's tangent, so a flank drops a radial depth `h`
/// over `h · TAN30` along the axis.
const TAN30: f64 = 0.5773502691896257;
const MAJOR_TO_PITCH: f64 = 0.64952;
const EXT_MINOR_TO_PITCH: f64 = 1.22687;
const INT_MINOR_TO_PITCH: f64 = 1.08253;
const EXT_DEPTH_TO_PITCH: f64 = 0.61343;
const INT_DEPTH_TO_PITCH: f64 = 0.54127;
const EXT_CREST_FLAT: f64 = 1.0 / 8.0;
const EXT_ROOT_FLAT: f64 = 1.0 / 6.0;
const INT_CREST_FLAT: f64 = 1.0 / 4.0;
const INT_ROOT_FLAT: f64 = 1.0 / 8.0;
/// Gap between the two threads when none is given.
const DEFAULT_SPACING: f64 = 1.5;
/// A thread is drawn over this many diameters of length.
const LENGTH_PER_DIAMETER: f64 = 2.0;
/// The nut's wall, as a multiple of the bore's radius, on each side: a nut is as tall as it is wide.
const NUT_WALLS: f64 = 1.0;
/// A size above this is refused rather than drawn.
const MAX_SIZE: f64 = 1e9;
/// The shallowest drawable depth, as a fraction of the diameter.
const MIN_DEPTH_RATIO: f64 = 1e-6;
/// The most threads a form may carry, so a tiny pitch can't spin the outline builder forever.
const MAX_THREADS: f64 = 10_000.0;
/// And a hard cap on the loop itself, in case of a still-smaller period.
const MAX_PERIODS: usize = 20_000;
/// Centre lines are drawn with the thinnest standard lineweight.
const THIN_LINE: u16 = 9;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("thread", "Thread", run_thread)
            .menu(&["Draw", "Thread"])
            .params("{at: [x,y], diameter (major, d), pitch (P), spacing? (gap between the two threads; default 1.5)}")
            .interactive(|_s| Ok(Box::new(ThreadM::default()))),
    ]
}

/// A thread profile: the pitch, minor and depth diameters.
struct Profile {
    pitch_diameter: f64,
    minor_diameter: f64,
    depth: f64,
}

/// ISO 68-1 external thread (a bolt): crest on the major, root on the minor. `h3 = 0.61343 P`.
fn external(d: f64, p: f64) -> Profile {
    Profile { pitch_diameter: d - MAJOR_TO_PITCH * p, minor_diameter: d - EXT_MINOR_TO_PITCH * p, depth: EXT_DEPTH_TO_PITCH * p }
}

/// ISO 68-1 internal thread (a nut): crest on the minor `D1`, root on the major. `H1 = 0.54127 P`.
fn internal(d: f64, p: f64) -> Profile {
    Profile { pitch_diameter: d - MAJOR_TO_PITCH * p, minor_diameter: d - INT_MINOR_TO_PITCH * p, depth: INT_DEPTH_TO_PITCH * p }
}

fn run_thread(s: &mut Session, p: &Value) -> Result<Value> {
    let at = point_req("thread", p, "at")?;
    let diameter = f64_req("thread", p, "diameter")?;
    if diameter <= 0.0 {
        return Err(bad("thread", "`diameter` must be positive"));
    }
    let pitch = f64_req("thread", p, "pitch")?;
    if pitch <= 0.0 {
        return Err(bad("thread", "`pitch` must be positive"));
    }
    let spacing = size_param("thread", p, "spacing", false)?.unwrap_or(DEFAULT_SPACING);
    if diameter > MAX_SIZE || pitch > MAX_SIZE || spacing > MAX_SIZE {
        return Err(bad("thread", "sizes are out of range"));
    }
    if let Some(msg) = pitch_range_error(diameter, pitch) {
        return Err(bad("thread", msg));
    }
    let mut handles = Vec::new();
    for (kind, thin) in pair(at, diameter, pitch, spacing) {
        let h = s.add_entity(kind)?;
        if thin {
            s.doc_mut()?.modify_entity(h, |e| e.common.lineweight = Lineweight::Mm100(THIN_LINE))?;
        }
        handles.push(h.hex());
    }
    let (ext, int) = (external(diameter, pitch), internal(diameter, pitch));
    let offset = diameter + spacing / 2.0;
    Ok(json!({
        "handles": handles,
        "external": {
            "majorDiameter": diameter,
            "pitchDiameter": ext.pitch_diameter,
            "minorDiameter": ext.minor_diameter,
            "depth": ext.depth,
            "center": [at.x - offset, at.y],
        },
        "internal": {
            "majorDiameter": diameter,
            "pitchDiameter": int.pitch_diameter,
            "minorDiameter": int.minor_diameter,
            "depth": int.depth,
            "center": [at.x + offset, at.y],
        },
    }))
}

/// The pair a thread of these sizes is drawn as, with `true` for a thin centre line.
fn pair(at: Vec2, diameter: f64, pitch: f64, spacing: f64) -> Vec<(EntityKind, bool)> {
    let half = LENGTH_PER_DIAMETER * diameter / 2.0;
    let (ext_cx, int_cx) = (at.x - (diameter + spacing / 2.0), at.x + (diameter + spacing / 2.0));

    let ext = external(diameter, pitch);
    let shaft = thread_form(
        Vec2::new(ext_cx - half, at.y),
        Vec2::new(ext_cx + half, at.y),
        diameter / 2.0,
        ext.minor_diameter / 2.0,
        EXT_CREST_FLAT,
        EXT_ROOT_FLAT,
        pitch,
    );

    // The nut is a plain rectangle around the bore, its wall one bore radius on each side.
    let nut_half = (diameter / 2.0) * (1.0 + NUT_WALLS);
    let nut = lwpoly(rect_vertices(Vec2::new(int_cx - half, at.y - nut_half), Vec2::new(int_cx + half, at.y + nut_half)), true);

    let int = internal(diameter, pitch);
    let bore = thread_form(
        Vec2::new(int_cx - half, at.y),
        Vec2::new(int_cx + half, at.y),
        int.minor_diameter / 2.0,
        diameter / 2.0,
        INT_CREST_FLAT,
        INT_ROOT_FLAT,
        pitch,
    );

    // A thin centre line through each thread, running on past it.
    let margin = diameter / 8.0;
    let ext_line = line(Vec2::new(ext_cx - half - margin, at.y), Vec2::new(ext_cx + half + margin, at.y));
    let int_line = line(Vec2::new(int_cx - half - margin, at.y), Vec2::new(int_cx + half + margin, at.y));

    vec![(shaft, false), (nut, false), (bore, false), (ext_line, true), (int_line, true)]
}

/// A closed form for a thread: a crest flat at `crest_r`, a `crest_frac · pitch` flat, a 60° flank
/// down to the root at `root_r`, a `root_frac · pitch` root flat, then the flank back up — repeated
/// along the length and mirrored about the axis into one closed polyline.
fn thread_form(left: Vec2, right: Vec2, crest_r: f64, root_r: f64, crest_frac: f64, root_frac: f64, pitch: f64) -> EntityKind {
    let (x0, x1, base) = (left.x, right.x, left.y);
    let flank = (crest_r - root_r).abs() * TAN30;
    let crest_w = crest_frac * pitch;
    let root_w = root_frac * pitch;
    let period = crest_w + 2.0 * flank + root_w;

    let mut crest: Vec<Vec2> = vec![Vec2::new(x0, base + crest_r)];
    let mut x = x0;
    let mut n = 0usize;
    while period > 0.0 && x + period <= x1 && n < MAX_PERIODS {
        crest.push(Vec2::new(x + crest_w, base + crest_r));
        crest.push(Vec2::new(x + crest_w + flank, base + root_r));
        crest.push(Vec2::new(x + crest_w + flank + root_w, base + root_r));
        crest.push(Vec2::new(x + period, base + crest_r));
        x += period;
        n += 1;
    }
    // Close the form at the right end on the crest, never cut mid-flank.
    if !crest.last().is_some_and(|p| (p.x - x1).abs() < 1e-9) {
        crest.push(Vec2::new(x1, base + crest_r));
    }

    let mut pts: Vec<Vec2> = Vec::with_capacity(2 * crest.len());
    pts.extend_from_slice(&crest);
    pts.extend(crest.iter().rev().map(|p| Vec2::new(p.x, base - (p.y - base))));
    lwpoly(pts.into_iter().map(PolyVertex::new).collect(), true)
}

/// Why a diameter/pitch pair can't be drawn, or `None` when it can.
fn pitch_range_error(diameter: f64, pitch: f64) -> Option<&'static str> {
    let ok = pitch * EXT_MINOR_TO_PITCH < diameter
        && EXT_DEPTH_TO_PITCH * pitch >= MIN_DEPTH_RATIO * diameter
        && LENGTH_PER_DIAMETER * diameter / pitch <= MAX_THREADS;
    (!ok).then_some("`pitch` is out of range")
}

/// THREAD at the command line: the diameter, the pitch, the gap between the pair, then its middle.
#[derive(Default)]
pub struct ThreadM {
    pub diameter: Option<f64>,
    pub pitch: Option<f64>,
    pub spacing: Option<f64>,
}

impl ThreadM {
    #[cfg(test)]
    pub fn preview(&self, _s: &Session, at: Vec2) -> Vec<EntityKind> {
        self.pair_preview(at)
    }

    fn pair_preview(&self, at: Vec2) -> Vec<EntityKind> {
        match (self.diameter, self.pitch, self.spacing) {
            (Some(d), Some(p), Some(sp))
                if d > 0.0 && p > 0.0 && sp > 0.0 && d <= MAX_SIZE && p <= MAX_SIZE && sp <= MAX_SIZE && pitch_range_error(d, p).is_none() =>
            {
                pair(at, d, p, sp).into_iter().map(|(kind, _)| kind).collect()
            }
            _ => Vec::new(),
        }
    }
}

/// A typed size, positive and within range, as the JSON form would accept it.
fn positive_size(t: &str) -> Option<f64> {
    machines::number(t).filter(|v| v.is_finite() && *v > 0.0 && *v <= MAX_SIZE)
}

impl Interactive for ThreadM {
    fn name(&self) -> &'static str {
        "THREAD"
    }

    fn prompt(&self, _s: &Session) -> Prompt {
        if self.diameter.is_none() {
            Prompt::new("Specify thread diameter", Accept::NUMBER)
        } else if self.pitch.is_none() {
            Prompt::new("Specify thread pitch", Accept::NUMBER)
        } else if self.spacing.is_none() {
            Prompt::new("Specify gap between the two threads", Accept::NUMBER).default(format!("{DEFAULT_SPACING}"))
        } else {
            Prompt::new("Specify middle point", Accept::POINT)
        }
    }

    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        if let Input::Cancel = i {
            return Ok(Step::Cancel);
        }
        if self.diameter.is_none() {
            match i {
                Input::Text(t) => match positive_size(&t) {
                    Some(v) => self.diameter = Some(v),
                    None => s.echo("Diameter must be a positive number."),
                },
                Input::Enter => return Ok(Step::Cancel),
                _ => {}
            }
            return Ok(Step::Continue);
        }
        if self.pitch.is_none() {
            match i {
                Input::Text(t) => match (positive_size(&t), self.diameter) {
                    (Some(v), Some(d)) if pitch_range_error(d, v).is_some() => s.echo("Pitch is out of range for the diameter."),
                    (Some(v), _) => self.pitch = Some(v),
                    (None, _) => s.echo("Pitch must be a positive number."),
                },
                Input::Enter => return Ok(Step::Cancel),
                _ => {}
            }
            return Ok(Step::Continue);
        }
        if self.spacing.is_none() {
            match i {
                Input::Text(t) => match positive_size(&t) {
                    Some(v) => self.spacing = Some(v),
                    None => s.echo("The gap must be a positive number."),
                },
                Input::Enter => self.spacing = Some(DEFAULT_SPACING),
                _ => {}
            }
            return Ok(Step::Continue);
        }
        match i {
            Input::Point(at) => {
                if let (Some(d), Some(p), Some(sp)) = (self.diameter, self.pitch, self.spacing) {
                    for (kind, thin) in pair(at, d, p, sp) {
                        let h = s.add_entity(kind)?;
                        if thin {
                            s.doc_mut()?.modify_entity(h, |e| e.common.lineweight = Lineweight::Mm100(THIN_LINE))?;
                        }
                    }
                    return Ok(Step::Done);
                }
                Ok(Step::Continue)
            }
            Input::Enter => Ok(Step::Cancel),
            _ => Ok(Step::Continue),
        }
    }

    fn preview(&self, _s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        self.pair_preview(cursor)
    }
}

#[cfg(test)]
#[path = "thread_tests.rs"]
mod tests;
