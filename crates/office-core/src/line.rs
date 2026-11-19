//! Parser for one transcript JSONL line. Typed structs with only the fields we
//! need, so unknown fields (and huge `content` payloads) are skipped by serde.
use serde::de::{self, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use std::marker::PhantomData;

pub const MAX_STR: usize = 200;

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnInfo {
    pub subagent_type: String,
    pub description: String,
    pub background: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineEvent {
    ToolUse {
        id: String,
        name: String,
        target: Option<String>,
        msg_id: Option<String>,
        ts: i64,
        spawn: Option<SpawnInfo>,
    },
    AssistantMsg {
        msg_id: Option<String>,
        usage: Option<Usage>,
        stop_reason: Option<String>,
        model: Option<String>,
        attribution_agent: Option<String>,
        ts: i64,
    },
    ToolResult {
        tool_use_id: String,
        is_error: bool,
        ts: i64,
    },
    UserPrompt {
        ts: i64,
    },
    TurnEnd {
        ts: i64,
    },
    ApiError {
        ts: i64,
    },
    Title(String),
    AgentName(String),
    LastPrompt(String),
    Context {
        cwd: Option<String>,
        git_branch: Option<String>,
    },
}

#[derive(Debug)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error: {}", self.0)
    }
}
impl std::error::Error for ParseError {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParseStats {
    pub malformed: u64,
    pub oversized: u64,
}

/// Parse and count malformed lines instead of returning the error.
pub fn parse_into(bytes: &[u8], stats: &mut ParseStats) -> Vec<LineEvent> {
    match parse_line(bytes) {
        Ok(v) => v,
        Err(_) => {
            stats.malformed += 1;
            Vec::new()
        }
    }
}

// ---- lenient deserialization helpers -------------------------------------

/// Deserializes a map as `T`; any other JSON shape becomes `None`.
struct Lenient<T>(Option<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Lenient<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = Lenient<T>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any json value")
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                T::deserialize(de::value::MapAccessDeserializer::new(map)).map(|t| Lenient(Some(t)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while seq.next_element::<IgnoredAny>()?.is_some() {}
                Ok(Lenient(None))
            }
            fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
            fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
            fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
            fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
            fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Lenient(None))
            }
        }
        d.deserialize_any(V(PhantomData))
    }
}

enum Content {
    Text,
    Blocks(Vec<Lenient<Block>>),
    Other,
}

impl<'de> Deserialize<'de> for Content {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Content;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("string or array")
            }
            fn visit_str<E>(self, _: &str) -> Result<Content, E> {
                Ok(Content::Text)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Content, A::Error> {
                let mut v = Vec::new();
                while let Some(b) = seq.next_element::<Lenient<Block>>()? {
                    v.push(b);
                }
                Ok(Content::Blocks(v))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Content, A::Error> {
                while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
                Ok(Content::Other)
            }
            fn visit_unit<E>(self) -> Result<Content, E> {
                Ok(Content::Other)
            }
            fn visit_bool<E>(self, _: bool) -> Result<Content, E> {
                Ok(Content::Other)
            }
            fn visit_u64<E>(self, _: u64) -> Result<Content, E> {
                Ok(Content::Other)
            }
            fn visit_i64<E>(self, _: i64) -> Result<Content, E> {
                Ok(Content::Other)
            }
            fn visit_f64<E>(self, _: f64) -> Result<Content, E> {
                Ok(Content::Other)
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Deserialize)]
struct Block {
    #[serde(rename = "type")]
    ty: Option<String>,
    id: Option<String>,
    name: Option<String>,
    input: Option<Lenient<Input>>,
    tool_use_id: Option<String>,
    is_error: Option<bool>,
}

#[derive(Deserialize, Default)]
struct Input {
    file_path: Option<Value>,
    notebook_path: Option<Value>,
    path: Option<Value>,
    pattern: Option<Value>,
    command: Option<Value>,
    url: Option<Value>,
    query: Option<Value>,
    subagent_type: Option<Value>,
    description: Option<Value>,
    run_in_background: Option<Value>,
}

#[derive(Deserialize)]
struct UsageRaw {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct Msg {
    id: Option<String>,
    model: Option<String>,
    stop_reason: Option<String>,
    usage: Option<Lenient<UsageRaw>>,
    content: Option<Content>,
}

#[derive(Deserialize)]
struct Raw {
    #[serde(rename = "type")]
    ty: Option<String>,
    subtype: Option<String>,
    timestamp: Option<String>,
    message: Option<Lenient<Msg>>,
    cwd: Option<String>,
    #[serde(rename = "gitBranch")]
    git_branch: Option<String>,
    #[serde(rename = "attributionAgent")]
    attribution_agent: Option<String>,
    #[serde(rename = "customTitle")]
    custom_title: Option<String>,
    #[serde(rename = "agentName")]
    agent_name: Option<String>,
    #[serde(rename = "lastPrompt")]
    last_prompt: Option<String>,
}

fn sv(v: &Option<Value>) -> Option<&str> {
    match v {
        Some(Value::String(s)) if !s.is_empty() => Some(s.as_str()),
        _ => None,
    }
}

fn target_for(name: &str, i: &Input) -> Option<String> {
    let t = match name {
        "Read" | "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => sv(&i.file_path)
            .or_else(|| sv(&i.notebook_path))
            .map(str::to_string),
        "Grep" | "Glob" => sv(&i.pattern).map(|p| match sv(&i.path) {
            Some(path) => format!("{p} in {path}"),
            None => p.to_string(),
        }),
        "Bash" | "PowerShell" => sv(&i.command).map(str::to_string),
        "WebFetch" | "WebSearch" => sv(&i.url).or_else(|| sv(&i.query)).map(str::to_string),
        "Agent" | "Task" => {
            let st = sv(&i.subagent_type).unwrap_or("agent");
            let d = sv(&i.description).unwrap_or("");
            Some(format!("{st}: {d}"))
        }
        _ => sv(&i.file_path)
            .or_else(|| sv(&i.path))
            .or_else(|| sv(&i.command))
            .or_else(|| sv(&i.url))
            .or_else(|| sv(&i.query))
            .map(str::to_string),
    };
    t.map(|s| truncate(&s, MAX_STR))
}

/// ISO-8601 UTC ("2026-11-18T12:34:56.789Z") to epoch ms. 0 when unparsable.
pub fn parse_ts(s: &str) -> i64 {
    let b = s.as_bytes();
    if b.len() < 19 {
        return 0;
    }
    let num = |a: usize, z: usize| -> Option<i64> { s.get(a..z)?.parse::<i64>().ok() };
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(se)) = (
        num(0, 4),
        num(5, 7),
        num(8, 10),
        num(11, 13),
        num(14, 16),
        num(17, 19),
    ) else {
        return 0;
    };
    let mut ms = 0i64;
    if b.len() > 20 && b[19] == b'.' {
        let mut digits = 0;
        let mut frac = 0i64;
        for &c in &b[20..] {
            if !c.is_ascii_digit() || digits == 3 {
                break;
            }
            frac = frac * 10 + (c - b'0') as i64;
            digits += 1;
        }
        while digits < 3 {
            frac *= 10;
            digits += 1;
        }
        ms = frac;
    }
    // days from civil (Howard Hinnant)
    let y2 = if mo <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = (mo + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    ((days * 24 + h) * 60 + mi) * 60_000 + se * 1000 + ms
}

pub fn parse_line(bytes: &[u8]) -> Result<Vec<LineEvent>, ParseError> {
    let raw: Raw = serde_json::from_slice(bytes).map_err(|e| ParseError(e.to_string()))?;
    let ts = raw.timestamp.as_deref().map(parse_ts).unwrap_or(0);
    let mut out = Vec::new();
    let ty = raw.ty.as_deref().unwrap_or("");
    match ty {
        "assistant" | "user" => {
            let Some(Lenient(Some(msg))) = raw.message else {
                return Ok(out);
            };
            if raw.cwd.is_some() || raw.git_branch.is_some() {
                out.push(LineEvent::Context {
                    cwd: raw.cwd.clone(),
                    git_branch: raw.git_branch.clone(),
                });
            }
            if ty == "assistant" {
                let usage = match msg.usage {
                    Some(Lenient(Some(u))) => Some(Usage {
                        input: u.input_tokens.unwrap_or(0),
                        output: u.output_tokens.unwrap_or(0),
                        cache_read: u.cache_read_input_tokens.unwrap_or(0),
                        cache_write: u.cache_creation_input_tokens.unwrap_or(0),
                    }),
                    _ => None,
                };
                out.push(LineEvent::AssistantMsg {
                    msg_id: msg.id.clone(),
                    usage,
                    stop_reason: msg.stop_reason.clone(),
                    model: msg.model.clone(),
                    attribution_agent: raw.attribution_agent.clone(),
                    ts,
                });
                if let Some(Content::Blocks(blocks)) = msg.content {
                    for b in blocks.into_iter().filter_map(|b| b.0) {
                        if b.ty.as_deref() != Some("tool_use") {
                            continue;
                        }
                        let (Some(id), Some(name)) = (b.id, b.name) else {
                            continue;
                        };
                        let input = b.input.and_then(|i| i.0).unwrap_or_default();
                        let target = target_for(&name, &input);
                        let spawn = if name == "Agent" || name == "Task" {
                            Some(SpawnInfo {
                                subagent_type: truncate(sv(&input.subagent_type).unwrap_or(""), MAX_STR),
                                description: truncate(sv(&input.description).unwrap_or(""), MAX_STR),
                                background: matches!(input.run_in_background, Some(Value::Bool(true))),
                            })
                        } else {
                            None
                        };
                        out.push(LineEvent::ToolUse {
                            id,
                            name,
                            target,
                            msg_id: msg.id.clone(),
                            ts,
                            spawn,
                        });
                    }
                }
            } else {
                match msg.content {
                    Some(Content::Text) => out.push(LineEvent::UserPrompt { ts }),
                    Some(Content::Blocks(blocks)) => {
                        let mut has_text = false;
                        let mut has_result = false;
                        for b in blocks.into_iter().filter_map(|b| b.0) {
                            match b.ty.as_deref() {
                                Some("tool_result") => {
                                    if let Some(id) = b.tool_use_id {
                                        has_result = true;
                                        out.push(LineEvent::ToolResult {
                                            tool_use_id: id,
                                            is_error: b.is_error.unwrap_or(false),
                                            ts,
                                        });
                                    }
                                }
                                Some("text") => has_text = true,
                                _ => {}
                            }
                        }
                        if has_text && !has_result {
                            out.push(LineEvent::UserPrompt { ts });
                        }
                    }
                    _ => {}
                }
            }
        }
        "system" => match raw.subtype.as_deref() {
            Some("stop_hook_summary") => out.push(LineEvent::TurnEnd { ts }),
            Some("api_error") => out.push(LineEvent::ApiError { ts }),
            _ => {}
        },
        "custom-title" => {
            if let Some(t) = raw.custom_title {
                out.push(LineEvent::Title(truncate(&t, MAX_STR)));
            }
        }
        "agent-name" => {
            if let Some(t) = raw.agent_name {
                out.push(LineEvent::AgentName(truncate(&t, MAX_STR)));
            }
        }
        "last-prompt" => {
            if let Some(t) = raw.last_prompt {
                out.push(LineEvent::LastPrompt(truncate(&t, MAX_STR)));
            }
        }
        _ => {}
    }
    Ok(out)
}
