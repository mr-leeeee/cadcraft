//! Rule-based natural language for the assistant: turns a short sentence into `batch` steps.
//!
//! No model, no network. Only the shapes below are understood, and every plan carries a one-line
//! summary of what was understood, so a wrong guess is visible before it reaches the drawing.
//!
//! Understood:
//! - 방/room with size, wall thickness, door and window counts
//! - 사각형/rectangle, 원/circle (radius or diameter), 선/line, 폴리라인/pline, 글자/text
//! - 확대/zoom, 새 도면/new, 취소/undo
//!
//! Units: `5x4` for a room or rectangle is metres (5.0 x 4.0 m); `5000x4000` is millimetres.
//! `m`, `mm`, `cm`, `미터`, `밀리`, `센티` override that. Circles and lines use drawing units.

use serde_json::{json, Value};

/// A parsed instruction: the steps to run plus a summary to show the user.
pub struct Plan {
    pub summary: String,
    pub steps: Vec<Step>,
}

pub struct Step {
    pub command: &'static str,
    pub params: Value,
}

impl Plan {
    /// The `batch` tool argument for these steps.
    pub fn batch(&self) -> Value {
        json!({
            "commands": self
                .steps
                .iter()
                .map(|s| json!({ "command": s.command, "params": s.params }))
                .collect::<Vec<_>>()
        })
    }
}

fn step(command: &'static str, params: Value) -> Step {
    Step { command, params }
}

/// Parse one line of natural language. `None` means "not a sentence I know", so the caller can
/// pass the text to the command line unchanged.
pub fn parse(input: &str) -> Option<Plan> {
    let text = input.trim();
    if text.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    let scale = unit_scale(&lower);

    if has_any(&lower, &["취소", "되돌", "undo", "실행 취소"]) {
        return Some(Plan {
            summary: "실행 취소".into(),
            steps: vec![step("u", json!({}))],
        });
    }

    let fresh = has_any(&lower, &["새 도면", "새도면", "새로", "new drawing", "빈 도면"]);

    if is_room(&lower) {
        return Some(room_plan(&lower, scale, fresh));
    }

    if let Some(p) = rect_plan(&lower, scale) {
        return Some(p);
    }

    if let Some(p) = circle_plan(&lower, scale) {
        return Some(p);
    }

    if let Some(p) = polyline_plan(&lower, scale) {
        return Some(p);
    }

    if let Some(p) = line_plan(&lower, scale) {
        return Some(p);
    }

    if let Some(p) = text_plan(&lower) {
        return Some(p);
    }

    if fresh {
        return Some(Plan {
            summary: "새 도면 (미터법)".into(),
            steps: vec![step("new", json!({ "metric": true }))],
        });
    }

    if has_any(&lower, &["확대", "줌", "전체 보기", "전체보기", "화면에 맞"]) {
        return Some(Plan {
            summary: "전체 보기 (zoom extents)".into(),
            steps: vec![step("zoom", json!({ "mode": "extents" }))],
        });
    }

    None
}

// ---------------------------------------------------------------- entities

fn is_room(lower: &str) -> bool {
    // Don't match room keywords when the text is clearly about creating text entities.
    if has_any(lower, &["글자", "텍스트", "문자", "text", "label"]) {
        return false;
    }
    has_korean_word(lower, "방")
        || has_any(lower, &["룸", "room", "거실", "침실", "주방", "화장실", "사무실", "평면도", "floor plan"])
}

fn room_plan(lower: &str, scale: Option<f64>, fresh: bool) -> Plan {
    let (w, d) = dims(lower, scale, 5000.0, 4000.0);
    let wall = number_after(lower, &["두께", "벽", "wall", "thickness"]).unwrap_or_else(|| if w >= 100.0 { 200.0 } else { 0.2 });
    let wall = wall * scale.unwrap_or(1.0);
    let doors = count_after(lower, &["문", "도어", "door"]).unwrap_or(1);
    let windows = count_after(lower, &["창문", "창", "window"]).unwrap_or(0);
    let label = if lower.contains("거실") {
        "거실"
    } else if lower.contains("침실") {
        "침실"
    } else if lower.contains("주방") {
        "주방"
    } else if lower.contains("화장실") {
        "화장실"
    } else if lower.contains("사무실") {
        "사무실"
    } else {
        "ROOM"
    };

    let mut steps = Vec::new();
    if fresh {
        steps.push(step("new", json!({ "metric": true })));
        steps.push(step("layer.new", json!({ "name": "WALLS", "color": "red", "current": true })));
        steps.push(step("layer.new", json!({ "name": "DOORS", "color": "green" })));
        steps.push(step("layer.new", json!({ "name": "WINDOWS", "color": "cyan" })));
        steps.push(step("layer.new", json!({ "name": "TEXT", "color": "white" })));
    }

    // Outer and inner wall faces.
    steps.push(step("rectang", json!({ "p1": [0.0, 0.0], "p2": [w, d] })));
    steps.push(step("rectang", json!({ "p1": [wall, wall], "p2": [w - wall, d - wall] })));

    // Doors: an opening in the bottom wall plus a leaf standing in the room.
    let leaf = (w / (doors.max(1) as f64 + 1.0)).clamp(300.0, 900.0);
    if fresh {
        steps.push(step("layer.set", json!({ "name": "DOORS" })));
    }
    for i in 1..=doors {
        let cx = w * i as f64 / (doors as f64 + 1.0);
        let x = cx - leaf / 2.0;
        steps.push(step("rectang", json!({ "p1": [x, 0.0], "p2": [x + leaf, wall] })));
        steps.push(step("line", json!({ "points": [[x, wall], [x, wall + leaf]] })));
    }

    // Windows: a band across the top wall.
    let sash = (w / (windows.max(1) as f64 + 1.0)).clamp(300.0, 1200.0);
    if fresh {
        steps.push(step("layer.set", json!({ "name": "WINDOWS" })));
    }
    for i in 1..=windows {
        let cx = w * i as f64 / (windows as f64 + 1.0);
        let x = cx - sash / 2.0;
        steps.push(step("rectang", json!({ "p1": [x, d - wall], "p2": [x + sash, d] })));
    }

    if fresh {
        steps.push(step("layer.set", json!({ "name": "TEXT" })));
    }
    let height = (d * 0.06).clamp(120.0, 400.0);
    steps.push(step(
        "text",
        json!({ "at": [w / 2.0, d / 2.0], "text": label, "height": height, "justify": "MC" }),
    ));

    // Overall dimensions below and to the left.
    steps.push(step("dimlinear", json!({ "p1": [0.0, 0.0], "p2": [w, 0.0], "at": [w / 2.0, -d * 0.12] })));
    steps.push(step("dimlinear", json!({ "p1": [0.0, 0.0], "p2": [0.0, d], "at": [-w * 0.12, d / 2.0] })));
    steps.push(step("zoom", json!({ "mode": "extents" })));

    let (sw, sd, unit) = display_size(w, d, scale);
    let summary = format!(
        "{label} {sw}×{sd} {unit} · 벽 {wall:.0} · 문 {doors} · 창 {windows}{}",
        if fresh { " · 새 도면" } else { "" }
    );
    Plan { summary, steps }
}

fn rect_plan(lower: &str, scale: Option<f64>) -> Option<Plan> {
    let keyword = has_any(lower, &["사각형", "직사각형", "네모", "rectangle", "rect"]);
    if !keyword {
        return None;
    }
    // Reject bare CAD syntax: "rectang 0,0 100,100" or "rect 0,0 100,100".
    if looks_like_raw_cad(lower, &["rectang", "rectangle", "rect"]) {
        return None;
    }
    let pts = pairs(lower);
    let (w, h, origin) = if pts.len() >= 2 {
        let p1 = pts[0];
        let p2 = pts[1];
        (p2[0] - p1[0], p2[1] - p1[1], p1)
    } else {
        let (w, h) = dims(lower, scale, 300.0, 300.0);
        (w, h, [0.0, 0.0])
    };
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (sw, sh, unit) = display_size(w, h, scale);
    Some(Plan {
        summary: format!("사각형 {sw}×{sh} {unit}"),
        steps: vec![
            step("rectang", json!({ "p1": origin, "p2": [origin[0] + w, origin[1] + h] })),
            step("zoom", json!({ "mode": "extents" })),
        ],
    })
}

fn circle_plan(lower: &str, scale: Option<f64>) -> Option<Plan> {
    let korean = has_any(lower, &["원", "써클", "동그라미"]);
    let english = lower.contains("circle") && has_any(lower, &["반지름", "반경", "radius", "지름", "직경", "diameter"]);
    if !korean && !english {
        return None;
    }
    // Reject bare CAD syntax: "circle 0,0 10" or "circle 0,0 5 0".
    if looks_like_raw_cad(lower, &["circle"]) {
        return None;
    }

    let radius = number_after(lower, &["반지름", "반경", "radius"])
        .or_else(|| number_after(lower, &["지름", "직경", "diameter"]).map(|d| d / 2.0))
        // "원 30" / "원 0,0 30": the last number is the radius.
        .or_else(|| numbers(lower).last().copied());
    let radius = radius? * scale.unwrap_or(1.0);
    if radius <= 0.0 {
        return None;
    }
    let center = pairs(lower).first().copied().unwrap_or([0.0, 0.0]);
    let (r, unit) = if scale == Some(1000.0) {
        (format!("{:.2} m", radius / 1000.0), "")
    } else {
        (format!("{radius:.0}"), " (도면 단위)")
    };
    Some(Plan {
        summary: format!("원 반지름 {r}{unit} · 중심 {:.0},{:.0}", center[0], center[1]),
        steps: vec![
            step("circle", json!({ "center": center, "radius": radius })),
            step("zoom", json!({ "mode": "extents" })),
        ],
    })
}

/// True if the text looks like a raw CAD command (English keyword immediately followed by coordinates/numbers).
/// Korean keywords are allowed to be followed by numbers (they're considered natural language).
fn looks_like_raw_cad(lower: &str, english_keywords: &[&str]) -> bool {
    for kw in english_keywords {
        if let Some(pos) = lower.find(kw) {
            let after = &lower[pos + kw.len()..];
            let trimmed = after.trim_start();
            if trimmed.starts_with(|c: char| c.is_ascii_digit() || c == '@' || c == '-') {
                return true;
            }
        }
    }
    false
}

fn line_plan(lower: &str, scale: Option<f64>) -> Option<Plan> {
    // `선택`/`선택된` is not a line.
    let named = has_any(lower, &["라인", "line"]) || (has_korean_word(lower, "선") && !lower.contains("선택"));
    if !named {
        return None;
    }
    // Reject bare CAD syntax: "line 0,0 10,0" (English keyword + coords).
    if looks_like_raw_cad(lower, &["line", "rectang", "rectangle", "rect", "circle", "pline", "polyline", "zoom"]) {
        return None;
    }

    let pts = pairs(lower);
    if pts.len() < 2 {
        return None;
    }
    let s = scale.unwrap_or(1.0);
    let points: Vec<[f64; 2]> = pts.iter().map(|p| [p[0] * s, p[1] * s]).collect();
    let summary = format!("선 {}개 점", points.len());
    Some(Plan {
        summary,
        steps: vec![
            step("line", json!({ "points": points })),
            step("zoom", json!({ "mode": "extents" })),
        ],
    })
}

fn polyline_plan(lower: &str, scale: Option<f64>) -> Option<Plan> {
    if !has_any(lower, &["폴리라인", "polyline", "pline"]) {
        return None;
    }
    // Reject bare CAD syntax: "pline 0,0 100,100" or "polyline 0,0 100,100".
    if looks_like_raw_cad(lower, &["pline", "polyline"]) {
        return None;
    }
    let pts = pairs(lower);
    if pts.len() < 2 {
        return None;
    }
    let s = scale.unwrap_or(1.0);
    let vertices: Vec<[f64; 2]> = pts.iter().map(|p| [p[0] * s, p[1] * s]).collect();
    let closed = has_any(lower, &["닫", "closed", "close"]);
    Some(Plan {
        summary: format!("폴리라인 {}개 점{}", vertices.len(), if closed { " (닫힘)" } else { "" }),
        steps: vec![
            step("pline", json!({ "vertices": vertices, "closed": closed })),
            step("zoom", json!({ "mode": "extents" })),
        ],
    })
}

fn text_plan(lower: &str) -> Option<Plan> {
    if !has_any(lower, &["글자", "텍스트", "문자", "text", "label"]) {
        return None;
    }
    let content = quoted(lower).unwrap_or_else(|| {
        let rest = ["글자", "텍스트", "문자", "text", "label"]
            .iter()
            .filter_map(|k| lower.find(k).map(|i| &lower[i + k.len()..]))
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        strip_verbs(&rest)
    });
    if content.is_empty() {
        return None;
    }
    let height = number_after(lower, &["높이", "height"]).unwrap_or(200.0);
    let at = pairs(lower).first().copied().unwrap_or([0.0, 0.0]);
    Some(Plan {
        summary: format!("글자 \"{content}\" 높이 {height:.0}"),
        steps: vec![
            step("text", json!({ "at": at, "text": content, "height": height })),
            step("zoom", json!({ "mode": "extents" })),
        ],
    })
}

// ---------------------------------------------------------------- numbers and units

struct Num {
    value: f64,
    start: usize,
    end: usize,
}

fn num_spans(s: &str) -> Vec<Num> {
    let bytes: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let negative = c == '-' && bytes.get(i + 1).is_some_and(|n| n.is_ascii_digit());
        if c.is_ascii_digit() || negative {
            let start = i;
            let mut j = if negative { i + 1 } else { i };
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == '.') {
                j += 1;
            }
            let token: String = bytes[start..j].iter().collect();
            if let Ok(value) = token.parse::<f64>() {
                out.push(Num { value, start, end: j });
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn numbers(s: &str) -> Vec<f64> {
    num_spans(s).into_iter().map(|n| n.value).collect()
}

/// `num , num` pairs, e.g. the points in `선 0,0 100,100`.
fn pairs(s: &str) -> Vec<[f64; 2]> {
    let spans = num_spans(s);
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < spans.len() {
        let a = &spans[i];
        let b = &spans[i + 1];
        let between: String = chars[a.end..b.start].iter().collect();
        if between.trim() == "," {
            out.push([a.value, b.value]);
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// `5x4`, `5 × 4`, `5*4`, `5 by 4`, `5m x 4m`.
fn size_pair(s: &str) -> Option<(f64, f64)> {
    let spans = num_spans(s);
    let chars: Vec<char> = s.chars().collect();
    for w in spans.windows(2) {
        let between: String = chars[w[0].end..w[1].start].iter().collect();
        let between = between.trim().to_lowercase();
        if matches!(between.as_str(), "x" | "×" | "*" | "by" | "곱" | "곱하기" | "ㅌ") {
            return Some((w[0].value, w[1].value));
        }
    }
    None
}

/// Millimetres per unit when the text names one.
fn unit_scale(s: &str) -> Option<f64> {
    if s.contains("밀리") || s.contains("mm") || s.contains("㎜") {
        return Some(1.0);
    }
    if s.contains("센티") || s.contains("cm") || s.contains("㎝") {
        return Some(10.0);
    }
    if s.contains("미터") || s.contains("meter") || s.contains("metre") {
        return Some(1000.0);
    }
    // A digit directly followed by `m`: "5m", "5m x 4m".
    let chars: Vec<char> = s.chars().collect();
    for i in 1..chars.len() {
        if chars[i] == 'm' && chars[i - 1].is_ascii_digit() && !chars.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic()) {
            return Some(1000.0);
        }
    }
    None
}

/// The size of a room or rectangle, in drawing units.
///
/// Bare numbers up to 50 are metres (`5x4` is a 5 m room); larger numbers are drawing units
/// (`5000x4000` is millimetres). A named unit always wins.
fn dims(s: &str, scale: Option<f64>, default_w: f64, default_d: f64) -> (f64, f64) {
    let (raw_w, raw_d) = if let Some((w, h)) = size_pair(s) {
        (w, h)
    } else if let (Some(w), Some(d)) = (
        number_after(s, &["가로", "width", "폭"]),
        number_after(s, &["세로", "height", "depth", "깊이"]),
    ) {
        (w, d)
    } else {
        // Any two numbers that are not a wall thickness or a count.
        let spans = num_spans(s);
        let chars: Vec<char> = s.chars().collect();
        let mut kept = Vec::new();
        for n in &spans {
            let before: String = chars[n.start.saturating_sub(8)..n.start].iter().collect();
            let after: String = chars[n.end..chars.len().min(n.end + 3)].iter().collect();
            let is_thickness = ["두께", "벽", "wall", "thickness"].iter().any(|k| before.contains(k));
            let is_count = after.trim_start().starts_with('개');
            if !is_thickness && !is_count {
                kept.push(n.value);
            }
        }
        match kept.len() {
            0 => (default_w, default_d),
            1 => (kept[0], kept[0]),
            _ => (kept[0], kept[1]),
        }
    };

    let factor = match scale {
        Some(s) => s,
        None => {
            if raw_w.max(raw_d) <= 50.0 {
                1000.0
            } else {
                1.0
            }
        }
    };
    (raw_w * factor, raw_d * factor)
}

/// A number within 12 characters after the first of `keywords`.
fn number_after(s: &str, keywords: &[&str]) -> Option<f64> {
    for keyword in keywords {
        if let Some(pos) = s.find(keyword) {
            let tail: String = s[pos + keyword.len()..].chars().take(12).collect();
            if let Some(n) = numbers(&tail).first() {
                return Some(*n);
            }
        }
    }
    None
}

/// How many of something: "문 2개", "창 두 개", "door 3".
fn count_after(s: &str, keywords: &[&str]) -> Option<u32> {
    for keyword in keywords {
        if let Some(pos) = s.find(keyword) {
            let tail: String = s[pos + keyword.len()..].chars().take(8).collect();
            if let Some(n) = numbers(&tail).first() {
                if *n >= 0.0 && *n <= 12.0 {
                    return Some(*n as u32);
                }
            }
            for (word, value) in [
                ("하나", 1), ("한", 1), ("둘", 2), ("두", 2), ("셋", 3), ("세", 3),
                ("넷", 4), ("네", 4), ("다섯", 5), ("여섯", 6), ("일곱", 7), ("여덟", 8),
            ] {
                if tail.contains(word) {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// How a size is shown in the summary: metres when it was given in metres, otherwise millimetres.
fn display_size(w: f64, d: f64, scale: Option<f64>) -> (String, String, &'static str) {
    if scale == Some(1000.0) || (scale.is_none() && w.max(d) <= 50.0) {
        (format!("{:.1}", w / 1000.0), format!("{:.1}", d / 1000.0), "m")
    } else {
        (format!("{w:.0}"), format!("{d:.0}"), "mm")
    }
}

// ---------------------------------------------------------------- text helpers

fn has_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn is_hangul(c: char) -> bool {
    matches!(c, '\u{AC00}'..='\u{D7A3}' | '\u{3131}'..='\u{318E}')
}

/// A Korean word that is not merely part of a longer word: `방` matches "방 그려", "방을",
/// "방5x4" but not "정보".
fn has_korean_word(s: &str, word: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let target: Vec<char> = word.chars().collect();
    if target.is_empty() || chars.len() < target.len() {
        return false;
    }
    for i in 0..=chars.len() - target.len() {
        if chars[i..i + target.len()] != target[..] {
            continue;
        }
        let prev_ok = i == 0 || !is_hangul(chars[i - 1]);
        let next_ok = match chars.get(i + target.len()) {
            None => true,
            Some(c) => {
                !is_hangul(*c)
                    || matches!(
                        c,
                        '을' | '를' | '이' | '가' | '은' | '는' | '도' | '만' | '에' | '으'
                            | '하' | '그' | '좀' | '와' | '과' | '로' | '써' | '만' | '두' | '세' | '한'
                    )
            }
        };
        if prev_ok && next_ok {
            return true;
        }
    }
    false
}

fn quoted(s: &str) -> Option<String> {
    for (open, close) in [('"', '"'), ('\'', '\''), ('「', '」'), ('『', '』')] {
        if let (Some(a), Some(b)) = (s.find(open), s.rfind(close)) {
            if b > a + 1 {
                let result = s[a + 1..b].trim().to_string();
                return Some(result);
            }
        }
    }
    None
}

fn strip_verbs(s: &str) -> String {
    let mut out = s.trim().to_string();
    for verb in ["그려줘", "그려", "써줘", "써", "작성해줘", "작성", "넣어줘", "넣어", "추가해줘", "추가", "표시해줘", "표시", "로", "라고"] {
        if out.ends_with(verb) {
            out.truncate(out.len() - verb.len());
            out = out.trim_end_matches([' ', ',', '.']).to_string();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commands(plan: &Plan) -> Vec<&'static str> {
        plan.steps.iter().map(|s| s.command).collect()
    }

    #[test]
    fn korean_room_with_metres() {
        let plan = parse("거실 5x4미터 방 그려줘").expect("parsed");
        assert!(commands(&plan).contains(&"rectang"), "{:?}", commands(&plan));
        assert!(plan.summary.contains("5.0×4.0 m"), "{}", plan.summary);
        let first = &plan.steps[0];
        assert_eq!(first.command, "rectang");
        assert_eq!(first.params["p2"], json!([5000.0, 4000.0]));
    }

    #[test]
    fn room_without_unit_is_metres() {
        let plan = parse("방 5x4").expect("parsed");
        assert_eq!(plan.steps[0].params["p2"], json!([5000.0, 4000.0]));
    }

    #[test]
    fn room_in_millimetres_stays() {
        let plan = parse("방 3000x4000").expect("parsed");
        assert_eq!(plan.steps[0].params["p2"], json!([3000.0, 4000.0]));
    }

    #[test]
    fn wall_thickness_is_not_a_room_size() {
        let plan = parse("벽 두께 150 방 4x3");
        let plan = plan.expect("parsed");
        assert_eq!(plan.steps[0].params["p2"], json!([4000.0, 3000.0]), "{}", plan.summary);
        assert!(plan.summary.contains("벽 150"), "{}", plan.summary);
        let inner = &plan.steps[1];
        assert_eq!(inner.params["p1"], json!([150.0, 150.0]));
    }

    #[test]
    fn door_and_window_counts() {
        let plan = parse("방 6x5 문 2개 창 1개").expect("parsed");
        assert!(plan.summary.contains("문 2"), "{}", plan.summary);
        assert!(plan.summary.contains("창 1"), "{}", plan.summary);
        // outer, inner, 2 door openings, 1 window = 5 rectang
        let rects = plan.steps.iter().filter(|s| s.command == "rectang").count();
        assert_eq!(rects, 5, "outer, inner, 2 door openings, 1 window");
    }

    #[test]
    fn korean_number_words() {
        let plan = parse("방 6x5 문 두 개").expect("parsed");
        assert!(plan.summary.contains("문 2"), "{}", plan.summary);
    }

    #[test]
    fn circle_radius_and_diameter() {
        let plan = parse("반지름 30인 원").expect("parsed");
        assert_eq!(plan.steps[0].command, "circle");
        assert_eq!(plan.steps[0].params["radius"], json!(30.0));

        let plan = parse("지름 50 원").expect("parsed");
        assert_eq!(plan.steps[0].params["radius"], json!(25.0));
    }

    #[test]
    fn circle_with_centre() {
        let plan = parse("원 100,200 반지름 40").expect("parsed");
        assert_eq!(plan.steps[0].params["center"], json!([100.0, 200.0]));
        assert_eq!(plan.steps[0].params["radius"], json!(40.0));
    }

    #[test]
    fn rectangle_by_size_and_by_points() {
        let plan = parse("300x400 사각형").expect("parsed");
        assert_eq!(plan.steps[0].params["p1"], json!([0.0, 0.0]));
        assert_eq!(plan.steps[0].params["p2"], json!([300.0, 400.0]));

        let plan = parse("사각형 10,20 110,120").expect("parsed");
        assert_eq!(plan.steps[0].params["p1"], json!([10.0, 20.0]));
        assert_eq!(plan.steps[0].params["p2"], json!([110.0, 120.0]));
    }

    #[test]
    fn line_from_points() {
        let plan = parse("선 0,0 100,100").expect("parsed");
        assert_eq!(plan.steps[0].command, "line");
        assert_eq!(plan.steps[0].params["points"], json!([[0.0, 0.0], [100.0, 100.0]]));
    }

    #[test]
    fn text_content_and_height() {
        let plan = parse("글자 \"거실\" 높이 300").expect("parsed");
        assert_eq!(plan.steps[0].params["text"], json!("거실"));
        assert_eq!(plan.steps[0].params["height"], json!(300.0));
    }

    #[test]
    fn view_and_new_and_undo() {
        assert_eq!(commands(&parse("전체 보기").unwrap()), vec!["zoom"]);
        assert_eq!(commands(&parse("취소").unwrap()), vec!["u"]);
        assert_eq!(commands(&parse("새 도면").unwrap()), vec!["new"]);
    }

    #[test]
    fn cad_commands_are_not_natural_language() {
        // Typed CAD syntax must reach the command line untouched.
        assert!(parse("circle 0,0 10").is_none());
        assert!(parse("line 0,0 10,0").is_none());
        assert!(parse("rectang 0,0 100,100").is_none());
        assert!(parse("zoom").is_none());
        assert!(parse("layer.new Walls").is_none());
        assert!(parse("asdf").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn selection_is_not_a_line() {
        assert!(parse("선택").is_none());
    }

    #[test]
    fn new_room_rebuilds_layers() {
        let plan = parse("새로 방 5x4").expect("parsed");
        let cmds = commands(&plan);
        assert_eq!(cmds[0], "new");
        assert!(cmds.contains(&"layer.new"));
    }
}
