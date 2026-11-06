//! AI Chat Modal for MCP interaction within cadcraft.

use egui::{Color32, Context, Frame, ScrollArea, TextEdit, Ui, Window};
use serde_json::{Value, json};
use std::sync::Arc;
use std::thread;

use crate::llm::{LlmClient, LlmConfig, LlmOutcome};
use crate::mcp_client::{ConnectionMode, McpClient};
use crate::nl::{parse, Plan};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

pub struct AiChatModal {
    pub is_open: bool,
    messages: Vec<ChatMessage>,
    input: String,
    mcp_client: McpClient,
    /// Local LLM client for natural language understanding (Stage C).
    llm_client: Option<Arc<LlmClient>>,
    /// LLM request in flight (runs on background thread).
    llm_pending: Option<thread::JoinHandle<Result<LlmOutcome, String>>>,
    connection_mode: ConnectionMode,
    tcp_addr: String,
    /// The port this window's control server listens on, when it has one.
    control_port: Option<u16>,
    /// One MCP request in flight at a time; answered on a later frame, never by blocking.
    pending: Option<Pending>,
    /// Requests waiting for their turn, so a quick click never gets dropped.
    queue: std::collections::VecDeque<Queued>,
    scroll_to_bottom: bool,
}

struct Queued {
    label: String,
    kind: PendingKind,
    request: Value,
}

/// What to do with a response once it arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    /// A `tools/call` result: show its text.
    Tool,
    /// A `tools/list` result: show the tool names.
    ToolsList,
    /// Any other JSON-RPC request: show the raw result.
    Raw,
}

struct Pending {
    id: u64,
    kind: PendingKind,
    deadline: std::time::Instant,
}

impl Default for AiChatModal {
    fn default() -> Self {
        Self::new()
    }
}

impl AiChatModal {
    pub fn new() -> Self {
        // Initialize LLM client (non-blocking; health check happens on first use).
        let llm_client = LlmClient::new(LlmConfig::default()).ok().map(Arc::new);

        Self {
            is_open: false,
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: "1) 🔗 Connect  2) type a command or press a Quick button.\n\
                              e.g. `room`, `circle 0,0 10`, `line 0,0 100,0 100,60 c`, `inspect`\n\
                              Korean works too: `방`, `도면`, `새도면`."
                        .into(),
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                }
            ],
            input: String::new(),
            mcp_client: McpClient::new(),
            llm_client,
            llm_pending: None,
            connection_mode: ConnectionMode::Tcp,
            tcp_addr: "127.0.0.1:7979".into(),
            control_port: None,
            pending: None,
            queue: std::collections::VecDeque::new(),
            scroll_to_bottom: true,
        }
    }

    /// Called by the app once it knows its own control port. Defaulting to the bridged mode means
    /// a plain Connect draws into the window the user is looking at.
    pub fn set_control_port(&mut self, port: u16) {
        self.control_port = Some(port);
        self.tcp_addr = format!("127.0.0.1:{port}");
        self.connection_mode = ConnectionMode::Tcp;
    }

    pub fn show(&mut self, ctx: &Context) {
        self.mcp_client.poll();
        self.poll_pending();
        self.poll_llm();

        if !self.is_open { return; }

        // A pending answer only arrives while frames keep coming.
        if self.pending.is_some() || self.llm_pending.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Cap the message list so a long history can never push the input off the window.
        let width = 540.0_f32;

        let mut is_open = self.is_open;
        Window::new("🤖 AI Assistant (MCP)")
            .open(&mut is_open)
            .default_size([width, 480.0])
            .min_size([320.0, 260.0])
            .max_size([720.0, 640.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.render_connection_bar(ui);
                ui.separator();

                // The input area is drawn last, so reserve its space up front.
                let input_height = 92.0;
                let list_height = (ui.available_height() - input_height).clamp(80.0, 360.0);
                self.render_messages(ui, list_height);

                ui.separator();
                self.render_input(ui);
            });
        self.is_open = is_open;
    }

    fn render_connection_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("conn_mode")
                .width(150.0)
                .selected_text(match self.connection_mode {
                    ConnectionMode::Stdio => "📟 stdio (headless)",
                    ConnectionMode::Tcp => "🔌 TCP (this window)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.connection_mode, ConnectionMode::Stdio, "📟 stdio (headless)");
                    ui.selectable_value(&mut self.connection_mode, ConnectionMode::Tcp, "🔌 TCP (this window)");
                });

            if self.connection_mode == ConnectionMode::Tcp {
                let response = ui.add_sized(
                    [132.0, 20.0],
                    TextEdit::singleline(&mut self.tcp_addr).hint_text("host:port"),
                );
                crate::control::record_widget(ui, &response, "AI chat addr", "text", None);
            }

            let connected = self.mcp_client.is_connected();
            let btn_text = if connected { "🔌 Disconnect" } else { "🔗 Connect" };
            if chat_button(ui, btn_text) {
                if connected {
                    self.mcp_client.disconnect();
                    self.add_message("system", "Disconnected from MCP server.".to_string());
                } else {
                    self.connect();
                }
            }

            if connected {
                ui.colored_label(Color32::GREEN, "● Connected");
            } else {
                ui.colored_label(Color32::RED, "○ Offline");
            }
        });
    }

    fn connect(&mut self) {
        self.add_message("system", format!("DEBUG: connect() called, mode={:?}, control_port={:?}, tcp_addr={}", self.connection_mode, self.control_port, self.tcp_addr));
        
        if self.connection_mode == ConnectionMode::Tcp && self.control_port.is_none() {
            self.add_message(
                "error",
                "This window has no control server, so the bridge cannot reach it.\n\
                 Start cadcraft with `--control 7979`, or switch Mode to 📟 stdio (headless)."
                    .to_string(),
            );
            self.scroll_to_bottom = true;
            return;
        }

        let result = match self.connection_mode {
            ConnectionMode::Stdio => self.mcp_client.connect_stdio(),
            ConnectionMode::Tcp => self.mcp_client.connect_tcp(&self.tcp_addr),
        };

        self.add_message("system", format!("DEBUG: connect_tcp result: {:?}", result));

        match result {
            Ok(_) => {
                self.add_message("system", "Connected. This window is the drawing target.".to_string());
                self.send_list_tools();
            }
            Err(e) => {
                self.add_message("error", format!("Connection failed: {}", e));
            }
        }
    }

    fn render_messages(&mut self, ui: &mut Ui, max_height: f32) {
        let stick = self.scroll_to_bottom;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(max_height)
            .stick_to_bottom(stick)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                for msg in &self.messages {
                    self.render_message(ui, msg);
                }
            });
        if stick {
            self.scroll_to_bottom = false;
        }
    }

    fn render_message(&self, ui: &mut Ui, msg: &ChatMessage) {
        let (color, prefix) = match msg.role.as_str() {
            "user" => (Color32::LIGHT_BLUE, "👤 You"),
            "assistant" => (Color32::LIGHT_GREEN, "🤖 Assistant"),
            "system" => (Color32::GRAY, "ℹ️ System"),
            "error" => (Color32::RED, "❌ Error"),
            _ => (Color32::WHITE, "💬"),
        };

        // Long tool output must wrap, not widen the window.
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.horizontal(|ui| {
            ui.colored_label(color, format!("{} [{}]", prefix, msg.timestamp));
        });

        let is_json = msg.content.trim().starts_with('{') || msg.content.trim().starts_with('[');
        if is_json {
            Frame::new()
                .fill(ui.visuals().code_bg_color)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.monospace(&msg.content);
                });
        } else {
            ui.label(&msg.content);
        }
        ui.add_space(4.0);
    }

    fn render_input(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Quick:");
            if chat_button(ui, "📋 Tools") {
                self.send_list_tools();
            }
            if chat_button(ui, "🏠 New") {
                self.send_quick_command("execute", json!({"command": "new", "params": {"metric": true}}));
            }
            if chat_button(ui, "📐 Room 5x4m") {
                self.send_room_template();
            }
            if chat_button(ui, "🔍 Inspect") {
                self.send_quick_command("inspect_design", json!({}));
            }
            if chat_button(ui, "📖 Skill") {
                self.send_quick_command("skill_reference", json!({"skill": "cadcraft-2d-drafting", "file": "reference.md"}));
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // Leave room for the Send button; an infinite width would stretch the window.
            let send_width = 56.0;
            let input_width = (ui.available_width() - send_width).max(120.0);
            let response = ui.add(
                TextEdit::singleline(&mut self.input)
                    .hint_text("명령 입력  예: circle 0,0 10  /  room  /  방")
                    .desired_width(input_width)
            );
            crate::control::record_widget(ui, &response, "AI chat input", "text", None);

            let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if chat_button(ui, "Send") || enter_pressed {
                if !self.input.trim().is_empty() {
                    self.send_user_message();
                }
            }
        });
    }

    fn send_user_message(&mut self) {
        let text = self.input.trim().to_string();
        self.input.clear();

        if text.is_empty() { return; }

        self.add_message("user", text.clone());
        self.scroll_to_bottom = true;

        // DEBUG
        self.add_message("system", format!("DEBUG: received text='{}', lower='{}'", text, text.to_lowercase()));

        // Short words for the most common actions.
        match text.to_lowercase().as_str() {
            "room" | "방" => {
                self.add_message("system", "DEBUG: matched room/bang shortcut".to_string());
                self.send_room_template();
                return;
            }
            "inspect" | "도면" => {
                self.send_quick_command("inspect_design", json!({}));
                return;
            }
            "tools" | "툴" => {
                self.send_list_tools();
                return;
            }
            "skill" | "help" | "도움말" => {
                self.send_quick_command(
                    "skill_reference",
                    json!({"skill": "cadcraft-2d-drafting", "file": "reference.md"}),
                );
                return;
            }
            "new" | "새도면" => {
                self.send_quick_command("execute", json!({"command": "new", "params": {"metric": true}}));
                return;
            }
            _ => {}
        }

        if text.starts_with('{') {
            match serde_json::from_str::<Value>(&text) {
                Ok(req) => { self.send_raw_request(req); return; }
                Err(_) => {}
            }
        }

        // Try natural language parsing first (Stage B).
        if let Some(plan) = parse(&text) {
            self.add_message("system", format!("📋 {}", plan.summary));
            self.send_tool("batch…", "batch", plan.batch());
            return;
        }

        if let Some(space_pos) = text.find(' ') {
            let tool_name = &text[..space_pos];
            let args_str = &text[space_pos+1..];
            if let Ok(args) = serde_json::from_str::<Value>(args_str) {
                self.send_tool_call(tool_name, args);
                return;
            }
        }

        // Stage C: Local LLM with function calling.
        self.send_to_llm(text);
    }

    /// Send the user's text to the local LLM for tool selection.
    fn send_to_llm(&mut self, text: String) {
        let Some(client) = self.llm_client.clone() else {
            // LLM not available (Ollama not running or model missing) -> fall back to command line.
            self.add_message("system", "🤖 LLM unavailable, sending as command line…".to_string());
            self.send_tool_call("command_line", json!({"text": text}));
            return;
        };

        if self.llm_pending.is_some() {
            self.add_message("system", "⏳ LLM busy, queued…".to_string());
            return;
        }

        self.add_message("system", "🤖 Thinking…".to_string());
        self.scroll_to_bottom = true;

        let handle = thread::spawn(move || client.chat(&text));
        self.llm_pending = Some(handle);
    }

    /// Send one request and return immediately. The answer is picked up by `poll_pending` on a
    /// later frame, so the bridge can reach this window's control server and do the work.
    fn send_async(&mut self, label: &str, kind: PendingKind, request: Value) {
        if !self.mcp_client.is_connected() {
            self.add_message("error", "Not connected — press 🔗 Connect first.".to_string());
            self.scroll_to_bottom = true;
            return;
        }
        if self.pending.is_some() {
            self.add_message("system", format!("⏳ queued: {label}"));
            self.queue.push_back(Queued { label: label.to_string(), kind, request });
            self.scroll_to_bottom = true;
            return;
        }
        self.start_pending(label, kind, request);
    }

    fn start_pending(&mut self, label: &str, kind: PendingKind, request: Value) {
        match self.mcp_client.start_request(request) {
            Ok(id) => {
                self.add_message("system", format!("⏳ {label}"));
                self.pending = Some(Pending {
                    id,
                    kind,
                    deadline: std::time::Instant::now() + std::time::Duration::from_secs(20),
                });
            }
            Err(e) => self.add_message("error", format!("Could not send: {e}")),
        }
        self.scroll_to_bottom = true;
    }

    fn send_tool(&mut self, label: &str, tool_name: &str, args: Value) {
        let request = json!({ "method": "tools/call", "params": { "name": tool_name, "arguments": args } });
        self.send_async(label, PendingKind::Tool, request);
    }

    /// Collect the answer to the request in flight, if it has arrived.
    fn poll_pending(&mut self) {
        let Some(pending) = self.pending.take() else { return };

        let Some(outcome) = self.mcp_client.take_response(pending.id) else {
            if std::time::Instant::now() >= pending.deadline {
                self.add_message(
                    "error",
                    "No answer from the MCP server after 20s. Is the bridge still connected?".to_string(),
                );
                self.scroll_to_bottom = true;
            } else {
                self.pending = Some(pending);
            }
            return;
        };

        match outcome {
            Err(e) => self.add_message("error", format!("Request failed: {e}")),
            Ok(response) => {
                if let Some(err) = response.get("error") {
                    let text = err.get("message").and_then(Value::as_str).map(str::to_string)
                        .unwrap_or_else(|| err.to_string());
                    self.add_message("error", text);
                } else {
                    let result = response.get("result").cloned().unwrap_or(Value::Null);
                    match pending.kind {
                        PendingKind::Tool => self.add_message("assistant", format_result(&result)),
                        PendingKind::Raw => self.add_message("assistant", format_result(&result)),
                        PendingKind::ToolsList => {
                            let tools = result.get("tools").and_then(Value::as_array).cloned().unwrap_or_default();
                            let names: Vec<&str> = tools
                                .iter()
                                .filter_map(|t| t.get("name").and_then(Value::as_str))
                                .collect();
                            self.add_message(
                                "assistant",
                                format!("{} tools:\n{}", names.len(), names.join(", ")),
                            );
                        }
                    }
                }
            }
        }
        self.scroll_to_bottom = true;

        // Send whatever was queued while this request was in flight.
        if let Some(next) = self.queue.pop_front() {
            self.start_pending(&next.label, next.kind, next.request);
        }
    }

    /// Check if the LLM background thread has finished and handle its response.
    fn poll_llm(&mut self) {
        let Some(handle) = self.llm_pending.take() else { return };

        if handle.is_finished() {
            match handle.join() {
                Ok(Ok(LlmOutcome::ToolCall { name, arguments })) => {
                    self.add_message("system", format!("🤖 → {}", name));
                    self.send_tool(&format!("{name}…"), &name, arguments);
                }
                Ok(Ok(LlmOutcome::Text(text))) => {
                    self.add_message("assistant", text);
                    self.scroll_to_bottom = true;
                }
                Ok(Err(e)) => {
                    self.add_message("error", format!("LLM error: {}", e));
                    // Fall back to command line
                    self.send_tool_call("command_line", json!({"text": e}));
                    self.scroll_to_bottom = true;
                }
                Err(e) => {
                    self.add_message("error", format!("LLM thread panic: {:?}", e));
                    self.scroll_to_bottom = true;
                }
            }
        } else {
            // Still running, put it back.
            self.llm_pending = Some(handle);
        }
    }

    fn send_raw_request(&mut self, req: Value) {
        self.send_async("request", PendingKind::Raw, req);
    }

    fn send_tool_call(&mut self, tool_name: &str, args: Value) {
        self.send_tool(&format!("{tool_name}…"), tool_name, args);
    }

    fn send_quick_command(&mut self, tool_name: &str, args: Value) {
        self.add_message("user", format!("Quick: {tool_name}"));
        self.send_tool(&format!("{tool_name}…"), tool_name, args);
    }

    fn send_list_tools(&mut self) {
        self.send_async("tools/list", PendingKind::ToolsList, json!({ "method": "tools/list" }));
    }

    fn send_room_template(&mut self) {
        // A 5 x 4 m room with 200 mm walls, a door opening, a round table and dimensions.
        // The inner wall is drawn explicitly: `batch` steps cannot refer to the handle a previous
        // step returned, so an `offset` of "the rectangle just drawn" would be fragile.
        let batch = json!({
            "commands": [
                {"command": "new", "params": {"metric": true}},
                {"command": "layer.new", "params": {"name": "WALLS", "color": "red", "current": true}},
                {"command": "layer.new", "params": {"name": "DOORS", "color": "green"}},
                {"command": "layer.new", "params": {"name": "FURN", "color": "cyan"}},
                {"command": "layer.new", "params": {"name": "TEXT", "color": "white"}},
                {"command": "rectang", "params": {"p1": [0,0], "p2": [5000,4000]}},
                {"command": "rectang", "params": {"p1": [200,200], "p2": [4800,3800]}},
                {"command": "layer.set", "params": {"name": "DOORS"}},
                {"command": "rectang", "params": {"p1": [2000,0], "p2": [2900,2000]}},
                {"command": "layer.set", "params": {"name": "FURN"}},
                {"command": "circle", "params": {"center": [1000,2000], "radius": 450}},
                {"command": "layer.set", "params": {"name": "TEXT"}},
                {"command": "text", "params": {"at": [2500,2000], "text": "DOOR", "height": 200, "justify": "MC"}},
                {"command": "dimlinear", "params": {"p1": [0,0], "p2": [5000,0], "at": [2500,-500]}},
                {"command": "dimlinear", "params": {"p1": [0,0], "p2": [0,4000], "at": [-500,2000]}},
                {"command": "zoom", "params": {"mode": "extents"}}
            ]
        });

        self.send_quick_command("batch", batch);
    }

    fn add_message(&mut self, role: &str, content: String) {
        self.messages.push(ChatMessage {
            role: role.into(),
            content,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        });
        self.scroll_to_bottom = true;
    }
}
/// Tool results arrive as `content: [{type: "text", text: "..."}]`; show the text itself,
/// pretty-printed when it is JSON.
fn format_result(result: &Value) -> String {
    if let Some(items) = result.get("content").and_then(Value::as_array) {
        let texts: Vec<&str> = items
            .iter()
            .filter_map(|i| i.get("text").and_then(Value::as_str))
            .collect();
        if !texts.is_empty() {
            let joined = texts.join("\n");
            if let Ok(parsed) = serde_json::from_str::<Value>(&joined) {
                return serde_json::to_string_pretty(&parsed).unwrap_or(joined);
            }
            return joined;
        }
    }
    serde_json::to_string_pretty(result).unwrap_or_default()
}

/// A button the control server can observe (`ui.elements`), so the chat can be driven and
/// checked from outside the window.
fn chat_button(ui: &mut Ui, text: &str) -> bool {
    let response = ui.button(text);
    crate::control::record_widget(ui, &response, text, "button", None);
    response.clicked()
}
