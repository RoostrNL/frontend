use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::{Value, json};
use thiserror::Error;

const API_PREFIX: &str = "/api";

#[derive(Debug, Error)]
pub enum ApiErrors {
    #[error("Failed to parse user input.")]
    InputParsing,

    #[error("could not obtain a token: {0}")]
    GettingTokenFailure(String),

    #[error("invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("host `{0}` could not be reached")]
    HostUnreachable(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthenticated: {0}")]
    Unauthorized(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("server error ({status}): {message}")]
    Server { status: u16, message: String },

    #[error("unexpected api error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("the api returned no data")]
    MissingData,

    #[error("could not decode the response (http {status}): {source}")]
    Decode {
        status: u16,
        #[source]
        source: serde_json::Error,
    },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl ApiErrors {
    /// HTTP 400 / 401 / 404 / 5xx + anything else the wrapper reports.
    fn from_status(status: u16, message: String) -> Self {
        let message = if message.trim().is_empty() {
            StatusCode::from_u16(status)
                .ok()
                .and_then(|s| s.canonical_reason())
                .unwrap_or("unknown error")
                .to_string()
        } else {
            message.trim().to_string()
        };

        match status {
            401 => ApiErrors::Unauthorized(message),
            400..=403 => ApiErrors::BadRequest(message),
            404 => ApiErrors::NotFound(message),
            500..=599 => ApiErrors::Server { status, message },
            other => ApiErrors::Api {
                status: other,
                message,
            },
        }
    }

    fn transport(err: reqwest::Error, hostname: &str) -> Self {
        if err.is_connect() || err.is_timeout() {
            ApiErrors::HostUnreachable(hostname.to_string())
        } else {
            ApiErrors::Http(err)
        }
    }
}

// modelletjes
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub id: LessonId,

    #[serde(default)]
    pub subjects: Vec<String>,
    #[serde(default)]
    pub change_description: String,

    #[serde(default)]
    pub location_changed: bool,
    #[serde(default)]
    pub locations: Vec<String>,

    #[serde(default)]
    pub teacher_changed: bool,
    #[serde(default)]
    pub teachers: Vec<String>,

    #[serde(default)]
    pub group_changed: bool,
    #[serde(default)]
    pub groups: Vec<String>,

    #[serde(default)]
    pub time_changed: bool,
    #[serde(deserialize_with = "de_datetime")]
    pub start: DateTime<Utc>,
    #[serde(deserialize_with = "de_datetime")]
    pub end: DateTime<Utc>,
    #[serde(default)]
    pub start_time_slot: i64,
    #[serde(default)]
    pub start_time_slot_name: String,
    #[serde(default)]
    pub end_time_slot: i64,
    #[serde(default)]
    pub end_time_slot_name: String,

    #[serde(default)]
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum LessonId {
    Number(i64),
    Text(String),
}

impl std::fmt::Display for LessonId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LessonId::Number(n) => write!(f, "{n}"),
            LessonId::Text(s) => write!(f, "{s}"),
        }
    }
}

pub type DbId = LessonId;

/// `GET /api/me`
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    pub id: DbId,
    pub issued_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Timetable {
    pub user_id: String,
    pub provider: String,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub user_id: String,
    pub provider: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: String,
}

/// `data` of `POST /api/enroll`
#[derive(Debug, Clone, Deserialize)]
pub struct Enrollment {
    pub token_id: i64,
    pub issued_at: i64,
    pub token: String,
}

#[derive(Clone, Debug)]
pub struct Api {
    hostname: String,
    token: String,
    client: Client,
}

impl Api {
    pub async fn with_key(
        hostname: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<Self, ApiErrors> {
        let hostname = Self::check_host(hostname).await?;
        let client = Self::build_client();
        let token = Self::get_token_with(&client, &hostname, key.as_ref()).await?;

        Ok(Self {
            hostname,
            token,
            client,
        })
    }

    pub async fn with_token(
        hostname: impl AsRef<str>,
        token: impl Into<String>,
    ) -> Result<Self, ApiErrors> {
        let hostname = Self::check_host(hostname).await?;
        let client = Self::build_client();

        let api = Self {
            hostname,
            client,
            token: token.into(),
        };
        api.get_me().await?;

        Ok(api)
    }

    pub async fn check_host(hostname: impl AsRef<str>) -> Result<String, ApiErrors> {
        let hostname = hostname.as_ref().trim().to_string();
        if hostname.is_empty() {
            return Err(ApiErrors::InvalidHostname("hostname is empty".into()));
        }

        let client = Self::build_client();
        match client
            .get(format!("{hostname}{API_PREFIX}/enroll"))
            .send()
            .await
        {
            Ok(_) => Ok(hostname),
            Err(err) => Err(ApiErrors::transport(err, &hostname)),
        }
    }

    async fn get_token_with(
        client: &Client,
        hostname: &str,
        key: &str,
    ) -> Result<String, ApiErrors> {
        let url = format!("{hostname}{API_PREFIX}/enroll");

        // the route reads `body.enroll_code` and 400s on anything else
        let response = client
            .post(&url)
            .json(&json!({ "enroll_code": key }))
            .send()
            .await
            .map_err(|err| ApiErrors::transport(err, hostname))?;

        let http_status = response.status();
        let body = response.text().await.map_err(ApiErrors::Http)?;

        // `data` is the Enrollment object itself
        let payload: TokenPayload = Self::decode_envelope(http_status, &body)?;
        payload.into_token().ok_or_else(|| {
            ApiErrors::GettingTokenFailure(format!("no token in response from {url}"))
        })
    }

    fn build_client() -> Client {
        Client::builder()
            .user_agent(concat!("timetable-api-rs/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client with rustls")
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }
    pub fn token(&self) -> &str {
        &self.token
    }

    /// `GET /api/me`
    pub async fn get_me(&self) -> Result<Me, ApiErrors> {
        let url = format!("{}{API_PREFIX}/me", self.hostname);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|err| ApiErrors::transport(err, &self.hostname))?;

        let http_status = response.status();
        let text = response.text().await.map_err(ApiErrors::Http)?;

        if !http_status.is_success() {
            return Err(ApiErrors::from_status(http_status.as_u16(), text));
        }

        serde_json::from_str(&text).map_err(|source| ApiErrors::Decode {
            status: http_status.as_u16(),
            source,
        })
    }

    /// `GET /api/user/:user_id`
    pub async fn get_user(&self, user_id: &str) -> Result<User, ApiErrors> {
        self.request_one(
            Method::GET,
            &format!("{API_PREFIX}/user/{}", encode_segment(user_id)),
            None,
        )
        .await
    }

    /// `GET /api/timetable/:user_id`
    pub async fn get_timetable(&self, user_id: &str) -> Result<Timetable, ApiErrors> {
        self.request_one(
            Method::GET,
            &format!("{API_PREFIX}/timetable/{}", encode_segment(user_id)),
            None,
        )
        .await
    }

    /// `POST /api/enroll`
    pub async fn enroll(&self, code: &str) -> Result<Enrollment, ApiErrors> {
        self.request_one(
            Method::POST,
            &format!("{API_PREFIX}/enroll"),
            Some(json!({ "enroll_code": code })),
        )
        .await
    }

    // request helper
    async fn request_one<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<T, ApiErrors> {
        let url = format!("{}{}", self.hostname, path);

        let mut request = self.client.request(method, &url).bearer_auth(&self.token);
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request
            .send()
            .await
            .map_err(|err| ApiErrors::transport(err, &self.hostname))?;

        let http_status = response.status();
        let text = response.text().await.map_err(ApiErrors::Http)?;

        Self::decode_envelope(http_status, &text)
    }

    /// unpack funny api structure: `{ "status": 200, "message": "", "data": {...} }`
    fn decode_envelope<T: DeserializeOwned>(
        http_status: StatusCode,
        text: &str,
    ) -> Result<T, ApiErrors> {
        let envelope: ApiEnvelope<T> = match serde_json::from_str(text) {
            Ok(envelope) => envelope,
            Err(source) => {
                return Err(if http_status.is_success() {
                    ApiErrors::Decode {
                        status: http_status.as_u16(),
                        source,
                    }
                } else {
                    ApiErrors::from_status(http_status.as_u16(), text.to_string())
                });
            }
        };

        let status = if envelope.status >= 100 {
            envelope.status
        } else {
            http_status.as_u16()
        };

        if status >= 400 {
            return Err(ApiErrors::from_status(status, envelope.message));
        }

        envelope.data.ok_or(ApiErrors::MissingData)
    }
}

/// `{ "status": 200, "message": "", "data": ... }` — `data` is a single object
/// on every route we talk to (`/enroll`, `/timetable/:user_id`), and absent on errors.
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: serde::de::DeserializeOwned"))]
struct ApiEnvelope<T> {
    #[serde(default)]
    status: u16,
    #[serde(default)]
    message: String,
    #[serde(default)]
    data: Option<T>,
}

/// `data` of `/enroll`, or the bare token string if the server ever answers that way.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenPayload {
    Enrollment(Enrollment),
    Raw(String),
}

impl TokenPayload {
    fn into_token(self) -> Option<String> {
        let token = match self {
            TokenPayload::Enrollment(e) => e.token,
            TokenPayload::Raw(t) => t,
        };
        (!token.trim().is_empty()).then_some(token)
    }
}

fn encode_segment(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn de_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as _;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Repr {
        Text(String),
        Int(i64),
    }

    match Repr::deserialize(deserializer)? {
        Repr::Text(text) => DateTime::parse_from_rfc3339(&text)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|err| D::Error::custom(format!("invalid date `{text}`: {err}"))),
        Repr::Int(value) => {
            let (secs, nanos) = if value.abs() > 100_000_000_000 {
                (
                    value / 1000,
                    (value % 1000).unsigned_abs() as u32 * 1_000_000,
                )
            } else {
                (value, 0)
            };
            Utc.timestamp_opt(secs, nanos)
                .single()
                .ok_or_else(|| D::Error::custom(format!("date out of range: {value}")))
        }
    }
}
