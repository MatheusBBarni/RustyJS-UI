#[path = "../src/runtime/fetch.rs"]
mod fetch_impl;

use fetch_impl::{FetchCompletion, FetchMethod, FetchRequest, FetchTransport};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn fetch_transport_resolves_get_response_from_local_server() {
    let server = spawn_server("HTTP/1.1 200 OK", "hello world");
    let transport = FetchTransport::new().unwrap();
    let request_id = transport
        .submit(
            FetchRequest::new(format!("{}/status", server.base_url)).with_method(FetchMethod::Get),
        )
        .unwrap();

    let completion = wait_for_completion(&transport);
    let request = server
        .request_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();

    assert_eq!(request_id, 1);
    assert!(request.starts_with("GET /status HTTP/1.1"));
    assert!(request.to_ascii_lowercase().contains("host: 127.0.0.1"));

    match completion {
        FetchCompletion::Response(response) => {
            assert_eq!(response.request_id, request_id);
            assert_eq!(response.status, 200);
            assert!(response.status_text.contains("OK"));
            assert_eq!(response.body, "hello world");
            assert!(response
                .headers
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("content-length") && value == "11"));
            assert!(response
                .headers
                .iter()
                .any(|(name, value)| name.eq_ignore_ascii_case("content-type")
                    && value == "text/plain"));
        }
        FetchCompletion::Error(error) => panic!("unexpected fetch error: {error:?}"),
    }
}

#[test]
fn fetch_transport_sends_post_body_and_headers_to_local_server() {
    let server = spawn_server("HTTP/1.1 204 No Content", "");
    let transport = FetchTransport::new().unwrap();
    let request = FetchRequest::new(format!("{}/submit", server.base_url))
        .with_method(FetchMethod::Post)
        .with_header("X-Trace-Id", "abc123")
        .with_body(r#"{"name":"Ada"}"#);

    let request_id = transport.submit(request).unwrap();
    let completion = wait_for_completion(&transport);
    let raw_request = server
        .request_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();

    assert_eq!(request_id, 1);
    let lower = raw_request.to_ascii_lowercase();
    assert!(lower.starts_with("post /submit http/1.1"));
    assert!(lower.contains("x-trace-id: abc123"));
    assert!(raw_request.contains(r#"{"name":"Ada"}"#));

    match completion {
        FetchCompletion::Response(response) => {
            assert_eq!(response.request_id, request_id);
            assert_eq!(response.status, 204);
            assert!(response.status_text.contains("No Content"));
            assert_eq!(response.body, "");
        }
        FetchCompletion::Error(error) => panic!("unexpected fetch error: {error:?}"),
    }
}

#[test]
fn fetch_request_builder_supports_put_delete_and_timeout() {
    let put_request = FetchRequest::new("http://127.0.0.1")
        .with_method(FetchMethod::Put)
        .with_timeout(Duration::from_secs(1));
    let delete_request = FetchRequest::new("http://127.0.0.1").with_method(FetchMethod::Delete);

    assert!(matches!(put_request.method, FetchMethod::Put));
    assert_eq!(put_request.timeout, Some(Duration::from_secs(1)));
    assert!(matches!(delete_request.method, FetchMethod::Delete));
}

struct ServerFixture {
    base_url: String,
    request_rx: mpsc::Receiver<String>,
}

fn spawn_server(status_line: &'static str, body: &'static str) -> ServerFixture {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (request_tx, request_rx) = mpsc::channel();
    let body = body.to_string();
    let status_line = status_line.to_string();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_http_request(&mut stream).unwrap();
        let _ = request_tx.send(request);

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        let _ = stream.shutdown(Shutdown::Both);
    });

    ServerFixture {
        base_url: format!("http://{addr}"),
        request_rx,
    }
}

fn read_http_request(stream: &mut TcpStream) -> std::io::Result<String> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;

    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];

    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);

        if let Some(header_end) = find_header_end(&buffer) {
            let content_length = parse_content_length(&buffer[..header_end])?;
            let total_len = header_end + 4 + content_length;

            while buffer.len() < total_len {
                let read = stream.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
            }

            break;
        }
    }

    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &[u8]) -> std::io::Result<usize> {
    let headers = String::from_utf8_lossy(headers);

    for line in headers.lines() {
        let mut parts = line.splitn(2, ':');
        let name = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim();

        if name.eq_ignore_ascii_case("content-length") {
            return value.parse::<usize>().map_err(|error| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
            });
        }
    }

    Ok(0)
}

fn wait_for_completion(transport: &FetchTransport) -> FetchCompletion {
    for _ in 0..200 {
        if let Some(completion) = transport.drain_completions().into_iter().next() {
            return completion;
        }

        thread::sleep(Duration::from_millis(10));
    }

    panic!("timed out waiting for fetch completion");
}
