//! Local LLM client (Ollama) with function calling for MCP tools.
//!
//! Falls back from the rule parser: if `nl::parse` returns None, we send the user's text
//! to the LLM with all MCP tool schemas; the model picks a tool and arguments.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_MODEL: &str = "qwen3:4b";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: OLLAMA_URL.into(),
            model: DEFAULT_MODEL.into(),
            temperature: 0.1,
        }
    }
}

/// A function the LLM can call. Mirrors the MCP tool schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

impl FunctionSpec {
    pub fn new(name: &str, description: &str, parameters: Value) -> Self {
        Self { name: name.into(), description: description.into(), parameters }
    }
}

/// Complete tool definition for the LLM (OpenAI-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub kind: String, // "function"
    pub function: FunctionSpec,
}

impl ToolDef {
    pub fn from_mcp_tool(name: &str, description: &str, params_schema: Value) -> Self {
        Self {
            kind: "function".into(),
            function: FunctionSpec::new(name, description, params_schema),
        }
    }
}

/// Response from /api/chat.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    #[serde(default)]
    thinking: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize)]
struct ToolCall {
    function: ToolCallFunction,
}

#[derive(Debug, Deserialize)]
struct ToolCallFunction {
    name: String,
    arguments: Value,
}

/// High-level client.
pub struct LlmClient {
    config: LlmConfig,
    http: reqwest::blocking::Client,
    /// All MCP tools as LLM-callable functions.
    tools: Vec<ToolDef>,
    /// name -> schema for validation/execution.
    tool_schemas: HashMap<String, Value>,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Result<Self, String> {
        let http = reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| format!("HTTP client: {e}"))?;
        let tools = mcp_tool_defs();
        let mut tool_schemas = HashMap::new();
        for t in &tools {
            tool_schemas.insert(t.function.name.clone(), t.function.parameters.clone());
        }
        Ok(Self { config, http, tools, tool_schemas })
    }

    /// Check if Ollama is reachable and the model exists.
    pub fn health_check(&self) -> Result<(), String> {
        let url = format!("{}/api/tags", self.config.base_url);
        let resp = self.http.get(&url).send().map_err(|e| format!("Ollama unreachable: {e}"))?;
        let tags: Value = resp.json().map_err(|e| format!("Tags parse: {e}"))?;
        let models = tags.get("models").and_then(Value::as_array).ok_or("no models")?;
        let has = models.iter().any(|m| m.get("name").and_then(Value::as_str) == Some(&self.config.model));
        if !has {
            return Err(format!("Model '{}' not found. Run: ollama pull {}", self.config.model, self.config.model));
        }
        Ok(())
    }

    /// Send a user message, get back either a tool call or a plain text response.
    pub fn chat(&self, user_text: &str) -> Result<LlmOutcome, String> {
        let system = r#"You are a CAD assistant for CADCraft. You have access to tools that map 1:1 to MCP tools.
When the user asks for something, pick the most appropriate tool and call it with the correct arguments.
If the user's request is ambiguous, ask for clarification instead of guessing.
All coordinates are in drawing units (mm by default). Y goes up."#;

        let body = json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user_text}
            ],
            "tools": self.tools,
            "tool_choice": "auto",
            "stream": false,
            "options": {"temperature": self.config.temperature}
        });

        let url = format!("{}/api/chat", self.config.base_url);
        let resp = self.http.post(&url).json(&body).send()
            .map_err(|e| format!("LLM request failed: {e}"))?;
        let chat: ChatResponse = resp.json().map_err(|e| format!("LLM response parse: {e}"))?;

        if let Some(calls) = chat.message.tool_calls {
            if let Some(call) = calls.first() {
                return Ok(LlmOutcome::ToolCall {
                    name: call.function.name.clone(),
                    arguments: call.function.arguments.clone(),
                });
            }
        }
        // Plain text response (clarification, error, etc.)
        Ok(LlmOutcome::Text(chat.message.content))
    }
}

#[derive(Debug, Clone)]
pub enum LlmOutcome {
    ToolCall { name: String, arguments: Value },
    Text(String),
}

/// All 20 MCP tools exposed as LLM functions.
fn mcp_tool_defs() -> Vec<ToolDef> {
    vec![
        // Core drawing
        ToolDef::from_mcp_tool("command_line", "Type one line at the CADCraft command line exactly like a user. Starts commands by name/alias (LINE, L, CIRCLE, C, PLINE, RECTANG, MOVE, TRIM, OFFSET, ZOOM…) and answers prompts: points x,y, relative @dx,dy, polar @dist<angle, distances, keywords (c for Close). Spaces act as Enter. Returns new prompt and output.", json!({
            "type": "object",
            "properties": {"text": {"type": "string"}},
            "required": ["text"]
        })),
        ToolDef::from_mcp_tool("execute", "Run any command with JSON parameters (no prompts). Use list_commands to find ids and parameter docs. Examples: line {points:[[0,0],[10,0]]}, circle {center:[0,0], radius:5}, layer.new {name:\"Walls\", color:\"red\", current:true}, move {handles:[\"10A\"], delta:[5,0]}, zoom {mode:\"extents\"}.", json!({
            "type": "object",
            "properties": {"command": {"type": "string"}, "params": {"type": "object"}},
            "required": ["command"]
        })),
        ToolDef::from_mcp_tool("batch", "Run multiple commands atomically with automatic rollback on failure. Each step: {command, params}. On error, completed steps are undone. Returns {completed, results, rolled_back, undone}.", json!({
            "type": "object",
            "properties": {"commands": {"type": "array", "items": {"type": "object", "properties": {"command": {"type": "string"}, "params": {"type": "object"}}, "required": ["command"]}}},
            "required": ["commands"]
        })),
        // Query
        ToolDef::from_mcp_tool("inspect_design", "Complete design snapshot: entities with geometry, layers, blocks, layouts, selection, undo history.", json!({
            "type": "object",
            "properties": {"entities": {"type": "boolean"}, "layers": {"type": "boolean"}, "blocks": {"type": "boolean"}, "views": {"type": "boolean"}, "limit": {"type": "integer"}},
        })),
        ToolDef::from_mcp_tool("inspect_drawing", "Summary of the active drawing: entity counts, extents, layers, styles, blocks, layouts, selection, undo history.", json!({
            "type": "object",
            "properties": {"entities": {"type": "boolean"}, "limit": {"type": "integer"}},
        })),
        ToolDef::from_mcp_tool("query_entities", "Entities filtered by type (Line, Circle, Polyline, Arc, Text, MText, Dimension, Hatch, Point, BlockRef, Polyface, Polygon, Spline, Ellipse, Leader, MLeader, Ray, XLine, Wipeout, Region, Body, Surface, Mesh, SubDMesh, Underlay, Light, Camera, Viewport, MInsertBlock, Attribute, AttributeDefinition, Table, MLine, MLineStyle, OleFrame, Ole2Frame, Helix, Sun, GeoMap, VisualStyle, Material, Section, Detail, Layout, PageSetup, PlotSettings, NamedView, Group, Dictionary, XRecord, Image, UnderlayDefinition, WipeoutVariable, TableStyle, MultilineStyle, VisualStyle, DetailViewStyle, SectionViewStyle, LightList, CameraList, MaterialList, SectionManager, PageSetupManager, PlotStyleTable, PlotStyleTableIterator, VisualStyleManager, MaterialManager, LightManager, CameraManager, SunProperties, GeoMapManager, RenderEnvironment, RenderSettings, Background, Fog, Sun, Sky, GroundPlane, VisualStyle, VisualStyle2, VisualStyle3), layer, handle range.", json!({
            "type": "object",
            "properties": {"type": {"type": "string"}, "layer": {"type": "string"}, "handles": {"type": "array", "items": {"type": "string"}}, "limit": {"type": "integer"}},
        })),
        ToolDef::from_mcp_tool("find_text", "Read the drawing's texts: TEXT, MTEXT (formatting removed), attribute values.", json!({
            "type": "object",
            "properties": {"pattern": {"type": "string"}, "layer": {"type": "string"}, "limit": {"type": "integer"}},
        })),
        ToolDef::from_mcp_tool("measure", "Measure distances, areas, perimeters, angles, and bounding boxes.", json!({
            "type": "object",
            "properties": {"kind": {"type": "string", "enum": ["dist", "area", "perimeter", "angle", "bbox"]}, "points": {"type": "array", "items": {"type": "array", "items": {"type": "number"}}}, "handles": {"type": "array", "items": {"type": "string"}}, "entity": {"type": "string"}},
            "required": ["kind"]
        })),
        // File
        ToolDef::from_mcp_tool("new_drawing", "Create a new drawing (imperial by default, or metric).", json!({
            "type": "object",
            "properties": {"metric": {"type": "boolean"}, "template": {"type": "string"}},
        })),
        ToolDef::from_mcp_tool("open", "Open a DXF file.", json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"]
        })),
        ToolDef::from_mcp_tool("save", "Save the drawing (DXF or DWG by extension). A .svg, .png or .pdf extension exports a rendering.", json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"]
        })),
        // UI / View
        ToolDef::from_mcp_tool("screenshot", "Screenshot the CADCraft window (PNG).", json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
        })),
        ToolDef::from_mcp_tool("render", "Render the active drawing to a PNG and return it as an image.", json!({
            "type": "object",
            "properties": {"width": {"type": "integer"}, "height": {"type": "integer"}, "fit": {"type": "boolean"}, "path": {"type": "string"}},
        })),
        ToolDef::from_mcp_tool("cancel", "Press Escape: cancel the running command (or clear the selection).", json!({"type": "object", "properties": {}})),
        // Meta
        ToolDef::from_mcp_tool("list_commands", "Every command: id, label, menu path, shortcut, aliases, params doc, enabled.", json!({"type": "object", "properties": {}})),
        ToolDef::from_mcp_tool("skill_reference", "Read a bundled agent skill document. Skills: cadcraft-2d-drafting.", json!({
            "type": "object",
            "properties": {"skill": {"type": "string"}, "file": {"type": "string"}},
        })),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_defs_have_required_fields() {
        let tools = mcp_tool_defs();
        assert_eq!(tools.len(), 20);
        for t in tools {
            assert!(!t.function.name.is_empty(), "missing name");
            assert!(!t.function.description.is_empty(), "missing description for {}", t.function.name);
            assert!(t.function.parameters.get("type").is_some(), "missing type for {}", t.function.name);
        }
    }
}