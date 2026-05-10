//! ddg_probe — empirical characterization of DuckDuckGo's HTML endpoint blocking behavior.
//!
//! This is a deliberate research tool, not part of the shipped library. It bypasses
//! the rate-limit guard and the higher-level search machinery so the only request
//! shape that reaches DDG is the one configured here.
//!
//! Run examples (always with `--release`; debug builds are 4-10× slower and skew timing):
//!
//!   cargo run --example ddg_probe --release -- preview
//!   cargo run --example ddg_probe --release -- serial --interval-ms 1500 --count 30
//!   cargo run --example ddg_probe --release -- burst  --parallel 10 --rounds 3 --gap-s 90
//!   cargo run --example ddg_probe --release -- recovery --probe-interval-s 10 --max 30
//!   cargo run --example ddg_probe --release -- pulse --rate-ms 700 --duration-s 60
//!
//! Each scenario emits one JSON object per line on stdout (JSONL). The last line is
//! a `"phase":"summary"` object. Pipe to a file with `--out runs/<label>.jsonl` so a
//! later analysis pass can aggregate across IPs / scenarios.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use url::Url;
use wreq::Client;
use wreq::header::{HeaderMap, HeaderValue};

const DEFAULT_ENDPOINT: &str = "https://html.duckduckgo.com/html";
const DEFAULT_QUERY: &str = "rust programming language";
const DEFAULT_REGION: &str = "us-en";

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let mut argv = std::env::args().skip(1);
    let scenario = argv.next().unwrap_or_else(|| "preview".to_owned());
    let args = parse_args(argv.collect::<Vec<_>>());

    let runner = Runner::new(&args)?;
    let started_at = OffsetDateTime::now_utc();
    eprintln!(
        "[probe] scenario={scenario} label={} ua_empty={} emulation={} endpoint={} query={:?}",
        args.label,
        args.user_agent.is_none(),
        args.emulation,
        args.endpoint,
        args.query,
    );

    let summary = match scenario.as_str() {
        "preview" => runner.preview().await,
        "cli-path" => runner.cli_path().await,
        "serial" => runner.serial(args.interval_ms, args.count).await,
        "burst" => runner.burst(args.parallel, args.rounds, args.gap_s).await,
        "recovery" => {
            runner
                .recovery(args.probe_interval_s, args.max_attempts)
                .await
        }
        "pulse" => runner.pulse(args.rate_ms, args.duration_s).await,
        other => {
            eprintln!("[probe] unknown scenario: {other}");
            std::process::exit(2);
        }
    };

    let envelope = SummaryEnvelope {
        phase: "summary",
        scenario,
        label: args.label.clone(),
        started_at: started_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        finished_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        params: ParamsEcho {
            endpoint: args.endpoint.to_string(),
            query: args.query.clone(),
            region: args.region.clone(),
            user_agent: args.user_agent.clone(),
            emulation: args.emulation,
            interval_ms: args.interval_ms,
            count: args.count,
            parallel: args.parallel,
            rounds: args.rounds,
            gap_s: args.gap_s,
            rate_ms: args.rate_ms,
            duration_s: args.duration_s,
            probe_interval_s: args.probe_interval_s,
            max_attempts: args.max_attempts,
        },
        summary,
    };
    runner.write_line(&envelope).await?;
    Ok(())
}

#[derive(Debug)]
struct Args {
    endpoint: Url,
    query: String,
    region: String,
    user_agent: Option<String>,
    emulation: bool,
    label: String,
    out: Option<PathBuf>,
    interval_ms: u64,
    count: u32,
    parallel: u32,
    rounds: u32,
    gap_s: u64,
    rate_ms: u64,
    duration_s: u64,
    probe_interval_s: u64,
    max_attempts: u32,
}

fn parse_args(argv: Vec<String>) -> Args {
    let mut args = Args {
        endpoint: Url::parse(DEFAULT_ENDPOINT).expect("default endpoint"),
        query: DEFAULT_QUERY.to_owned(),
        region: DEFAULT_REGION.to_owned(),
        user_agent: None,
        emulation: false,
        label: "default".to_owned(),
        out: None,
        interval_ms: 1500,
        count: 30,
        parallel: 5,
        rounds: 3,
        gap_s: 60,
        rate_ms: 700,
        duration_s: 60,
        probe_interval_s: 10,
        max_attempts: 24,
    };
    let mut iter = argv.into_iter();
    while let Some(token) = iter.next() {
        match token.as_str() {
            "--endpoint" => args.endpoint = Url::parse(&need(&mut iter, &token)).expect("url"),
            "--query" => args.query = need(&mut iter, &token),
            "--region" => args.region = need(&mut iter, &token),
            "--user-agent" => args.user_agent = Some(need(&mut iter, &token)),
            "--emulation" => args.emulation = true,
            "--no-emulation" => args.emulation = false,
            "--label" => args.label = need(&mut iter, &token),
            "--out" => args.out = Some(PathBuf::from(need(&mut iter, &token))),
            "--interval-ms" => args.interval_ms = parse_u64(&mut iter, &token),
            "--count" => args.count = parse_u32(&mut iter, &token),
            "--parallel" => args.parallel = parse_u32(&mut iter, &token),
            "--rounds" => args.rounds = parse_u32(&mut iter, &token),
            "--gap-s" => args.gap_s = parse_u64(&mut iter, &token),
            "--rate-ms" => args.rate_ms = parse_u64(&mut iter, &token),
            "--duration-s" => args.duration_s = parse_u64(&mut iter, &token),
            "--probe-interval-s" => args.probe_interval_s = parse_u64(&mut iter, &token),
            "--max" | "--max-attempts" => args.max_attempts = parse_u32(&mut iter, &token),
            other => {
                eprintln!("[probe] unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }
    args
}

fn need(iter: &mut impl Iterator<Item = String>, flag: &str) -> String {
    iter.next().unwrap_or_else(|| {
        eprintln!("[probe] {flag} requires a value");
        std::process::exit(2);
    })
}

fn parse_u64(iter: &mut impl Iterator<Item = String>, flag: &str) -> u64 {
    need(iter, flag).parse().unwrap_or_else(|_| {
        eprintln!("[probe] {flag} expects an integer");
        std::process::exit(2);
    })
}

fn parse_u32(iter: &mut impl Iterator<Item = String>, flag: &str) -> u32 {
    need(iter, flag).parse().unwrap_or_else(|_| {
        eprintln!("[probe] {flag} expects an integer");
        std::process::exit(2);
    })
}

struct Runner {
    client: std::cell::OnceCell<Client>,
    args: Args,
    out: Arc<Mutex<Option<tokio::fs::File>>>,
    t0: Instant,
}

impl Runner {
    fn new(args: &Args) -> std::io::Result<Self> {
        let out = if let Some(path) = &args.out {
            Some(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?,
            )
        } else {
            None
        };
        Ok(Self {
            client: std::cell::OnceCell::new(),
            args: clone_args(args),
            out: Arc::new(Mutex::new(out.map(tokio::fs::File::from_std))),
            t0: Instant::now(),
        })
    }

    fn client(&self) -> &Client {
        self.client.get_or_init(|| {
            let mut default_headers = HeaderMap::new();
            default_headers.insert("Accept-Encoding", HeaderValue::from_static("gzip"));
            default_headers.insert("DNT", HeaderValue::from_static("1"));
            let mut builder = Client::builder()
                .default_headers(default_headers)
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(5))
                .read_timeout(Duration::from_secs(15))
                .no_proxy()
                .user_agent("");
            if self.args.emulation && self.args.endpoint.scheme() == "https" {
                builder = builder.emulation(wreq::EmulationProvider::default());
            }
            if let Some(ua) = &self.args.user_agent {
                builder = builder.user_agent(ua.as_str());
            }
            builder.build().expect("build wreq client")
        })
    }

    async fn preview(&self) -> ScenarioSummary {
        let event = self.fire(0).await;
        self.write_line(&event).await.ok();
        ScenarioSummary {
            total: 1,
            ok: u32::from(event.classify == "ok"),
            ok_empty: u32::from(event.classify == "ok_empty"),
            blocked: u32::from(event.classify.starts_with("blocked")),
            remote: u32::from(event.classify.starts_with("remote")),
            errored: u32::from(event.classify.starts_with("error")),
            first_block_ix: (event.classify.starts_with("blocked")).then_some(0),
            first_block_t_ms: (event.classify.starts_with("blocked")).then_some(event.t_ms),
        }
    }

    /// Side-channel: drive the public CLI-facing `Client` API the same
    /// way the production binary does. Used as a debugging aid to verify
    /// whether the difference between probe and CLI is in the low-level
    /// transport (this path) vs higher-level CLI plumbing.
    async fn cli_path(&self) -> ScenarioSummary {
        use duckduckgo_core::Client;
        let dir = std::env::temp_dir().join(format!("ddg_probe_state_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let client = Client::builder()
            .endpoint(self.args.endpoint.as_str().to_owned())
            .num(1)
            .safe(true)
            .timeout(30)
            .no_rate_limit(true)
            .state_dir(dir)
            .build()
            .expect("build");
        let started = OffsetDateTime::now_utc();
        let begin = Instant::now();
        let response = client.search(&self.args.query).page(1).send().await;
        let duration_ms = begin.elapsed().as_millis() as u64;
        let event = match response {
            Ok(r) => Event {
                phase: "request",
                ix: 0,
                t_ms: 0,
                started_at: started
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                duration_ms,
                status: Some(200),
                len: Some(r.results.len()),
                classify: if r.results.is_empty() {
                    "ok_empty".into()
                } else {
                    "ok".into()
                },
                final_url: None,
                location: None,
                body_marker: Some(format!("via cli_path; results={}", r.results.len())),
                error: None,
            },
            Err(e) => Event {
                phase: "request",
                ix: 0,
                t_ms: 0,
                started_at: started
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                duration_ms,
                status: None,
                len: None,
                classify: format!("error:{e}").chars().take(96).collect(),
                final_url: None,
                location: None,
                body_marker: None,
                error: Some(e.to_string()),
            },
        };
        let mut summary = ScenarioSummary {
            total: 1,
            ..ScenarioSummary::default()
        };
        if event.classify == "ok" {
            summary.ok = 1;
        } else if event.classify == "ok_empty" {
            summary.ok_empty = 1;
        } else if event.classify.starts_with("blocked") {
            summary.blocked = 1;
        } else {
            summary.errored = 1;
        }
        self.write_line(&event).await.ok();
        summary
    }

    async fn serial(&self, interval_ms: u64, count: u32) -> ScenarioSummary {
        let mut summary = ScenarioSummary {
            total: count,
            ..ScenarioSummary::default()
        };
        for ix in 0..count {
            if ix > 0 {
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            }
            let event = self.fire(ix).await;
            self.update_summary(&mut summary, &event);
            self.write_line(&event).await.ok();
        }
        summary
    }

    async fn burst(&self, parallel: u32, rounds: u32, gap_s: u64) -> ScenarioSummary {
        let mut summary = ScenarioSummary {
            total: parallel.saturating_mul(rounds),
            ..ScenarioSummary::default()
        };
        for round in 0..rounds {
            if round > 0 {
                tokio::time::sleep(Duration::from_secs(gap_s)).await;
            }
            let mut handles = Vec::with_capacity(parallel as usize);
            for slot in 0..parallel {
                let ix = round * parallel + slot;
                let task = self.clone_for_task();
                handles.push(tokio::spawn(async move { task.send(ix).await }));
            }
            for h in handles {
                if let Ok(event) = h.await {
                    self.update_summary(&mut summary, &event);
                    self.write_line(&event).await.ok();
                }
            }
        }
        summary
    }

    async fn recovery(&self, interval_s: u64, max_attempts: u32) -> ScenarioSummary {
        let mut summary = ScenarioSummary {
            total: max_attempts,
            ..ScenarioSummary::default()
        };
        for ix in 0..max_attempts {
            if ix > 0 {
                tokio::time::sleep(Duration::from_secs(interval_s)).await;
            }
            let event = self.fire(ix).await;
            self.update_summary(&mut summary, &event);
            self.write_line(&event).await.ok();
            if matches!(event.classify.as_str(), "ok" | "ok_empty") {
                break;
            }
        }
        summary
    }

    async fn pulse(&self, rate_ms: u64, duration_s: u64) -> ScenarioSummary {
        let mut summary = ScenarioSummary::default();
        let deadline = Instant::now() + Duration::from_secs(duration_s);
        let mut ix: u32 = 0;
        while Instant::now() < deadline {
            let event = self.fire(ix).await;
            self.update_summary(&mut summary, &event);
            self.write_line(&event).await.ok();
            summary.total += 1;
            ix += 1;
            tokio::time::sleep(Duration::from_millis(rate_ms)).await;
        }
        summary
    }

    fn clone_for_task(&self) -> Arc<RunnerTask> {
        Arc::new(RunnerTask {
            client: self.client().clone(),
            endpoint: self.args.endpoint.clone(),
            query: self.args.query.clone(),
            region: self.args.region.clone(),
            t0: self.t0,
        })
    }

    async fn fire(&self, ix: u32) -> Event {
        let task = self.clone_for_task();
        task.send(ix).await
    }

    fn update_summary(&self, summary: &mut ScenarioSummary, event: &Event) {
        match event.classify.as_str() {
            "ok" => summary.ok += 1,
            "ok_empty" => summary.ok_empty += 1,
            c if c.starts_with("blocked") => {
                summary.blocked += 1;
                if summary.first_block_ix.is_none() {
                    summary.first_block_ix = Some(event.ix);
                    summary.first_block_t_ms = Some(event.t_ms);
                }
            }
            c if c.starts_with("remote") => summary.remote += 1,
            _ => summary.errored += 1,
        }
    }

    async fn write_line<S: Serialize>(&self, value: &S) -> std::io::Result<()> {
        let line = serde_json::to_string(value).map_err(std::io::Error::other)?;
        println!("{line}");
        let mut guard = self.out.lock().await;
        if let Some(file) = guard.as_mut() {
            file.write_all(line.as_bytes()).await?;
            file.write_all(b"\n").await?;
            file.flush().await?;
        }
        Ok(())
    }
}

struct RunnerTask {
    client: Client,
    endpoint: Url,
    query: String,
    region: String,
    t0: Instant,
}

impl RunnerTask {
    async fn send(self: Arc<Self>, ix: u32) -> Event {
        let started = OffsetDateTime::now_utc();
        let t_ms = self.t0.elapsed().as_millis() as u64;
        let begin = Instant::now();
        let fields = vec![
            ("q", self.query.as_str()),
            ("b", ""),
            ("df", ""),
            ("kf", "-1"),
            ("kh", "1"),
            ("kl", self.region.as_str()),
            ("kp", "1"),
            ("k1", "-1"),
        ];
        let result = self
            .client
            .post(self.endpoint.as_str())
            .form(&fields)
            .send()
            .await;
        let duration_ms = begin.elapsed().as_millis() as u64;
        match result {
            Ok(response) => {
                let status = response.status().as_u16();
                let final_url = response.url().clone();
                let location = response
                    .headers()
                    .get(wreq::header::LOCATION)
                    .and_then(|h| h.to_str().ok())
                    .map(str::to_owned);
                let body = response.text().await.unwrap_or_default();
                let len = body.len();
                let classify = classify(
                    status,
                    &body,
                    &final_url,
                    &self.endpoint,
                    location.as_deref(),
                );
                Event {
                    phase: "request",
                    ix,
                    t_ms,
                    started_at: started
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default(),
                    duration_ms,
                    status: Some(status),
                    len: Some(len),
                    classify: classify.to_owned(),
                    final_url: Some(final_url.to_string()),
                    location,
                    body_marker: body_marker(&body, status),
                    error: None,
                }
            }
            Err(error) => Event {
                phase: "request",
                ix,
                t_ms,
                started_at: started
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                duration_ms,
                status: None,
                len: None,
                classify: format!("error:{}", short_error(&error)),
                final_url: None,
                location: None,
                body_marker: None,
                error: Some(error.to_string()),
            },
        }
    }
}

fn classify(
    status: u16,
    body: &str,
    final_url: &Url,
    endpoint: &Url,
    location: Option<&str>,
) -> &'static str {
    match status {
        202 => return "blocked:http_202",
        403 => return "blocked:http_403",
        429 => return "blocked:http_429",
        500..=599 => return "remote:http_5xx",
        _ => {}
    }
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("anomaly_modal")
        || lowered.contains("captcha")
        || lowered.contains("are you a robot")
    {
        return "blocked:anomaly";
    }
    if let Some(loc) = location
        && let Ok(target) = Url::parse(loc)
        && target.host_str() != endpoint.host_str()
    {
        return "blocked:redirect";
    }
    if (300..400).contains(&status) && final_url.host_str() != endpoint.host_str() {
        return "blocked:redirect";
    }
    if status == 200 {
        if lowered.contains("class=\"result")
            || lowered.contains("class=\"web-result")
            || lowered.contains("class=\"result__a\"")
        {
            return "ok";
        }
        if lowered.contains("no results") || lowered.contains("not find any results") {
            return "ok_empty";
        }
        return "remote:unknown_200";
    }
    "remote:other"
}

fn body_marker(body: &str, status: u16) -> Option<String> {
    if status != 200 && body.len() < 4096 {
        let snippet: String = body.chars().take(240).collect();
        return Some(snippet.replace('\n', " "));
    }
    if status == 200 {
        let lowered = body.to_ascii_lowercase();
        if lowered.contains("class=\"result__a\"") {
            return Some("results".to_owned());
        }
        if lowered.contains("anomaly_modal") {
            return Some("anomaly_modal".to_owned());
        }
        if lowered.contains("no results") {
            return Some("no_results".to_owned());
        }
    }
    None
}

fn short_error(error: &wreq::Error) -> String {
    let raw = error.to_string();
    raw.chars().take(64).collect::<String>().replace(':', "_")
}

#[derive(Serialize)]
struct Event {
    phase: &'static str,
    ix: u32,
    t_ms: u64,
    started_at: String,
    duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    len: Option<usize>,
    classify: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body_marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Default, Serialize)]
struct ScenarioSummary {
    total: u32,
    ok: u32,
    ok_empty: u32,
    blocked: u32,
    remote: u32,
    errored: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_block_ix: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_block_t_ms: Option<u64>,
}

#[derive(Serialize)]
struct SummaryEnvelope {
    phase: &'static str,
    scenario: String,
    label: String,
    started_at: String,
    finished_at: String,
    params: ParamsEcho,
    summary: ScenarioSummary,
}

#[derive(Serialize)]
struct ParamsEcho {
    endpoint: String,
    query: String,
    region: String,
    user_agent: Option<String>,
    emulation: bool,
    interval_ms: u64,
    count: u32,
    parallel: u32,
    rounds: u32,
    gap_s: u64,
    rate_ms: u64,
    duration_s: u64,
    probe_interval_s: u64,
    max_attempts: u32,
}

fn clone_args(args: &Args) -> Args {
    Args {
        endpoint: args.endpoint.clone(),
        query: args.query.clone(),
        region: args.region.clone(),
        user_agent: args.user_agent.clone(),
        emulation: args.emulation,
        label: args.label.clone(),
        out: args.out.clone(),
        interval_ms: args.interval_ms,
        count: args.count,
        parallel: args.parallel,
        rounds: args.rounds,
        gap_s: args.gap_s,
        rate_ms: args.rate_ms,
        duration_s: args.duration_s,
        probe_interval_s: args.probe_interval_s,
        max_attempts: args.max_attempts,
    }
}
