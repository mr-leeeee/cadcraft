//! Minimal MCP client for the in-app AI Assistant.
//!
//! Speaks JSON-RPC 2.0 over the stdio of a child `cadcraft-cli mcp` process. The same transport
//! serves both modes: a headless server (`cadcraft-cli mcp`) or a bridge to a running app
//! (`cadcraft-cli mcp --connect 127.0.0.1:7979`).

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const PROTOCOL_VERSION: &str = "2025-06-18";
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// A message read from the server's stdout.
pub enum McpMessage {
    Response(Value),
    Error(String),
    Disconnected,
}

/// Where the assistant's MCP server comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    /// A private headless server process; drawings live in that process, not in this window.
    Stdio,
    /// A bridge to this window's control server, so drawings appear on screen.
    Tcp,
}

pub struct McpClient {
    tx: Option<Sender<String>>,
    rx: Option<Receiver<McpMessage>>,
    child: Option<Child>,
    next_id: u64,
    /// Responses that arrived but have not been collected by [`McpClient::take_response`].
    received: HashMap<u64, Value>,
    /// The last transport-level failure, handed out by the next response lookup.
    last_error: Option<String>,
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            tx: None,
            rx: None,
            child: None,
            next_id: 1,
            received: HashMap::new(),
            last_error: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.tx.is_some() && self.rx.is_some()
    }

    /// Connect to a private headless MCP server (`cadcraft-cli mcp`).
    pub fn connect_stdio(&mut self) -> Result<(), String> {
        self.connect_child(&[])
    }

    /// Connect through the CLI bridge to a running app's control server.
    pub fn connect_tcp(&mut self, addr: &str) -> Result<(), String> {
        eprintln!("[MCP] connect_tcp called with addr: {}", addr);
        let result = self.connect_child(&["--connect", addr]);
        eprintln!("[MCP] connect_tcp result: {:?}", result);
        result
    }

    fn connect_child(&mut self, extra_args: &[&str]) -> Result<(), String> {
        eprintln!("[MCP] connect_child called with args: {:?}", extra_args);
        self.disconnect();

        let mut child = spawn_cli(extra_args)?;
        eprintln!("[MCP] spawn_cli succeeded");
        let stdin = child.stdin.take().ok_or("cadcraft-cli has no stdin")?;
        let stdout = child.stdout.take().ok_or("cadcraft-cli has no stdout")?;

        let (tx_in, rx_in) = mpsc::channel::<String>();
        let (tx_out, rx_out) = mpsc::channel::<McpMessage>();

        // stdin writer: one JSON-RPC message per line.
        let mut stdin = stdin;
        thread::spawn(move || {
            eprintln!("[MCP] stdin writer thread started");
            while let Ok(line) = rx_in.recv() {
                if stdin.write_all(line.as_bytes()).is_err()
                    || stdin.write_all(b"\n").is_err()
                    || stdin.flush().is_err()
                {
                    eprintln!("[MCP] stdin writer: write failed, breaking");
                    break;
                }
            }
            eprintln!("[MCP] stdin writer thread ended");
        });

        // stdout reader.
        let out = tx_out.clone();
        thread::spawn(move || {
            eprintln!("[MCP] stdout reader thread started");
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("[MCP] stdout reader: read error: {}", e);
                        let _ = out.send(McpMessage::Error(format!("read error: {e}")));
                        break;
                    }
                };
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                eprintln!("[MCP] stdout reader: got line: {}", line);
                match serde_json::from_str::<Value>(line) {
                    Ok(v) => {
                        let _ = out.send(McpMessage::Response(v));
                    }
                    Err(e) => {
                        eprintln!("[MCP] stdout reader: bad JSON: {}", e);
                        let _ = out.send(McpMessage::Error(format!("bad JSON from server: {e}")));
                    }
                }
            }
            eprintln!("[MCP] stdout reader: ended, sending Disconnected");
            let _ = out.send(McpMessage::Disconnected);
        });

        self.tx = Some(tx_in);
        self.rx = Some(rx_out);
        self.child = Some(child);

        eprintln!("[MCP] sending initialize request");
        let init = self.send_request(
            json!({
                "method": "initialize",
                "params": { "protocolVersion": PROTOCOL_VERSION, "capabilities": {}, "clientInfo": { "name": "cadcraft-ui", "version": env!("CARGO_PKG_VERSION") } }
            }),
            15,
        );
        eprintln!("[MCP] initialize result: {:?}", init);
        if let Err(e) = init {
            self.disconnect();
            return Err(e);
        }
        self.send_notification(json!({ "method": "notifications/initialized" }));
        eprintln!("[MCP] connect_child succeeded");
        Ok(())
    }

    /// Stop the server and drop the connection. Safe to call when not connected.
    pub fn disconnect(&mut self) {
        self.tx = None;
        self.rx = None;
        self.received.clear();
        self.last_error = None;
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// Send a raw JSON-RPC request and wait for its response.
    pub fn send_request(&mut self, request: Value, timeout_secs: u64) -> Result<Value, String> {
        let id = self.start_request(request)?;
        self.wait_for(id, timeout_secs)
    }

    /// Send a request without waiting: returns its id, for [`McpClient::take_response`].
    ///
    /// The assistant must use this. A tool call runs on the UI thread, and in bridged mode the
    /// bridge can only answer once this window draws another frame — waiting here would deadlock.
    pub fn start_request(&mut self, request: Value) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;

        let mut req = request;
        if let Value::Object(map) = &mut req {
            map.insert("jsonrpc".into(), json!("2.0"));
            map.insert("id".into(), json!(id));
        }
        let line = req.to_string();
        self.tx
            .as_ref()
            .ok_or("Not connected to the MCP server")?
            .send(line)
            .map_err(|_| "MCP server is gone".to_string())?;
        Ok(id)
    }

    fn send_notification(&mut self, notification: Value) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(notification.to_string());
        }
    }

    /// Collect a response if it has arrived. Never blocks, so it is safe inside a frame.
    pub fn take_response(&mut self, id: u64) -> Option<Result<Value, String>> {
        self.poll();
        if let Some(v) = self.received.remove(&id) {
            return Some(Ok(v));
        }
        if let Some(e) = self.last_error.take() {
            return Some(Err(e));
        }
        None
    }

    fn wait_for(&mut self, id: u64, timeout_secs: u64) -> Result<Value, String> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        loop {
            if let Some(result) = self.take_response(id) {
                return result;
            }
            if Instant::now() >= deadline {
                return Err(format!("MCP request timed out after {timeout_secs}s"));
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    /// Call an MCP tool and return its `result` payload.
    pub fn call_tool(&mut self, tool_name: &str, args: Value, timeout_secs: u64) -> Result<Value, String> {
        let response = self.send_request(
            json!({ "method": "tools/call", "params": { "name": tool_name, "arguments": args } }),
            timeout_secs,
        )?;
        result_of(response)
    }

    /// The server's tool list.
    pub fn list_tools(&mut self) -> Result<Vec<Value>, String> {
        let response = self.send_request(json!({ "method": "tools/list" }), 10)?;
        let result = result_of(response)?;
        Ok(result
            .get("tools")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default())
    }

    /// Move every message that has arrived into `received`, so nothing is lost between frames.
    pub fn poll(&mut self) {
        let mut disconnected = false;
        if let Some(rx) = self.rx.as_ref() {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    McpMessage::Response(v) => {
                        if let Some(rid) = v.get("id").and_then(Value::as_u64) {
                            self.received.insert(rid, v);
                        }
                    }
                    McpMessage::Error(e) => {
                        log::debug!("MCP: {e}");
                        self.last_error = Some(e);
                    }
                    McpMessage::Disconnected => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }
        if disconnected {
            self.tx = None;
            self.rx = None;
            self.last_error = Some("MCP server disconnected".into());
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

fn result_of(response: Value) -> Result<Value, String> {
    if let Some(err) = response.get("error") {
        return Err(err.to_string());
    }
    Ok(response.get("result").cloned().unwrap_or(Value::Null))
}

/// Prefer the `cadcraft-cli` binary built next to this executable; fall back to `cargo run`.
fn spawn_cli(extra_args: &[&str]) -> Result<Child, String> {
    let mut errors = Vec::new();

    if let Some(sibling) = cli_sibling() {
        match Command::new(&sibling)
            .arg("mcp")
            .args(extra_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => return Ok(child),
            Err(e) => errors.push(format!("{}: {e}", sibling.display())),
        }
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    match Command::new("cargo")
        .args(["run", "--quiet", "--release", "-p", "cadcraft-cli", "--", "mcp"])
        .args(extra_args)
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => Ok(child),
        Err(e) => {
            errors.push(format!("cargo run -p cadcraft-cli: {e}"));
            Err(format!("could not start cadcraft-cli ({})", errors.join("; ")))
        }
    }
}

fn cli_sibling() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) {
        "cadcraft-cli.exe"
    } else {
        "cadcraft-cli"
    };
    let path = dir.join(name);
    path.is_file().then_some(path)
}
