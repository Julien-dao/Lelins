//! Ollama HTTP implementation of [`LlmProvider`].
//!
//! Targets the Ollama 0.23.x API surface:
//! - `POST /api/chat` with `stream: true` returns NDJSON.
//! - `POST /api/embeddings` returns `{ "embedding": [...] }`.
//! - `GET /api/version` for the health-check.

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::{ChatMessage, EmbedResponse, GenerateOptions, GenerateRequest, LlmError, Token};
use crate::LlmProvider;

/// HTTP client that talks to a local Ollama daemon.
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    base_url: String,
    client: Client,
}

impl OllamaProvider {
    /// Build a provider pointing at `base_url` (e.g. `http://localhost:11434`).
    /// Trailing slashes are normalized.
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        while url.ends_with('/') {
            url.pop();
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            // Streaming endpoints can run for minutes during long generations;
            // disable the per-request timeout for those at call sites if needed.
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .build()
            .expect("reqwest client builder is infallible with default tls");
        Self {
            base_url: url,
            client,
        }
    }

    /// Build with the conventional ANDREA default endpoint.
    pub fn local_default() -> Self {
        Self::new("http://localhost:11434")
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

#[derive(Serialize)]
struct ChatRequestBody<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
    options: ChatOptionsBody<'a>,
}

#[derive(Serialize)]
struct ChatOptionsBody<'a> {
    temperature: f32,
    top_p: f32,
    repeat_penalty: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    stop: &'a [String],
}

impl<'a> From<&'a GenerateOptions> for ChatOptionsBody<'a> {
    fn from(o: &'a GenerateOptions) -> Self {
        Self {
            temperature: o.temperature,
            top_p: o.top_p,
            repeat_penalty: o.repeat_penalty,
            num_predict: o.num_predict,
            num_ctx: o.num_ctx,
            stop: &o.stop,
        }
    }
}

#[derive(Deserialize)]
struct ChatStreamFrame {
    #[serde(default)]
    message: Option<ChatStreamMessage>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    total_duration: Option<u64>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct ChatStreamMessage {
    #[serde(default)]
    content: String,
}

#[derive(Serialize)]
struct EmbeddingsRequestBody<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<BoxStream<'static, Result<Token, LlmError>>, LlmError> {
        let body = ChatRequestBody {
            model: &request.model,
            messages: &request.messages,
            stream: true,
            options: ChatOptionsBody::from(&request.options),
        };
        let url = self.endpoint("/api/chat");
        let resp = self.client.post(url).json(&body).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::Http {
                status: status.as_u16(),
                body: truncate_body(&text),
            });
        }

        let byte_stream = resp.bytes_stream();
        Ok(parse_ndjson_stream(byte_stream).boxed())
    }

    async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>, LlmError> {
        let url = self.endpoint("/api/embeddings");
        let body = EmbeddingsRequestBody {
            model,
            prompt: text,
        };
        let resp = self.client.post(url).json(&body).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::Http {
                status: status.as_u16(),
                body: truncate_body(&text),
            });
        }
        let parsed: EmbedResponse = resp.json().await?;
        Ok(parsed.embedding)
    }

    async fn health(&self) -> Result<(), LlmError> {
        let url = self.endpoint("/api/version");
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(LlmError::Http {
                status: resp.status().as_u16(),
                body: String::new(),
            });
        }
        Ok(())
    }
}

fn truncate_body(s: &str) -> String {
    const MAX: usize = 500;
    if s.len() <= MAX {
        s.to_string()
    } else {
        format!("{}…", &s[..MAX])
    }
}

/// Parse Ollama's NDJSON streaming response into a stream of [`Token`]s.
///
/// Each line is a JSON object. Frames may straddle byte chunks, so we keep
/// a small line buffer that flushes on every `\n`.
fn parse_ndjson_stream<S>(
    byte_stream: S,
) -> impl futures_util::Stream<Item = Result<Token, LlmError>>
where
    S: futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
{
    use futures_util::stream::unfold;

    enum State<S> {
        Streaming { stream: S, buf: String },
        Drained,
    }

    unfold(
        State::Streaming {
            stream: Box::pin(byte_stream),
            buf: String::new(),
        },
        |state| async {
            match state {
                State::Drained => None,
                State::Streaming {
                    mut stream,
                    mut buf,
                } => loop {
                    if let Some(idx) = buf.find('\n') {
                        let line = buf[..idx].to_string();
                        buf.drain(..=idx);
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<ChatStreamFrame>(trimmed) {
                            Ok(frame) => {
                                if let Some(err) = frame.error {
                                    return Some((Err(LlmError::Protocol(err)), State::Drained));
                                }
                                let content = frame.message.map(|m| m.content).unwrap_or_default();
                                let token = Token {
                                    content,
                                    done: frame.done,
                                    total_duration_ns: frame.total_duration,
                                };
                                let next = if token.done {
                                    State::Drained
                                } else {
                                    State::Streaming { stream, buf }
                                };
                                return Some((Ok(token), next));
                            }
                            Err(e) => {
                                return Some((
                                    Err(LlmError::Protocol(format!(
                                        "bad NDJSON frame: {e} (line: {trimmed:.120})"
                                    ))),
                                    State::Drained,
                                ));
                            }
                        }
                    }

                    match stream.next().await {
                        None => {
                            if buf.trim().is_empty() {
                                return None;
                            }
                            // Flush trailing partial line as best-effort.
                            let trailing = std::mem::take(&mut buf);
                            match serde_json::from_str::<ChatStreamFrame>(trailing.trim()) {
                                Ok(frame) => {
                                    let token = Token {
                                        content: frame
                                            .message
                                            .map(|m| m.content)
                                            .unwrap_or_default(),
                                        done: true,
                                        total_duration_ns: frame.total_duration,
                                    };
                                    return Some((Ok(token), State::Drained));
                                }
                                Err(e) => {
                                    return Some((
                                        Err(LlmError::Protocol(format!(
                                            "trailing partial frame: {e}"
                                        ))),
                                        State::Drained,
                                    ));
                                }
                            }
                        }
                        Some(Err(e)) => {
                            return Some((Err(LlmError::from(e)), State::Drained));
                        }
                        Some(Ok(bytes)) => match std::str::from_utf8(&bytes) {
                            Ok(s) => buf.push_str(s),
                            Err(e) => {
                                return Some((
                                    Err(LlmError::Protocol(format!("invalid UTF-8: {e}"))),
                                    State::Drained,
                                ));
                            }
                        },
                    }
                },
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChatMessage, GenerateRequest};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn ndjson_body() -> String {
        // A typical Ollama /api/chat stream: 3 partial frames + a done frame.
        [
            r#"{"model":"mistral-small3.2:24b","message":{"role":"assistant","content":"Bonjour"},"done":false}"#,
            r#"{"model":"mistral-small3.2:24b","message":{"role":"assistant","content":" Marie"},"done":false}"#,
            r#"{"model":"mistral-small3.2:24b","message":{"role":"assistant","content":" !"},"done":false}"#,
            r#"{"model":"mistral-small3.2:24b","message":{"role":"assistant","content":""},"done":true,"total_duration":1234567}"#,
        ]
        .join("\n")
    }

    #[tokio::test]
    async fn generate_collects_streamed_chunks() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/x-ndjson")
                    .set_body_string(ndjson_body()),
            )
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri());
        let request = GenerateRequest::new(
            "mistral-small3.2:24b",
            vec![ChatMessage::user("Bonjour ANDREA")],
        );
        let text = provider.generate(request).await.unwrap();
        assert_eq!(text, "Bonjour Marie !");
    }

    #[tokio::test]
    async fn generate_stream_emits_done_frame() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/x-ndjson")
                    .set_body_string(ndjson_body()),
            )
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri());
        let request = GenerateRequest::new("mistral-small3.2:24b", vec![ChatMessage::user("Hi")]);
        let mut stream = provider.generate_stream(request).await.unwrap();

        let mut tokens = Vec::new();
        while let Some(token) = stream.next().await {
            tokens.push(token.unwrap());
        }
        assert_eq!(tokens.len(), 4);
        assert!(!tokens[0].done);
        assert!(!tokens[1].done);
        assert!(!tokens[2].done);
        assert!(tokens[3].done);
        assert_eq!(tokens[3].total_duration_ns, Some(1234567));
    }

    #[tokio::test]
    async fn generate_returns_unavailable_on_connection_refused() {
        let provider = OllamaProvider::new("http://127.0.0.1:1");
        let request = GenerateRequest::new("any", vec![ChatMessage::user("hi")]);
        let err = provider.generate(request).await.unwrap_err();
        assert!(matches!(err, LlmError::Unavailable(_)));
    }

    #[tokio::test]
    async fn http_error_is_surfaced() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(500).set_body_string("model not found"))
            .mount(&server)
            .await;

        let provider = OllamaProvider::new(server.uri());
        let request = GenerateRequest::new("missing", vec![ChatMessage::user("hi")]);
        let err = provider.generate(request).await.unwrap_err();
        assert!(matches!(err, LlmError::Http { status: 500, .. }));
    }

    #[tokio::test]
    async fn embed_parses_vector() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/embeddings"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "embedding": [0.1, 0.2, 0.3] })),
            )
            .mount(&server)
            .await;
        let provider = OllamaProvider::new(server.uri());
        let v = provider.embed("bge-m3", "bonjour").await.unwrap();
        assert_eq!(v, vec![0.1, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn health_ok() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "version": "0.23.1" })),
            )
            .mount(&server)
            .await;
        let provider = OllamaProvider::new(server.uri());
        provider.health().await.unwrap();
    }

    #[tokio::test]
    async fn malformed_ndjson_yields_protocol_error() {
        let server = MockServer::start().await;
        let body = "not even close to json\n";
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
        let provider = OllamaProvider::new(server.uri());
        let req = GenerateRequest::new("x", vec![ChatMessage::user("hi")]);
        let err = provider.generate(req).await.unwrap_err();
        assert!(matches!(err, LlmError::Protocol(_)), "got {err:?}");
    }

    #[test]
    fn base_url_strips_trailing_slashes() {
        let p = OllamaProvider::new("http://localhost:11434/");
        assert_eq!(p.endpoint("/api/chat"), "http://localhost:11434/api/chat");
        let p = OllamaProvider::new("http://localhost:11434///");
        assert_eq!(p.endpoint("/api/chat"), "http://localhost:11434/api/chat");
    }
}
