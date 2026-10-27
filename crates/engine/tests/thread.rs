//! THREAD: ISO 6410 simplified screw threads ??an external thread (a screw or bolt) and an
//! internal thread (a nut) drawn side by side from a diameter and a pitch.

use cadcraft_doc::{EntityKind, Space};
use cadcraft_engine::Session;
use serde_json::{Value, json};

fn count(s: &Session) -> usize {
    s.doc().map(|d| d.model.iter().count()).unwrap_or(0)
}

fn type_names(s: &Session) -> Vec<&'static str> {
    s.doc().unwrap().model.iter().map(|e| e.kind.type_name()).collect()
}

fn poly_points(s: &Session) -> Vec<Vec<(f64, f64)>> {
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

fn err_of(s: &mut Session, p: &Value) -> String {
    let before = count(s);
    let e = s.execute("thread", p).unwrap_err().to_string();
    assert_eq!(count(s), before, "{p} added an object");
    e
}

#[test]
fn thread_creates_expected_entities() {
    let (d, pitch, gap) = (10.0, 1.5, 5.0);
    let mut s = Session::new();
    let r = s.execute("thread", &json!({"at": [0, 0], "diameter": d, "pitch": pitch, "spacing": gap})).unwrap();

    // Every handle the command reports is really in the drawing.
    let handles = r["handles"].as_array().cloned().unwrap();
    assert_eq!(handles.len(), 5);
    let doc = s.doc().unwrap();
    for h in &handles {
        let h = cadcraft_doc::Handle::parse_hex(h.as_str().unwrap()).unwrap();
        assert!(doc.entity(h).is_some(), "{h:?} is missing");
    }

    let names = type_names(&s);
    assert_eq!(names.iter().filter(|n| **n == "Polyline").count(), 3, "{names:?}");
    assert_eq!(names.iter().filter(|n| **n == "Arc").count(), 0, "{names:?}");
    assert_eq!(names.iter().filter(|n| **n == "Line").count(), 2, "{names:?}");

    // The external thread is a 60° ISO 68-1 form over twice its diameter, its axis `d + spacing/2`
    // to the left: its crest reaches the major diameter d and its root the minor (d1).
    let half = d;
    let (ext_left, _ext_right) = (-(d + gap / 2.0) - half, -(d + gap / 2.0) + half);
    let major = d / 2.0;
    let minor = (d - 1.22687 * pitch) / 2.0;
    let forms = poly_points(&s);
    let shaft = forms
        .iter()
        .find(|pts| pts.iter().any(|(_, y)| (y - major).abs() < 1e-9) && pts.iter().any(|(_, y)| (y + major).abs() < 1e-9))
        .unwrap_or_else(|| panic!("no external form: {forms:?}"));
    assert!(shaft.iter().any(|(_, y)| (y - minor).abs() < 1e-9) && shaft.iter().any(|(_, y)| (y + minor).abs() < 1e-9), "{shaft:?}");
    // Every flank drops the full depth over depth·tan 30° along the axis: the 60° thread slope.
    for w in shaft.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        let (dx, dy) = ((x1 - x0).abs(), (y1 - y0).abs());
        if dy < 1e-9 || dx < 1e-9 {
            continue; // a flat crest or root, or the vertical end of the outline
        }
        assert!((dx / dy - 0.5773502691896257).abs() < 1e-6, "flank {dx}/{dy} is not 60°: {shaft:?}");
    }
    // The form starts on a crest flat at the major diameter, at the left end.
    assert!(shaft.iter().any(|(x, y)| (x - ext_left).abs() < 1e-9 && (y - major).abs() < 1e-9), "{shaft:?}");

    // The nut's outline is a closed rectangle around the bore, its wall one bore radius on each
    // side, so every corner sits a bore diameter from the axis.
    let nut = forms.iter().find(|pts| pts.iter().all(|(_, y)| (y.abs() - d).abs() < 1e-9)).unwrap_or_else(|| panic!("no nut outline"));
    let xs: Vec<f64> = nut.iter().map(|(x, _)| *x).collect();
    assert_eq!(xs.len(), 4, "{nut:?}");
    assert!((xs[0] - (d + gap / 2.0 - half)).abs() < 1e-9 && (xs[2] - (d + gap / 2.0 + half)).abs() < 1e-9, "{nut:?}");

    // The internal thread's form has its crest at the minor diameter D1 and its root at the major.
    let int_minor = (d - 1.08253 * pitch) / 2.0;
    assert!(
        forms.iter().any(|pts| pts.iter().any(|(_, y)| (y - int_minor).abs() < 1e-9) && pts.iter().any(|(_, y)| (y - major).abs() < 1e-9)),
        "{forms:?}"
    );

    // The whole pair renders.
    let _ = cadcraft_render::build(doc, &Space::Model, &cadcraft_render::Options::default());
}

#[test]
fn thread_command_line_draws_the_same_pair() {
    let mut s = Session::new();
    s.cmdline("thread 10 1.5 5 0,0").unwrap();
    assert!(s.running.is_none(), "{}", s.prompt_text());
    assert_eq!(count(&s), 5);
    let names = type_names(&s);
    assert_eq!(names.iter().filter(|n| **n == "Polyline").count(), 3, "{names:?}");

    // A pitch too coarse for the diameter stops at the prompt, having drawn nothing.
    s.cmdline("thread 10").unwrap();
    s.cmdline("9").unwrap();
    s.cmdline("5").unwrap();
    s.cmdline("0,0").unwrap();
    assert_eq!(count(&s), 5, "a bad pitch drew anyway");
    assert!(s.running.is_some());
    s.input(cadcraft_engine::Input::Cancel).unwrap();
    assert_eq!(count(&s), 5);
}

#[test]
fn thread_rejects_bad_params() {
    let mut s = Session::new();
    let cases = [
        (json!({"at": [0, 0], "diameter": 0, "pitch": 1.5}), "`diameter` must be positive"),
        (json!({"at": [0, 0], "diameter": 10, "pitch": 0}), "`pitch` must be positive"),
        (json!({"at": [0, 0], "diameter": 10, "pitch": -1.5}), "`pitch` must be positive"),
        (json!({"at": [0, 0], "diameter": 10, "pitch": 1.5, "spacing": 0}), "`spacing` must be positive"),
        (json!({"at": [0, 0], "diameter": 10, "pitch": 1.5, "spacing": -5}), "`spacing` must be positive"),
        (json!({"at": [0, 0], "diameter": 10, "pitch": 9}), "`pitch` is out of range"),
        (json!({"at": [0, 0], "diameter": 1e12, "pitch": 1.5}), "sizes are out of range"),
        (json!({"at": [0, 0], "diameter": 10, "pitch": 1.5, "spacing": 1e12}), "sizes are out of range"),
        (json!({"at": [0, 0], "pitch": 1.5}), "`diameter` (number) is required"),
        (json!({"at": [0, 0], "diameter": 10}), "`pitch` (number) is required"),
        (json!({"diameter": 10, "pitch": 1.5}), "`at` ([x, y]) is required"),
    ];
    for (p, want) in &cases {
        let err = err_of(&mut s, p);
        assert!(err.contains(want), "{p}: {err}");
    }
    assert_eq!(count(&s), 0);

    // Hostile sizes never panic and never leave a half-drawn pair behind.
    for p in [
        json!({"at": [1e9, -1e9], "diameter": 1e10, "pitch": 1e-9, "spacing": 1e-9}),
        json!({"at": [0, 0], "diameter": 1e-9, "pitch": 1e-9, "spacing": 1e-9}),
        json!({"at": [0, 0], "diameter": 1e308, "pitch": 1.0}),
        json!({"at": [0, 0], "diameter": "M10", "pitch": 1.5}),
        json!({"at": "somewhere", "diameter": 10, "pitch": 1.5}),
    ] {
        let _ = s.execute("thread", &p);
        assert_eq!(count(&s), 0, "{p}");
    }
    let doc = s.doc().unwrap();
    let _ = cadcraft_render::build(doc, &Space::Model, &cadcraft_render::Options::default());
}
