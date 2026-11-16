//! PARTNO (Util ▸ Part Number): a part-number balloon — an arrow-tipped leader from the first
//! point to a numbered circle at the second. The number grows with every balloon placed in the
//! session, and a whole number typed at either prompt is the next balloon's number.

use cadcraft_doc::{HAlign, VAlign};

use super::common::style;
use super::leader::sizes;
use super::{Accept, EntityKind, Handle, Input, Interactive, Prompt, Result, Session, Step, Value, Vec2, bad, json, v3};

/// Where PARTNO is in its prompts.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Ask {
    /// The arrow point.
    Arrow,
    /// The balloon point.
    Balloon,
}

/// The balloon's text height: the dimension style's text height, TEXTSIZE when it has none.
fn text_height(s: &Session) -> f64 {
    let z = sizes(s).text;
    if z.is_finite() && z > 0.0 {
        return z;
    }
    let t = s.doc().map(|d| d.header.f64("TEXTSIZE", 0.2)).unwrap_or(0.2);
    if t.is_finite() && t > 0.0 { t } else { 0.2 }
}

/// The leader, circle and number text of one balloon numbered `n` at `at`; also the preview while
/// the balloon point is being placed. The circle grows with the number's digits so the text fits.
fn balloon(s: &Session, arrow: Vec2, at: Vec2, n: u64) -> Vec<EntityKind> {
    let h = text_height(s);
    let label = n.to_string();
    let r = h * (1.1 + 0.4 * (label.len().saturating_sub(2)) as f64);
    let mut out = Vec::with_capacity(3);
    let d = at - arrow;
    // The leader stops at the circle's edge; a balloon on its own arrow has no leader.
    if d.len() > 1e-12 {
        let edge = at - d.normalized() * r;
        out.push(EntityKind::Leader(cadcraft_doc::Leader { vertices: vec![v3(arrow), v3(edge)], arrow: true, spline: false, style: style(s) }));
    }
    out.push(crate::cmd::helpers::circle(at, r));
    out.push(crate::cmd::draw::make_text(s, at, &label, h, 0.0, (HAlign::Center, VAlign::Middle)));
    out
}

/// Place one balloon's entities; their handles.
fn add_balloon(s: &mut Session, arrow: Vec2, at: Vec2, n: u64) -> Result<Vec<Handle>> {
    let ks = balloon(s, arrow, at, n);
    let mut hs = Vec::with_capacity(ks.len());
    for k in ks {
        hs.push(s.add_entity(k)?);
    }
    Ok(hs)
}

/// PARTNO's JSON form: one balloon at `arrow` → `balloon`, numbered `number` (the session's next
/// number when left out). The handles of the leader, circle and text.
pub(super) fn run(s: &mut Session, p: &Value) -> Result<Value> {
    let arrow = super::point_param(p, "arrow").ok_or_else(|| bad("partno", "`arrow` is required"))?;
    let at = super::point_param(p, "balloon").ok_or_else(|| bad("partno", "`balloon` is required"))?;
    let number = match p.get("number").and_then(Value::as_f64) {
        Some(n) if n.is_finite() && n >= 0.0 && n.fract() == 0.0 => n as u64,
        Some(_) => return Err(bad("partno", "`number` must be a whole number from 0")),
        None => s.last_used.partno_next,
    };
    let hs = add_balloon(s, arrow, at, number)?;
    s.last_used.partno_next = number.saturating_add(1);
    Ok(json!({ "handles": hs.iter().map(|h| h.hex()).collect::<Vec<_>>() }))
}

pub(super) struct PartNoM {
    ask: Ask,
    arrow: Vec2,
    /// The number the next balloon gets.
    next: u64,
}

impl PartNoM {
    /// Starts at the number the last PARTNO left off at.
    pub(super) fn new(s: &Session) -> Self {
        PartNoM { ask: Ask::Arrow, arrow: Vec2::ZERO, next: s.last_used.partno_next }
    }
    fn remember(&self, s: &mut Session) {
        s.last_used.partno_next = self.next;
    }
}

impl Interactive for PartNoM {
    fn name(&self) -> &'static str {
        "PARTNO"
    }
    fn prompt(&self, _s: &Session) -> Prompt {
        match self.ask {
            Ask::Arrow => Prompt::new("Specify arrow point", Accept::POINT_OR_NUMBER),
            Ask::Balloon => Prompt::new("Specify balloon point", Accept::POINT_OR_NUMBER).base(self.arrow),
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match self.ask {
            Ask::Arrow => match i {
                Input::Point(p) => {
                    self.arrow = p;
                    self.ask = Ask::Balloon;
                }
                // A typed whole number is the next balloon's number.
                Input::Text(t) => match t.trim().parse::<u64>() {
                    Ok(n) => self.next = n,
                    _ => s.echo("Requires a point or a part number."),
                },
                // Enter or Esc ends the command; balloons placed so far stay.
                _ => {
                    self.remember(s);
                    return Ok(Step::Done);
                }
            },
            Ask::Balloon => match i {
                Input::Point(p) => {
                    add_balloon(s, self.arrow, p, self.next)?;
                    self.next = self.next.saturating_add(1);
                    // The next balloon follows at once.
                    self.ask = Ask::Arrow;
                }
                Input::Text(t) => match t.trim().parse::<u64>() {
                    Ok(n) => self.next = n,
                    _ => s.echo("Requires a point or a part number."),
                },
                _ => {
                    self.remember(s);
                    return Ok(Step::Done);
                }
            },
        }
        Ok(Step::Continue)
    }
    fn preview(&self, s: &Session, cursor: Vec2) -> Vec<EntityKind> {
        if self.ask == Ask::Balloon { balloon(s, self.arrow, cursor, self.next) } else { Vec::new() }
    }
    fn cancelled(&mut self, s: &mut Session) {
        self.remember(s);
    }
}
