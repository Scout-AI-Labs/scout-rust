//! Server-Sent Events streaming for chat completions and run events.

use crate::client::Json;
use crate::error::Error;

/// A live SSE stream that decodes each event's JSON payload.
///
/// ```no_run
/// # async fn run(client: scout_sdk::Client) -> Result<(), scout_sdk::Error> {
/// let mut stream = client.chat().completions().create_stream(Default::default()).await?;
/// while let Some(chunk) = stream.next().await {
///     let chunk = chunk?;
///     // read chunk["choices"][0]["delta"]["content"]
/// }
/// # Ok(()) }
/// ```
pub struct Stream {
    resp: reqwest::Response,
    buffer: String,
}

impl Stream {
    pub(crate) fn from_response(resp: reqwest::Response) -> Self {
        Stream {
            resp,
            buffer: String::new(),
        }
    }

    /// Return the next decoded event, `None` at end of stream (or on `[DONE]`).
    #[allow(clippy::should_implement_trait)]
    pub async fn next(&mut self) -> Option<Result<Json, Error>> {
        loop {
            if let Some(idx) = self.buffer.find("\n\n") {
                let block: String = self.buffer.drain(..idx + 2).collect();
                match parse_block(&block) {
                    Some(data) if data == "[DONE]" => return None,
                    Some(data) => {
                        return Some(
                            serde_json::from_str(&data).map_err(|e| Error::Decode(e.to_string())),
                        )
                    }
                    None => continue,
                }
            }
            match self.resp.chunk().await {
                Ok(Some(bytes)) => {
                    let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
                    self.buffer.push_str(&text);
                }
                Ok(None) => {
                    if !self.buffer.is_empty() {
                        let block = std::mem::take(&mut self.buffer);
                        if let Some(data) = parse_block(&block) {
                            if data == "[DONE]" {
                                return None;
                            }
                            return Some(
                                serde_json::from_str(&data)
                                    .map_err(|e| Error::Decode(e.to_string())),
                            );
                        }
                    }
                    return None;
                }
                Err(e) => return Some(Err(Error::Connection(e))),
            }
        }
    }
}

/// Extract the joined `data:` payload from one SSE block (ignoring comments).
fn parse_block(raw: &str) -> Option<String> {
    let mut data: Vec<&str> = Vec::new();
    for line in raw.split('\n') {
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("data:") {
            data.push(rest.strip_prefix(' ').unwrap_or(rest));
        }
    }
    if data.is_empty() {
        None
    } else {
        Some(data.join("\n"))
    }
}
