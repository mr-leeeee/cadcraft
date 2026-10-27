use super::*;

fn kinds(s: &Session) -> Vec<&'static str> {
    s.doc().unwrap().model.iter().map(|e| e.kind.type_name()).collect()
}

fn count(s: &Session) -> usize {
    s.doc().unwrap().model.iter().count()
}

fn near(a: &Value, b: f64) -> bool {
    (a.as_f64().unwrap() - b).abs() < 1e-9
}

/// Every polyline in the drawing as a plain (x, y) point list.
fn forms(s: &Session) -> Vec<Vec<(f64, f64)>> {
    s.doc()
        .unwrap()
        .model
        .iter()
        .filter_map(|e| match &e.kind {
            EntityKind::LwPolyline(p) => Some(p.vertices.iter().map(|v| (v.p.x, v.p.y)).collect()),
            _ => None,
        })
        .collect()
}

#[test]
fn run_thread_json() {
    let (d, p) = (10.0, 1.5);
    let mut s = Session::new();
    let r = s.execute("thread", &json!({"at": [0, 0], "diameter": d, "pitch": p, "spacing": 5})).unwrap();
    assert_eq!(r["handles"].as_array().map(Vec::len), Some(5), "{r}");

    // ISO 68-1: d2 = d - 0.64952 P, d1 = d - 1.22687 P, h3 = 0.61343 P; D1 = D - 1.08253 P, H1 = 0.54127 P.
    let (ext, int) = (&r["external"], &r["internal"]);
    assert!(near(&ext["majorDiameter"], d) && near(&int["majorDiameter"], d), "{r}");
    assert!(near(&ext["pitchDiameter"], d - 0.64952 * p) && near(&int["pitchDiameter"], d - 0.64952 * p), "{r}");
    assert!(near(&ext["minorDiameter"], d - 1.22687 * p) && near(&ext["depth"], 0.61343 * p), "{r}");
    assert!(near(&int["minorDiameter"], d - 1.08253 * p) && near(&int["depth"], 0.54127 * p), "{r}");

    // Each thread is drawn over twice its diameter, `spacing` apart, so each axis is
    // d + spacing/2 from the middle.
    assert!((ext["center"][0].as_f64().unwrap() + 12.5).abs() < 1e-9, "{r}");
    assert!((int["center"][0].as_f64().unwrap() - 12.5).abs() < 1e-9, "{r}");

    // Three closed polylines (the shaft's two-sided form, the nut's outline and its bore's form)
    // and two thin centre lines.
    let names = kinds(&s);
    assert_eq!(names.iter().filter(|n| **n == "Polyline").count(), 3, "{names:?}");
    assert_eq!(names.iter().filter(|n| **n == "Arc").count(), 0, "{names:?}");
    assert_eq!(names.iter().filter(|n| **n == "Line").count(), 2, "{names:?}");

    // The external form's crest reaches the major diameter and its root the minor (d1), and every
    // flank has the ISO 68-1 60° slope: it drops the whole depth over depth·tan 30° along the axis.
    let major = d / 2.0;
    let minor = (d - 1.22687 * p) / 2.0;
    let tan30 = 0.5773502691896257;
    let all = forms(&s);
    let shaft = all
        .iter()
        .find(|pts| pts.iter().any(|(_, y)| (y - major).abs() < 1e-9) && pts.iter().any(|(_, y)| (y + major).abs() < 1e-9))
        .unwrap_or_else(|| panic!("no external form: {all:?}"));
    assert!(shaft.iter().any(|(_, y)| (y - minor).abs() < 1e-9) && shaft.iter().any(|(_, y)| (y + minor).abs() < 1e-9), "{shaft:?}");
    let mut flanks = 0;
    for w in shaft.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
        if dy < 1e-9 || dx < 1e-9 {
            continue; // a flat crest or root, or the vertical end of the outline
        }
        assert!((dx / dy - tan30).abs() < 1e-6, "flank {dx}/{dy} is not 60°: {shaft:?}");
        flanks += 1;
    }
    assert!(flanks >= 8, "only {flanks} flanks in the external form");

    // Every centre line is thin; the thread forms keep the drawing's own lineweight.
    let thin = s.doc().unwrap().model.iter().filter(|e| e.common.lineweight == cadcraft_doc::Lineweight::Mm100(THIN_LINE)).count();
    assert_eq!(thin, 2);

    // The drawing reaches past both threads: the nut is a bore radius wider than the shaft, and
    // every centre line runs on past its thread.
    let b = s.doc().unwrap().extents(&cadcraft_doc::Space::Model);
    assert!(b.min.x < -22.5 && b.max.x > 22.5, "{b:?}");
    assert!(b.min.y < -9.9 && b.max.y > 9.9, "{b:?}");
}

#[test]
fn thread_m_prompts_for_diameter_pitch_spacing_and_centre() {
    let mut s = Session::new();
    s.start("thread").unwrap();
    assert!(s.prompt_text().contains("thread diameter"), "{}", s.prompt_text());
    s.input(Input::Text("10".into())).unwrap();
    assert!(s.prompt_text().contains("thread pitch"), "{}", s.prompt_text());
    s.input(Input::Text("1.5".into())).unwrap();
    assert!(s.prompt_text().contains("gap between"), "{}", s.prompt_text());
    s.input(Input::Text("5".into())).unwrap();
    assert!(s.prompt_text().contains("middle point"), "{}", s.prompt_text());
    let before = count(&s);
    s.input(Input::Point(Vec2::new(0.0, 0.0))).unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(count(&s), before + 5);
}

#[test]
fn thread_m_refuses_sizes_it_cannot_draw() {
    let mut s = Session::new();
    s.start("thread").unwrap();
    for bad in ["0", "-3", "abc", ""] {
        s.input(Input::Text(bad.into())).unwrap();
        assert!(s.running.is_some(), "{bad:?} ended the command");
        assert_eq!(count(&s), 0);
    }
    // Enter at the last prompt cancels, leaving the drawing untouched.
    s.input(Input::Text("10".into())).unwrap();
    s.input(Input::Text("1.5".into())).unwrap();
    s.input(Input::Text("5".into())).unwrap();
    s.input(Input::Enter).unwrap();
    assert!(s.running.is_none());
    assert_eq!(count(&s), 0);
}

#[test]
fn thread_previews_the_pair_before_the_pick() {
    let s = Session::new();
    let m = ThreadM { diameter: Some(10.0), pitch: Some(1.5), spacing: Some(5.0) };
    assert_eq!(m.preview(&s, Vec2::ZERO).len(), 5);
    // A pitch too coarse for the diameter previews nothing rather than something degenerate.
    let bad = ThreadM { diameter: Some(10.0), pitch: Some(9.0), spacing: Some(5.0) };
    assert!(bad.preview(&s, Vec2::ZERO).is_empty());
    assert!(ThreadM::default().preview(&s, Vec2::ZERO).is_empty());
}

#[test]
fn thread_json_refuses_sizes_it_cannot_draw() {
    let cases = [
        json!({"at": [0, 0], "diameter": 0, "pitch": 1.5}),
        json!({"at": [0, 0], "diameter": -10, "pitch": 1.5}),
        json!({"at": [0, 0], "diameter": 10, "pitch": 0}),
        json!({"at": [0, 0], "diameter": 10, "pitch": -1.5}),
        json!({"at": [0, 0], "diameter": 10, "pitch": 1.5, "spacing": 0}),
        json!({"at": [0, 0], "diameter": 10, "pitch": 1.5, "spacing": -5}),
        json!({"at": [0, 0], "diameter": 10, "pitch": 9}),
        json!({"at": [0, 0], "diameter": 10, "pitch": 1e-6}),
        json!({"at": [0, 0], "diameter": 1e12, "pitch": 1.5}),
        json!({"at": [0, 0], "diameter": 10, "pitch": 1.5, "spacing": 1e12}),
        json!({"at": [0, 0], "diameter": "wide", "pitch": 1.5}),
        json!({"at": [0, 0], "pitch": 1.5}),
        json!({"at": [0, 0], "diameter": 10}),
        json!({"diameter": 10, "pitch": 1.5}),
        json!({"at": [0, "x"], "diameter": 10, "pitch": 1.5}),
    ];
    let mut s = Session::new();
    for p in &cases {
        assert!(s.execute("thread", p).is_err(), "{p} should be refused");
        assert_eq!(count(&s), 0, "{p} added an object");
    }
}
