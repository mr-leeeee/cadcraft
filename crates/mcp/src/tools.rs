//! MCP tool definitions and dispatch.

use serde_json::{Value, json};

use crate::backend::Backend;

pub fn tool_definitions(has_ui: bool) -> Value {
    let mut tools = vec![
        json!({"name": "command_line", "description": "Type one line at the CADCraft command line, exactly as a user would at an AutoCAD-style prompt. Starts commands by name or alias (LINE, L, CIRCLE, C, PLINE, RECTANG, MOVE, TRIM, OFFSET, ZOOM…) and answers the active prompt: points `x,y`, relative `@dx,dy`, polar `@dist<angle`, distances, keywords (e.g. `c` for Close), empty text = Enter. Spaces act as Enter, so `circle 0,0 5` works in one call. Returns the new prompt and command-line output, and `error` when the line was refused.",
            "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]}}),
        json!({"name": "script", "description": "Run a multi-line script (like an AutoCAD .scr file): one or more inputs per line, blank line = Enter.",
            "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}, "required": ["text"]}}),
        json!({"name": "execute", "description": "Run any command with JSON parameters (no prompts or dialogs). Use list_commands to find ids and parameter docs. Examples: line {points:[[0,0],[10,0]]}, circle {center:[0,0], radius:5}, layer.new {name:\"Walls\", color:\"red\", current:true}, move {handles:[\"10A\"], delta:[5,0]}, zoom {mode:\"extents\"}.",
            "inputSchema": {"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}}),
        json!({"name": "batch", "description": "Run multiple commands atomically with automatic rollback on failure. Each step: {command, params}. On error, completed steps are undone. Returns {completed, results, rolled_back, undone}.",
            "inputSchema": {"type": "object", "properties": {"commands": {"type": "array", "items": {"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}}, "rollback": {"type": "boolean", "default": true}}, "required": ["commands"]}}),
        json!({"name": "inspect_design", "description": "Complete design snapshot: entities with geometry, layers, blocks, layouts, viewports, selection, undo depth, and named views. One call for full context.",
            "inputSchema": {"type": "object", "properties": {"entities": {"type": "boolean", "default": true}, "layers": {"type": "boolean", "default": true}, "blocks": {"type": "boolean", "default": true}, "views": {"type": "boolean", "default": true}, "selection": {"type": "boolean", "default": true}}}}),
        json!({"name": "measure", "description": "Measure distances, areas, perimeters, angles, and bounding boxes. Input: items array of {type: \"entity\", handle} or {type: \"point\", x, y}. Two items = distance between them. Returns per-item metrics and min/max/center distances.",
            "inputSchema": {"type": "object", "properties": {"items": {"type": "array", "items": {"type": "object", "properties": {"type": {"type": "string", "enum": ["entity", "point"]}, "handle": {"type": "string"}, "x": {"type": "number"}, "y": {"type": "number"}}, "required": ["type"]}}}, "required": ["items"]}}),
        json!({"name": "skill_reference", "description": "Read a bundled agent skill document. Skills: cadcraft-2d-drafting (reference.md), cadcraft-best-practices (layers.md, blocks.md, dims.md).",
            "inputSchema": {"type": "object", "properties": {"skill": {"type": "string", "enum": ["cadcraft-2d-drafting", "cadcraft-best-practices"]}, "file": {"type": "string", "default": "reference.md"}}, "required": ["skill"]}}),
        json!({"name": "list_commands", "description": "List commands with ids, labels, menu paths, aliases and parameter docs. Optional filter: a substring of the command id, label or an alias.",
            "inputSchema": {"type": "object", "properties": {"filter": {"type": "string"}}}}),
        json!({"name": "inspect_drawing", "description": "Summary of the active drawing: entity counts, extents, layers, styles, blocks, layouts, selection, undo history and (optionally) entities with handles and geometry.",
            "inputSchema": {"type": "object", "properties": {"entities": {"type": "boolean"}, "limit": {"type": "integer"}}}}),
        json!({"name": "query_entities", "description": "Entities filtered by type (Line, Circle, Polyline…), layer and/or a window [[x0,y0],[x1,y1]] (objects whose geometry is inside or crosses it; crossing:false = entirely inside). Returns count (all matches), returned (this page) and entities; page with offset and limit (default 500).",
            "inputSchema": {"type": "object", "properties": {"type": {"type": "string"}, "layer": {"type": "string"},
                "window": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2},
                "crossing": {"type": "boolean"}, "limit": {"type": "integer", "minimum": 0}, "offset": {"type": "integer", "minimum": 0}}}}),
        json!({"name": "render", "description": "Render the active drawing to a PNG and return it as an image: fitted to the extents by default, or just the area `window` [[x0,y0],[x1,y1]] (drawing units), e.g. one sheet or one room of a large drawing. With a window and no size the image is 1600 px wide with the window's shape.",
            "inputSchema": {"type": "object", "properties": {"width": {"type": "integer", "minimum": 16}, "height": {"type": "integer", "minimum": 16}, "fit": {"type": "boolean"},
                "window": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2},
                "path": {"type": "string"}}}}),
        json!({"name": "find_text", "description": "Read the drawing's texts: TEXT, MTEXT (formatting removed), block attributes and multileaders, each with its text, insertion point `at`, height, rotation, style, layer and handle. Filter by `contains` (any case), `layer` and/or `window` [[x0,y0],[x1,y1]] around the insertion points; page with offset and limit (default 200). Use it to find room names, sheet titles and notes without reading all the geometry.",
            "inputSchema": {"type": "object", "properties": {"contains": {"type": "string"}, "layer": {"type": "string"},
                "window": {"type": "array", "items": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}, "minItems": 2, "maxItems": 2},
                "limit": {"type": "integer", "minimum": 0}, "offset": {"type": "integer", "minimum": 0}}}}),
        json!({"name": "new_drawing", "description": "Create a new drawing (imperial by default, or metric).",
            "inputSchema": {"type": "object", "properties": {"metric": {"type": "boolean"}}}}),
        json!({"name": "open", "description": "Open a DXF file.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}}),
        json!({"name": "save", "description": "Save the drawing (DXF or DWG by extension). A .svg, .png or .pdf path exports instead: the drawing keeps its name and unsaved changes.", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}}),
        json!({"name": "cancel", "description": "Press Escape: cancel the running command (or clear the selection).", "inputSchema": {"type": "object", "properties": {}}}),
    ];
    if has_ui {
        tools.push(json!({"name": "screenshot", "description": "Screenshot the CADCraft window (PNG).", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}}));
        tools.push(json!({"name": "ui_inspect", "description": "UI state: panels, canvas rect, view, cursor and snap, performance.", "inputSchema": {"type": "object", "properties": {}}}));
        tools.push(json!({"name": "ui_click", "description": "Click in the drawing area at world coordinates (or screen with space:\"screen\"). Right button = Enter.", "inputSchema": {"type": "object", "properties": {"x": {"type": "number"}, "y": {"type": "number"}, "space": {"type": "string"}, "button": {"type": "string"}}, "required": ["x", "y"]}}));
    }
    json!({ "tools": tools })
}

fn text(v: &Value) -> Value {
    json!({"content": [{"type": "text", "text": serde_json::to_string_pretty(v).unwrap_or_default()}]})
}

fn err_text(e: &str) -> Value {
    json!({"content": [{"type": "text", "text": e}], "isError": true})
}

/// Whether `v` fits the subset of JSON Schema the tool definitions use: `type`, `items`,
/// `minItems`, `maxItems` and `minimum`.
fn fits(v: &Value, schema: &Value) -> bool {
    let typed = match schema["type"].as_str() {
        Some("string") => v.is_string(),
        // JSON Schema integers include numbers such as 2.0.
        Some("integer") => v.as_f64().is_some_and(|f| f.fract() == 0.0),
        Some("number") => v.is_number(),
        Some("boolean") => v.is_boolean(),
        Some("array") => v.is_array(),
        Some("object") => v.is_object(),
        _ => true,
    };
    let len = v.as_array().map(Vec::len);
    typed
        && schema["minimum"].as_f64().is_none_or(|m| v.as_f64().is_none_or(|f| f >= m))
        && schema["minItems"].as_u64().is_none_or(|m| len.is_none_or(|n| n as u64 >= m))
        && schema["maxItems"].as_u64().is_none_or(|m| len.is_none_or(|n| n as u64 <= m))
        && (schema["items"].is_null() || v.as_array().is_none_or(|a| a.iter().all(|i| fits(i, &schema["items"]))))
}

/// Arguments checked against the tool's input schema: an object, required properties present and
/// each given property fitting its schema (`null` counts as not given). `Err` is the
/// invalid-params message.
fn check_args(name: &str, args: &Value) -> Result<Value, String> {
    let defs = tool_definitions(true);
    let schema = defs["tools"].as_array().and_then(|t| t.iter().find(|t| t["name"] == name)).map(|t| t["inputSchema"].clone());
    let Some(schema) = schema else { return Err(format!("unknown tool `{name}`")) };
    let a = match args {
        Value::Null => json!({}),
        Value::Object(_) => args.clone(),
        _ => return Err(format!("`arguments` of `{name}` must be an object")),
    };
    for key in schema["required"].as_array().into_iter().flatten().filter_map(Value::as_str) {
        if a.get(key).is_none_or(Value::is_null) {
            return Err(format!("`{name}` needs `{key}`"));
        }
    }
    for (key, prop) in schema["properties"].as_object().into_iter().flatten() {
        if let Some(v) = a.get(key).filter(|v| !v.is_null())
            && !fits(v, prop)
        {
            return Err(format!("`{key}` of `{name}` must match {prop}"));
        }
    }
    Ok(a)
}

/// Whether a command-catalog entry's id, label or one of its aliases contains `f` (lower case).
fn command_matches(c: &Value, f: &str) -> bool {
    let has = |s: &str| s.to_ascii_lowercase().contains(f);
    ["id", "label"].iter().any(|k| c[*k].as_str().is_some_and(has))
        || c["aliases"].as_array().is_some_and(|a| a.iter().filter_map(Value::as_str).any(has))
}

/// Run a tool. `Err` is a protocol error (unknown tool, or arguments that don't fit its input
/// schema), answered with JSON-RPC invalid params; a failing tool is an `isError` result.
pub fn call_tool(b: &mut dyn Backend, name: &str, args: &Value) -> Result<Value, String> {
    let a = check_args(name, args)?;
    let r = match name {
        "command_line" => b.call("cmdline.input", a),
        "script" => b.call("cmdline.script", a),
        "execute" => b.call(
            "engine.execute",
            json!({"command": a.get("command").cloned().unwrap_or(Value::Null), "params": a.get("params").cloned().unwrap_or(json!({}))}),
        ),
        "batch" => {
            let cmds: Vec<Value> = a.get("commands").and_then(Value::as_array).map(|v| v.to_vec()).unwrap_or_default();
            let rollback = a.get("rollback").and_then(Value::as_bool).unwrap_or(true);
            // Execute each command, collect results, rollback on failure
            let mut results = Vec::new();
            let mut completed = 0;
            let mut failed = false;
            for (i, cmd) in cmds.iter().enumerate() {
                let cmd_name = cmd.get("command").and_then(Value::as_str).unwrap_or("");
                let params = cmd.get("params").cloned().unwrap_or(json!({}));
                match b.call("engine.execute", json!({"command": cmd_name, "params": params})) {
                    Ok(v) => {
                        results.push(json!({"index": i, "command": cmd_name, "result": v}));
                        completed += 1;
                    }
                    Err(e) => {
                        failed = true;
                        results.push(json!({"index": i, "command": cmd_name, "error": e.clone()}));
                        break;
                    }
                }
            }
            let mut rolled_back = false;
            let mut undone = Vec::new();
            if failed && rollback {
                // Undo completed steps in reverse order
                for (i, cmd) in cmds.iter().take(completed).enumerate().rev() {
                    let _cmd_name = cmd.get("command").and_then(Value::as_str).unwrap_or("");
                    // Try to undo via engine.execute with "undo" command
                    if let Ok(_) = b.call("engine.execute", json!({"command": "undo", "params": {}})) {
                        undone.push(i);
                    }
                }
                rolled_back = !undone.is_empty();
            }
            Ok(json!({"completed": completed, "results": results, "rolled_back": rolled_back, "undone": undone}))
        },
        "inspect_design" => {
            let mut result = json!({});
            let entities = a.get("entities").and_then(Value::as_bool).unwrap_or(true);
            let layers = a.get("layers").and_then(Value::as_bool).unwrap_or(true);
            let blocks = a.get("blocks").and_then(Value::as_bool).unwrap_or(true);
            let views = a.get("views").and_then(Value::as_bool).unwrap_or(true);
            let selection = a.get("selection").and_then(Value::as_bool).unwrap_or(true);
            
            if entities {
                if let Ok(v) = b.call("drawing.inspect", json!({"entities": true, "limit": 1000})) {
                    result["entities"] = v;
                }
            }
            if layers {
                if let Ok(v) = b.call("engine.execute", json!({"command": "layer.list", "params": {}})) {
                    result["layers"] = v;
                }
            }
            if blocks {
                if let Ok(v) = b.call("engine.execute", json!({"command": "block.list", "params": {}})) {
                    result["blocks"] = v;
                }
            }
            if views {
                if let Ok(v) = b.call("engine.execute", json!({"command": "view.list", "params": {}})) {
                    result["views"] = v;
                }
            }
            if selection {
                if let Ok(v) = b.call("engine.execute", json!({"command": "selection.get", "params": {}})) {
                    result["selection"] = v;
                }
            }
            // Add undo depth
            if let Ok(v) = b.call("engine.execute", json!({"command": "undo.info", "params": {}})) {
                result["undo"] = v;
            }
            Ok(result)
        },
        "measure" => {
            let items: Vec<Value> = a.get("items").and_then(Value::as_array).map(|v| v.to_vec()).unwrap_or_default();
            let mut results = Vec::new();
            let mut positions = Vec::new();
            
            // First, collect all handles we need to look up
            let handles: Vec<String> = items.iter()
                .filter_map(|item| {
                    if item.get("type").and_then(Value::as_str) == Some("entity") {
                        item.get("handle").and_then(Value::as_str).map(String::from)
                    } else {
                        None
                    }
                })
                .collect();
            
            // Fetch all entities once if we need any entity lookups
            let all_entities = if !handles.is_empty() {
                b.call("engine.execute", json!({"command": "entities", "params": {}}))
                    .ok()
                    .and_then(|v| v.get("entities").and_then(Value::as_array).cloned())
                    .unwrap_or_default()
            } else {
                vec![]
            };
            
            for item in items {
                if let Some(t) = item.get("type").and_then(Value::as_str) {
                    match t {
                        "entity" => {
                            if let Some(handle) = item.get("handle").and_then(Value::as_str) {
                                // Find entity by handle in the fetched list
                                let mut found = false;
                                for ent in &all_entities {
                                    if ent.get("handle").and_then(Value::as_str) == Some(handle) {
                                        let mut info = ent.clone();
                                        // Extract geometry info first to avoid borrow issues
                                        let (kind, radius, area, perimeter, center) = if let Some(ent_obj) = info.get("entity").or_else(|| info.get("geometry")) {
                                            let kind = ent_obj.get("type").and_then(Value::as_str).or_else(|| ent_obj.get("kind").and_then(Value::as_str)).unwrap_or("");
                                            let radius = ent_obj.get("radius").and_then(Value::as_f64);
                                            let area = ent_obj.get("area").and_then(Value::as_f64);
                                            let perimeter = ent_obj.get("perimeter").and_then(Value::as_f64);
                                            let center = ent_obj.get("center").and_then(Value::as_array).cloned();
                                            (kind, radius, area, perimeter, center)
                                        } else {
                                            ("", None, None, None, None)
                                        };
                                        
                                        // Add geometry-specific measurements
                                        match kind {
                                            "circle" | "Circle" | "arc" | "Arc" => {
                                                if let Some(r) = radius {
                                                    info["diameter"] = json!(r * 2.0);
                                                    info["circumference"] = json!(2.0 * std::f64::consts::PI * r);
                                                    info["area"] = json!(std::f64::consts::PI * r * r);
                                                }
                                            }
                                            "lwPolyline" | "LwPolyline" | "Polyline" | "polyline" => {
                                                if let Some(a) = area {
                                                    info["area"] = json!(a.abs());
                                                }
                                                if let Some(p) = perimeter {
                                                    info["perimeter"] = json!(p);
                                                }
                                            }
                                            _ => {}
                                        }
                                        
                                        // Extract position for distance calculation
                                        if let Some(center) = center {
                                            if center.len() >= 2 {
                                                positions.push((center[0].as_f64().unwrap_or(0.0), center[1].as_f64().unwrap_or(0.0)));
                                            }
                                        }
                                        
                                        results.push(info);
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    results.push(json!({"handle": handle, "error": "not found"}));
                                }
                            }
                        }
                        "point" => {
                            if let (Some(x), Some(y)) = (item.get("x").and_then(Value::as_f64), item.get("y").and_then(Value::as_f64)) {
                                positions.push((x, y));
                                results.push(json!({"type": "point", "x": x, "y": y}));
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            // Calculate distances between points
            let mut distances = json!({});
            if positions.len() >= 2 {
                let mut min_dist: f64 = f64::INFINITY;
                let mut max_dist: f64 = 0.0;
                for i in 0..positions.len() {
                    for j in i+1..positions.len() {
                        let dx = positions[i].0 - positions[j].0;
                        let dy = positions[i].1 - positions[j].1;
                        let d = (dx*dx + dy*dy).sqrt();
                        min_dist = min_dist.min(d);
                        max_dist = max_dist.max(d);
                    }
                }
                if min_dist != f64::INFINITY {
                    distances["min"] = json!(min_dist);
                    distances["max"] = json!(max_dist);
                    // Center distance (average of centers)
                    let cx: f64 = positions.iter().map(|p| p.0).sum::<f64>() / positions.len() as f64;
                    let cy: f64 = positions.iter().map(|p| p.1).sum::<f64>() / positions.len() as f64;
                    let mut center_dists = Vec::new();
                    for (x, y) in &positions {
                        let dx = *x - cx;
                        let dy = *y - cy;
                        center_dists.push((dx*dx + dy*dy).sqrt());
                    }
                    if !center_dists.is_empty() {
                        distances["center_avg"] = json!(center_dists.iter().sum::<f64>() / center_dists.len() as f64);
                    }
                }
            }
            
            Ok(json!({"items": results, "distances": distances}))
        },
        "skill_reference" => {
            let skill = a.get("skill").and_then(Value::as_str).unwrap_or("");
            let file = a.get("file").and_then(Value::as_str).unwrap_or("reference.md");
            let content = match (skill, file) {
                ("cadcraft-2d-drafting", "reference.md") => include_str!("../skills/cadcraft-2d-drafting/reference.md"),
                ("cadcraft-best-practices", "layers.md") => include_str!("../skills/cadcraft-best-practices/layers.md"),
                ("cadcraft-best-practices", "blocks.md") => include_str!("../skills/cadcraft-best-practices/blocks.md"),
                ("cadcraft-best-practices", "dims.md") => include_str!("../skills/cadcraft-best-practices/dims.md"),
                _ => "Skill or file not found. Available: cadcraft-2d-drafting (reference.md), cadcraft-best-practices (layers.md, blocks.md, dims.md)",
            };
            Ok(json!({"content": content}))
        },
        "list_commands" => b.call("engine.commands", json!({})).map(|v| {
            let f = a.get("filter").and_then(Value::as_str).map(str::to_ascii_lowercase);
            match (v, f) {
                (Value::Array(items), Some(f)) => Value::Array(items.into_iter().filter(|c| command_matches(c, &f)).collect()),
                (v, _) => v,
            }
        }),
        "inspect_drawing" => b.call("drawing.inspect", a),
        "query_entities" => b.call("engine.execute", json!({"command": "entities", "params": a})),
        "find_text" => b.call("engine.execute", json!({"command": "texts", "params": a})),
        "new_drawing" => b.call("engine.execute", json!({"command": "new", "params": a})),
        "open" => b.call("app.open", a),
        "save" => b.call("app.save", a),
        "cancel" => b.call("cmdline.key", json!({"key": "escape"})),
        "screenshot" => b.call("ui.screenshot", a),
        "ui_inspect" => b.call("ui.inspect", json!({})),
        "ui_click" => b.call("ui.pointer", a),
        "render" => {
            return Ok(match b.call("ui.render", a) {
                Ok(v) => match v.get("pngBase64").and_then(Value::as_str) {
                    Some(data) => json!({"content": [{"type": "image", "data": data, "mimeType": "image/png"}]}),
                    None => text(&v),
                },
                Err(e) => err_text(&e),
            });
        }
        other => return Err(format!("unknown tool `{other}`")),
    };
    Ok(match r {
        Ok(v) => text(&v),
        Err(e) => err_text(&e),
    })
}
