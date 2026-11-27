//! AI入口(2026-11-27新設): VS Code拡張 maid-cafe-vsc などが、maid-cafe.tokyo 経由で
//! aruaru-llm(127.0.0.1:4600)の無料AI(Translate / Generate / Ocr)を使うための門。
//!
//! - 版なしConnect(`POST /openenglish.AiGate/<Method>`、JSON)。新しいRESTは作らない。
//! - ログイン: メールアドレス → メールのワンタイムパスワード(既存の`auth.rs`の発行・検証を再利用)。
//!   成功するとAI入口専用のトークン(`Authorization: Bearer`)を返す。通常のログインセッション
//!   (管理画面など)は発行しないので、管理者の2段階認証を迂回する道にはならない。
//! - 未ログインは401。aruaru-llm を素のまま外へ出さず、この門を通した3つのメソッドだけを転送する。
//! - 入場制限: VPSの空きメモリ(`/proc/meminfo` の MemAvailable)を2秒ごとに測り、
//!   同時実行の上限を自動で上げ下げする(加算増加・乗算減少=AIMD の単純なフィードバック制御)。
//!   **正直な説明**: 機械学習のモデルではなく、空きメモリと応答時間の推移を見て上限を
//!   調整する規則ベースの自動制御である。空きが危険域なら新規は即503+Retry-After、
//!   上限に達したら待ち行列(既定8人、最大20秒待ち)、あふれたら503+Retry-After。
//!   aruaru-llm 側の受付制御(同時実行上限・サイト別公平キュー)には `x-tenant: maid-cafe-vsc` で参加する。

use open_runo_poem_compat::{Request, Response, StatusCode};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

const TOKEN_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const MAX_BODY: usize = 8 * 1024 * 1024;
pub const TENANT: &str = "maid-cafe-vsc";
/// 転送してよい aruaru-llm のメソッド(これ以外は404)。
pub const ALLOWED_METHODS: [&str; 3] = ["Translate", "Generate", "Ocr"];

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// 入場制御の設定(環境変数で変更可)。
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub critical_mb: u64,
    pub low_mb: u64,
    pub high_mb: u64,
    pub min_limit: usize,
    pub max_limit: usize,
    pub max_queue: usize,
    pub queue_wait: Duration,
    pub slow_ms: u64,
}

impl Limits {
    pub fn from_env() -> Self {
        Limits {
            critical_mb: env_u64("AI_GATE_MEM_CRITICAL_MB", 200),
            low_mb: env_u64("AI_GATE_MEM_LOW_MB", 350),
            high_mb: env_u64("AI_GATE_MEM_HIGH_MB", 700),
            min_limit: 1,
            max_limit: env_u64("AI_GATE_MAX_CONCURRENT", 4).max(1) as usize,
            max_queue: env_u64("AI_GATE_MAX_QUEUE", 8) as usize,
            queue_wait: Duration::from_secs(env_u64("AI_GATE_QUEUE_WAIT_SECS", 20)),
            slow_ms: env_u64("AI_GATE_SLOW_MS", 60_000),
        }
    }
}

/// 自動調整の1ステップ(純粋関数、試験しやすいよう分離)。
/// 空きが少ない→半分に(最低1)、空きが十分で上限まで使われ・遅くない→1つ増やす。
pub fn next_limit(cur: usize, mem_avail_mb: Option<u64>, saturated: bool, latency_ms: u64, l: &Limits) -> usize {
    match mem_avail_mb {
        Some(m) if m < l.low_mb => (cur / 2).max(l.min_limit),
        Some(m) if m > l.high_mb && saturated && latency_ms < l.slow_ms => (cur + 1).min(l.max_limit),
        None if saturated && latency_ms < l.slow_ms => (cur + 1).min(l.max_limit),
        _ if latency_ms >= l.slow_ms => (cur.saturating_sub(1)).max(l.min_limit),
        _ => cur.clamp(l.min_limit, l.max_limit),
    }
}

struct Gate {
    limits: Limits,
    limit: AtomicUsize,
    inflight: AtomicUsize,
    waiting: AtomicUsize,
    /// MemAvailable(MB)。u64::MAX は「測れない(Linux以外)」。
    mem_avail_mb: AtomicU64,
    latency_ewma_ms: AtomicU64,
    saturated_since_adjust: AtomicUsize,
    notify: tokio::sync::Notify,
    tokens: Mutex<HashMap<String, (String, SystemTime)>>,
}

fn gate() -> &'static Gate {
    static G: OnceLock<Gate> = OnceLock::new();
    G.get_or_init(|| {
        let limits = Limits::from_env();
        Gate {
            limit: AtomicUsize::new(2.min(limits.max_limit)),
            limits,
            inflight: AtomicUsize::new(0),
            waiting: AtomicUsize::new(0),
            mem_avail_mb: AtomicU64::new(u64::MAX),
            latency_ewma_ms: AtomicU64::new(0),
            saturated_since_adjust: AtomicUsize::new(0),
            notify: tokio::sync::Notify::new(),
            tokens: Mutex::new(HashMap::new()),
        }
    })
}

pub fn parse_mem_available_mb(meminfo: &str) -> Option<u64> {
    meminfo.lines().find_map(|l| {
        let rest = l.strip_prefix("MemAvailable:")?;
        let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
        Some(kb / 1024)
    })
}

fn read_mem_available_mb() -> Option<u64> {
    std::fs::read_to_string("/proc/meminfo").ok().as_deref().and_then(parse_mem_available_mb)
}

fn mem_now() -> Option<u64> {
    match gate().mem_avail_mb.load(Ordering::Relaxed) {
        u64::MAX => None,
        v => Some(v),
    }
}

/// 監視タスクを起動する(サーバー起動時に1回)。2秒ごとに測定、10秒ごとに上限を調整。
pub fn spawn_monitor() {
    tokio::spawn(async move {
        let g = gate();
        let mut tick: u64 = 0;
        loop {
            if let Some(m) = read_mem_available_mb() {
                g.mem_avail_mb.store(m, Ordering::Relaxed);
            }
            tick += 1;
            if tick % 5 == 0 {
                let cur = g.limit.load(Ordering::Relaxed);
                let saturated = g.saturated_since_adjust.swap(0, Ordering::Relaxed) > 0;
                let next = next_limit(cur, mem_now(), saturated, g.latency_ewma_ms.load(Ordering::Relaxed), &g.limits);
                if next != cur {
                    g.limit.store(next, Ordering::Relaxed);
                    eprintln!("[ai-gate] concurrency limit {cur} -> {next} (mem_avail_mb={:?})", mem_now());
                    g.notify.notify_waiters();
                }
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

fn busy(retry_after: u64, why: &str) -> Response {
    let body = serde_json::json!({
        "error": format!("busy: {why} / 混み合っています。{retry_after}秒ほど待ってから再試行してください"),
        "retry_after_secs": retry_after,
    });
    hyper::Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header("content-type", "application/json")
        .header("retry-after", retry_after.to_string())
        .body(open_runo_poem_compat::hyper_compat::fixed_body(bytes::Bytes::from(body.to_string())))
        .expect("valid response")
}

struct Permit;
impl Drop for Permit {
    fn drop(&mut self) {
        gate().inflight.fetch_sub(1, Ordering::AcqRel);
        gate().notify.notify_one();
    }
}

fn try_take() -> Option<Permit> {
    let g = gate();
    let limit = g.limit.load(Ordering::Relaxed);
    let mut cur = g.inflight.load(Ordering::Acquire);
    while cur < limit {
        match g.inflight.compare_exchange(cur, cur + 1, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => return Some(Permit),
            Err(v) => cur = v,
        }
    }
    None
}

async fn admit() -> Result<Permit, Response> {
    let g = gate();
    if let Some(m) = mem_now() {
        if m < g.limits.critical_mb {
            return Err(busy(30, "server memory is low"));
        }
    }
    if let Some(p) = try_take() {
        return Ok(p);
    }
    g.saturated_since_adjust.fetch_add(1, Ordering::Relaxed);
    if g.waiting.fetch_add(1, Ordering::AcqRel) >= g.limits.max_queue {
        g.waiting.fetch_sub(1, Ordering::AcqRel);
        return Err(busy(15, "queue is full"));
    }
    let deadline = Instant::now() + g.limits.queue_wait;
    let result = loop {
        let notified = g.notify.notified();
        if let Some(p) = try_take() {
            break Ok(p);
        }
        let now = Instant::now();
        if now >= deadline {
            break Err(busy(10, "waited too long in the queue"));
        }
        let _ = tokio::time::timeout(deadline - now, notified).await;
    };
    g.waiting.fetch_sub(1, Ordering::AcqRel);
    result
}

fn json(status: StatusCode, v: serde_json::Value) -> Response {
    hyper::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(open_runo_poem_compat::hyper_compat::fixed_body(bytes::Bytes::from(v.to_string())))
        .expect("valid response")
}

async fn read_body(req: Request) -> Result<bytes::Bytes, Response> {
    use http_body_util::BodyExt;
    let limited = http_body_util::Limited::new(req.into_body(), MAX_BODY);
    match limited.collect().await {
        Ok(c) => Ok(c.to_bytes()),
        Err(_) => Err(json(StatusCode::PAYLOAD_TOO_LARGE, serde_json::json!({"error": "body too large or unreadable"}))),
    }
}

fn read_json(bytes: &[u8]) -> Result<serde_json::Value, Response> {
    rust_json::from_slice_strict::<serde_json::Value>(bytes)
        .map_err(|e| json(StatusCode::BAD_REQUEST, serde_json::json!({"error": format!("invalid JSON: {e}")})))
}

fn bearer(req: &Request) -> Option<String> {
    let h = req.headers().get("authorization")?.to_str().ok()?;
    let t = h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer "))?.trim();
    (!t.is_empty()).then(|| t.to_string())
}

pub fn token_email(token: &str) -> Option<String> {
    let mut map = gate().tokens.lock().unwrap();
    match map.get(token) {
        Some((email, exp)) if SystemTime::now() < *exp => Some(email.clone()),
        Some(_) => {
            map.remove(token);
            None
        }
        None => None,
    }
}

pub fn issue_token(email: &str) -> String {
    use ring::rand::SecureRandom;
    let mut buf = [0u8; 32];
    ring::rand::SystemRandom::new().fill(&mut buf).expect("system RNG");
    let token: String = buf.iter().map(|b| format!("{b:02x}")).collect();
    let mut map = gate().tokens.lock().unwrap();
    let now = SystemTime::now();
    map.retain(|_, (_, exp)| *exp > now);
    map.insert(token.clone(), (email.to_string(), now + TOKEN_TTL));
    token
}

/// `POST /openenglish.AiGate/RequestOtp` `{"email"}` → `{"ok":true}`
pub async fn request_otp(req: Request) -> Response {
    let body = match read_body(req).await.and_then(|b| read_json(&b)) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let email = body.get("email").and_then(|v| v.as_str()).unwrap_or_default();
    match crate::auth::request_otp_any(email).await {
        Ok(()) => json(StatusCode::OK, serde_json::json!({"ok": true})),
        Err(e) => json(StatusCode::BAD_REQUEST, serde_json::json!({"error": format!("{e:#}")})),
    }
}

/// `POST /openenglish.AiGate/VerifyOtp` `{"email","code"}` → `{"token","expires_in_secs"}`
pub async fn verify_otp(req: Request) -> Response {
    let body = match read_body(req).await.and_then(|b| read_json(&b)) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let email = body.get("email").and_then(|v| v.as_str()).unwrap_or_default();
    let code = body.get("code").and_then(|v| v.as_str()).unwrap_or_default();
    match crate::auth::verify_code(email, code) {
        Ok(id) => json(StatusCode::OK, serde_json::json!({"token": issue_token(&id), "expires_in_secs": TOKEN_TTL.as_secs()})),
        Err(e) => json(StatusCode::UNAUTHORIZED, serde_json::json!({"error": format!("{e:#}")})),
    }
}

fn require_token(req: &Request) -> Result<String, Response> {
    bearer(req).as_deref().and_then(token_email).ok_or_else(|| {
        json(StatusCode::UNAUTHORIZED, serde_json::json!({"error": "login required / ログインが必要です(メールのワンタイムパスワード)"}))
    })
}

/// `POST /openenglish.AiGate/Status` → 入場制御の状態(ログイン必須)。
pub async fn status(req: Request) -> Response {
    let email = match require_token(&req) {
        Ok(e) => e,
        Err(r) => return r,
    };
    let g = gate();
    json(StatusCode::OK, serde_json::json!({
        "email": email,
        "concurrency_limit": g.limit.load(Ordering::Relaxed),
        "inflight": g.inflight.load(Ordering::Relaxed),
        "waiting": g.waiting.load(Ordering::Relaxed),
        "mem_available_mb": mem_now(),
        "latency_ewma_ms": g.latency_ewma_ms.load(Ordering::Relaxed),
    }))
}

/// `POST /openenglish.AiGate/{Translate,Generate,Ocr}` → aruaru-llm の同名メソッドへ転送。
pub async fn forward(req: Request, method: &'static str) -> Response {
    if let Err(r) = require_token(&req) {
        return r;
    }
    let body = match read_body(req).await {
        Ok(b) => b,
        Err(r) => return r,
    };
    if let Err(r) = read_json(&body) {
        return r;
    }
    let _permit = match admit().await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let started = Instant::now();
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(180)).build() {
        Ok(c) => c,
        Err(e) => return json(StatusCode::INTERNAL_SERVER_ERROR, serde_json::json!({"error": e.to_string()})),
    };
    let resp = client
        .post(crate::llm_connect_url(method))
        .header("content-type", "application/json")
        .header("x-tenant", TENANT)
        .body(body)
        .send()
        .await;
    let out = match resp {
        Ok(r) => {
            let status = StatusCode::from_u16(r.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let retry = r.headers().get("retry-after").and_then(|v| v.to_str().ok()).map(str::to_string);
            let bytes = r.bytes().await.unwrap_or_default();
            let mut b = hyper::Response::builder().status(status).header("content-type", "application/json");
            if let Some(ra) = retry {
                b = b.header("retry-after", ra);
            }
            b.body(open_runo_poem_compat::hyper_compat::fixed_body(bytes)).expect("valid response")
        }
        Err(e) => json(StatusCode::BAD_GATEWAY, serde_json::json!({"error": format!("AI engine unavailable / AIエンジンに接続できません: {e}")})),
    };
    let ms = started.elapsed().as_millis() as u64;
    let g = gate();
    let prev = g.latency_ewma_ms.load(Ordering::Relaxed);
    g.latency_ewma_ms.store(if prev == 0 { ms } else { (prev * 4 + ms) / 5 }, Ordering::Relaxed);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l() -> Limits {
        Limits { critical_mb: 200, low_mb: 350, high_mb: 700, min_limit: 1, max_limit: 4, max_queue: 8, queue_wait: Duration::from_secs(1), slow_ms: 60_000 }
    }

    #[test]
    fn meminfo_is_parsed() {
        assert_eq!(parse_mem_available_mb("MemTotal: 3737600 kB\nMemAvailable:    984064 kB\n"), Some(961));
        assert_eq!(parse_mem_available_mb("MemTotal: 1 kB\n"), None);
    }

    #[test]
    fn limit_halves_when_memory_low_and_grows_when_plenty() {
        assert_eq!(next_limit(4, Some(300), true, 1000, &l()), 2);
        assert_eq!(next_limit(1, Some(100), true, 1000, &l()), 1);
        assert_eq!(next_limit(2, Some(900), true, 1000, &l()), 3);
        assert_eq!(next_limit(4, Some(900), true, 1000, &l()), 4);
        assert_eq!(next_limit(2, Some(900), false, 1000, &l()), 2);
        assert_eq!(next_limit(3, Some(500), true, 90_000, &l()), 2);
    }

    #[test]
    fn tokens_round_trip() {
        let t = issue_token("a@example.com");
        assert_eq!(token_email(&t).as_deref(), Some("a@example.com"));
        assert_eq!(token_email("nope"), None);
    }
}
