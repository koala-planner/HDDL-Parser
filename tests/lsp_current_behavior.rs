use serde_json::{Value, json};
use std::fs;
use std::io::{self, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

enum LspEvent {
    Message(Value),
    Eof,
    Error(String),
}

struct LspProcess {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<LspEvent>,
}

impl LspProcess {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_language_server"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("language_server subprocess should start");
        let stdin = child.stdin.take().expect("stdin should be piped");
        let mut stdout = child.stdout.take().expect("stdout should be piped");
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            loop {
                match read_lsp_message(&mut stdout) {
                    Ok(Some(message)) => {
                        if tx.send(LspEvent::Message(message)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => {
                        let _ = tx.send(LspEvent::Eof);
                        break;
                    }
                    Err(error) => {
                        let _ = tx.send(LspEvent::Error(error.to_string()));
                        break;
                    }
                }
            }
        });

        Self { child, stdin, rx }
    }

    fn initialize(&mut self) -> Value {
        self.request(
            1,
            "initialize",
            json!({
                "processId": null,
                "rootUri": null,
                "capabilities": {}
            }),
        )
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }));
        self.response(id)
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }));
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .expect("LSP message should be writable");
        self.stdin.flush().expect("LSP stdin should flush");
    }

    fn response(&mut self, id: u64) -> Value {
        loop {
            match self.rx.recv_timeout(Duration::from_secs(5)) {
                Ok(LspEvent::Message(message))
                    if message.get("id").and_then(Value::as_u64) == Some(id) =>
                {
                    return message;
                }
                Ok(LspEvent::Message(_notification)) => {}
                Ok(LspEvent::Eof) => panic!("language_server closed stdout before response {id}"),
                Ok(LspEvent::Error(error)) => {
                    panic!("failed to read language_server response {id}: {error}")
                }
                Err(error) => {
                    panic!("timed out waiting for language_server response {id}: {error}")
                }
            }
        }
    }

    fn shutdown(&mut self) {
        let _ = self.request(99, "shutdown", json!(null));
        self.notify("exit", json!(null));
    }
}

impl Drop for LspProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_lsp_message(reader: &mut impl Read) -> io::Result<Option<Value>> {
    let mut headers = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match reader.read_exact(&mut byte) {
            Ok(()) => {
                headers.push(byte[0]);
                if headers.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(error),
        }
    }

    let headers = String::from_utf8(headers)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?
        .parse::<usize>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

#[test]
fn lsp_initialize_current_behavior_reports_stale_server_version() {
    let mut server = LspProcess::start();
    let response = server.initialize();

    assert_eq!(response["result"]["serverInfo"]["version"], "0.1.0");
    assert_ne!(
        response["result"]["serverInfo"]["version"],
        env!("CARGO_PKG_VERSION")
    );

    server.shutdown();
}

#[test]
fn lsp_diagnostic_unsynced_document_current_behavior_returns_jsonrpc_error() {
    let mut server = LspProcess::start();
    server.initialize();
    server.notify("initialized", json!({}));

    let response = server.request(
        2,
        "textDocument/diagnostic",
        json!({
            "textDocument": {
                "uri": "file:///tmp/hddl-parser-current-behavior-unsynced.hddl"
            }
        }),
    );

    assert_eq!(response["error"]["code"], -32602);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("is not synced")
    );

    server.shutdown();
}

#[test]
fn lsp_diagnostic_problem_without_domain_current_behavior_returns_empty_report() {
    let mut server = LspProcess::start();
    server.initialize();
    server.notify("initialized", json!({}));

    let temp_dir = std::env::temp_dir().join(format!(
        "hddl-parser-lsp-current-behavior-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let problem_path = temp_dir.join("p01.hddl");
    let problem_text = fs::read_to_string("tests/ipc/Blocksworld-GTOHP/p01.hddl").unwrap();
    fs::write(&problem_path, &problem_text).unwrap();
    let uri = format!("file://{}", problem_path.display());

    server.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "hddl",
                "version": 1,
                "text": problem_text
            }
        }),
    );

    let response = server.request(
        3,
        "textDocument/diagnostic",
        json!({
            "textDocument": {
                "uri": uri
            }
        }),
    );

    assert_eq!(response["result"]["kind"], "full");
    assert_eq!(response["result"]["items"].as_array().unwrap().len(), 0);

    server.shutdown();
    fs::remove_dir_all(temp_dir).unwrap();
}
