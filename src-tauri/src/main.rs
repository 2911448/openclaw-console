use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SessionRecord {
  session_id: String,
  updated_at: Option<u64>,
  label: Option<String>,
  kind: Option<String>,
  model: Option<String>,
  model_provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionView {
  session_key: String,
  session_id: String,
  label: Option<String>,
  kind: Option<String>,
  model: Option<String>,
  model_provider: Option<String>,
  updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalStatus {
  openclaw_found: bool,
  openclaw_version: Option<String>,
  openclaw_path: Option<String>,
  resolved_gateway_port: Option<u16>,
  sessions_dir: String,
  sessions_file_exists: bool,
  sessions_file_readable: bool,
  sessions_count: usize,
  latest_session_updated_at: Option<String>,
  errors: Vec<String>,
  gateway_state: Option<String>,
  gateway_rpc_ok: bool,
  gateway_rpc_error: Option<String>,
  current_channel: Option<String>,
  configured_channels: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionResult {
  ok: bool,
  command: String,
  stdout: String,
  stderr: String,
  error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentItem {
  agent_id: String,
  sessions_count: usize,
  latest_updated_at: Option<String>,
  sessions_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentStatusItem {
  agent_id: String,
  running: bool,
  status: String,
  sessions_count: usize,
  latest_updated_at: Option<String>,
  bootstrap_pending: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentPanelData {
  default_agent_id: Option<String>,
  items: Vec<AgentStatusItem>,
  errors: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GatewayLogs {
  lines: Vec<String>,
  fetched_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GatewayPortItem {
  port: u16,
  pid: Option<i32>,
  command: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CronJobItem {
  id: String,
  name: Option<String>,
  schedule: Option<String>,
  timezone: Option<String>,
  enabled: bool,
  next_run_at: Option<String>,
  last_run_at: Option<String>,
  last_status: Option<String>,
  last_duration_ms: Option<u64>,
  consecutive_failures: u32,
  workspace: Option<String>,
  prompt: Option<String>,
  updated_at: Option<String>,
  risk_flags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CronRunItem {
  job_id: String,
  started_at: Option<String>,
  finished_at: Option<String>,
  status: Option<String>,
  duration_ms: Option<u64>,
  exit_code: Option<i64>,
  error: Option<String>,
  summary: Option<String>,
  raw_line: Option<String>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct CronSchedulerStatus {
  state: Option<String>,
  enabled: Option<bool>,
  running: Option<bool>,
  timezone: Option<String>,
  last_tick_at: Option<String>,
  next_tick_at: Option<String>,
  worker_count: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CronPanelData {
  scheduler: CronSchedulerStatus,
  jobs: Vec<CronJobItem>,
  recent_runs: Vec<CronRunItem>,
  errors: Vec<String>,
  fetched_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SkillsPayload {
  workspace_dir: Option<String>,
  managed_skills_dir: Option<String>,
  skills: Vec<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelOption {
  provider: String,
  model: String,
  value: String,
  label: String,
  source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenclawConfigPayload {
  path: String,
  content: String,
}

fn home_dir() -> Result<PathBuf, String> {
  std::env::var("HOME")
    .map(PathBuf::from)
    .map_err(|_| "cannot resolve HOME".to_string())
}

fn sessions_file() -> Result<PathBuf, String> {
  Ok(home_dir()?.join(".openclaw/agents/main/sessions/sessions.json"))
}

fn sessions_dir() -> Result<PathBuf, String> {
  Ok(home_dir()?.join(".openclaw/agents/main/sessions"))
}

fn agents_root_dir() -> Result<PathBuf, String> {
  Ok(home_dir()?.join(".openclaw/agents"))
}

fn openclaw_config_file() -> Result<PathBuf, String> {
  Ok(home_dir()?.join(".openclaw/openclaw.json"))
}

fn resolve_openclaw_path() -> Option<PathBuf> {
  if let Ok(raw) = std::env::var("OPENCLAW_BIN") {
    let p = PathBuf::from(raw.trim());
    if p.exists() {
      return Some(p);
    }
  }

  let mut candidates = vec![
    PathBuf::from("/opt/homebrew/bin/openclaw"),
    PathBuf::from("/usr/local/bin/openclaw"),
    PathBuf::from("/usr/bin/openclaw"),
  ];
  if let Ok(home) = std::env::var("HOME") {
    candidates.push(PathBuf::from(format!("{}/.local/bin/openclaw", home)));
  }
  for p in candidates {
    if p.exists() {
      return Some(p);
    }
  }

  let hit = Command::new("which")
    .arg("openclaw")
    .output()
    .ok()
    .and_then(|o| {
      if !o.status.success() {
        return None;
      }
      let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
      if p.is_empty() { None } else { Some(PathBuf::from(p)) }
    });
  if hit.is_some() {
    return hit;
  }

  None
}

fn runtime_path() -> String {
  let fallback = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".to_string();
  let existing = std::env::var("PATH").unwrap_or_default();
  if existing.is_empty() {
    return fallback;
  }
  if existing.contains("/opt/homebrew/bin") && existing.contains("/usr/local/bin") {
    return existing;
  }
  format!("{}/{}", fallback, existing)
}

fn apply_runtime_env(cmd: &mut Command, gateway_port: Option<u16>) {
  cmd.env("PATH", runtime_path());
  if let Some(p) = gateway_port {
    cmd.env("OPENCLAW_GATEWAY_PORT", p.to_string());
  }
}

fn parse_listener_port(s: &str) -> Option<u16> {
  let trimmed = s.trim();
  let colon = trimmed.rfind(':')?;
  let tail = &trimmed[colon + 1..];
  let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
  if digits.is_empty() {
    return None;
  }
  digits.parse::<u16>().ok()
}

fn parse_port_from_url(raw: &str) -> Option<u16> {
  let s = raw.trim();
  let after_scheme = s.split_once("://").map(|(_, rhs)| rhs).unwrap_or(s);
  let host_port = after_scheme.split('/').next().unwrap_or(after_scheme);
  let port_str = host_port.rsplit(':').next()?;
  port_str.parse::<u16>().ok()
}

fn extract_gateway_port(j: &Value) -> Option<u16> {
  let paths: &[&[&str]] = &[
    &["gateway", "bindings", "port"],
    &["gateway", "port"],
    &["service", "gateway", "bindings", "port"],
    &["service", "gateway", "port"],
    &["port"],
  ];
  for path in paths {
    let mut cur = j;
    let mut ok = true;
    for key in *path {
      if let Some(next) = cur.get(*key) {
        cur = next;
      } else {
        ok = false;
        break;
      }
    }
    if ok {
      if let Some(v) = cur.as_u64().and_then(|n| u16::try_from(n).ok()) {
        return Some(v);
      }
      if let Some(s) = cur.as_str() {
        if let Ok(v) = s.parse::<u16>() {
          return Some(v);
        }
      }
    }
  }
  if let Some(url) = j
    .get("rpc")
    .and_then(|x| x.get("url"))
    .and_then(|x| x.as_str())
  {
    return parse_port_from_url(url);
  }
  None
}

fn discover_gateway_ports() -> Vec<GatewayPortItem> {
  let mut ps_cmd = Command::new("/bin/ps");
  ps_cmd.args(["-Ao", "pid=,command="]);
  apply_runtime_env(&mut ps_cmd, None);
  let ps_out = match ps_cmd.output() {
    Ok(o) if o.status.success() => o,
    _ => return Vec::new(),
  };

  let mut openclaw_pids = HashSet::<i32>::new();
  let mut pid_cmd = HashMap::<i32, String>::new();
  for line in String::from_utf8_lossy(&ps_out.stdout).lines() {
    let t = line.trim();
    if t.is_empty() {
      continue;
    }
    let mut it = t.split_whitespace();
    let pid = match it.next().and_then(|x| x.parse::<i32>().ok()) {
      Some(v) => v,
      None => continue,
    };
    let cmdline = t.split_once(' ').map(|(_, rhs)| rhs.trim().to_string()).unwrap_or_default();
    if cmdline.to_lowercase().contains("openclaw") {
      openclaw_pids.insert(pid);
      pid_cmd.insert(pid, cmdline);
    }
  }
  if openclaw_pids.is_empty() {
    return Vec::new();
  }

  let mut lsof_cmd = Command::new("/usr/sbin/lsof");
  lsof_cmd.args(["-nP", "-iTCP", "-sTCP:LISTEN"]);
  apply_runtime_env(&mut lsof_cmd, None);
  let lsof_out = match lsof_cmd.output() {
    Ok(o) if o.status.success() => o,
    _ => return Vec::new(),
  };

  let mut out = Vec::<GatewayPortItem>::new();
  let mut seen_port = HashSet::<u16>::new();
  for line in String::from_utf8_lossy(&lsof_out.stdout).lines().skip(1) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
      continue;
    }
    let pid = match parts[1].parse::<i32>() {
      Ok(v) => v,
      Err(_) => continue,
    };
    if !openclaw_pids.contains(&pid) {
      continue;
    }
    let name_col = parts.last().copied().unwrap_or("");
    let port = match parse_listener_port(name_col) {
      Some(v) => v,
      None => continue,
    };
    if seen_port.insert(port) {
      out.push(GatewayPortItem {
        port,
        pid: Some(pid),
        command: pid_cmd.get(&pid).cloned(),
      });
    }
  }

  out.sort_by(|a, b| a.port.cmp(&b.port));
  out
}

fn read_sessions_map() -> Result<HashMap<String, SessionRecord>, String> {
  let fp = sessions_file()?;
  let raw = fs::read_to_string(&fp)
    .map_err(|e| format!("failed reading {}: {}", fp.display(), e))?;
  serde_json::from_str::<HashMap<String, SessionRecord>>(&raw)
    .map_err(|e| format!("invalid sessions.json: {}", e))
}

fn format_updated_at(ms: Option<u64>) -> Option<String> {
  ms.map(|v| v.to_string())
}

fn as_text(v: &Value) -> Option<String> {
  if let Some(s) = v.as_str() {
    let t = s.trim();
    return if t.is_empty() { None } else { Some(t.to_string()) };
  }
  if let Some(n) = v.as_i64() {
    return Some(n.to_string());
  }
  if let Some(n) = v.as_u64() {
    return Some(n.to_string());
  }
  if let Some(b) = v.as_bool() {
    return Some(if b { "true".to_string() } else { "false".to_string() });
  }
  None
}

fn pick_string(obj: &Value, keys: &[&str]) -> Option<String> {
  for k in keys {
    if let Some(v) = obj.get(*k).and_then(as_text) {
      return Some(v);
    }
  }
  None
}

fn pick_u64(obj: &Value, keys: &[&str]) -> Option<u64> {
  for k in keys {
    if let Some(v) = obj.get(*k) {
      if let Some(n) = v.as_u64() {
        return Some(n);
      }
      if let Some(n) = v.as_i64() {
        if n >= 0 {
          return Some(n as u64);
        }
      }
      if let Some(s) = v.as_str().and_then(|x| x.trim().parse::<u64>().ok()) {
        return Some(s);
      }
    }
  }
  None
}

fn pick_i64(obj: &Value, keys: &[&str]) -> Option<i64> {
  for k in keys {
    if let Some(v) = obj.get(*k) {
      if let Some(n) = v.as_i64() {
        return Some(n);
      }
      if let Some(n) = v.as_u64() {
        if let Ok(c) = i64::try_from(n) {
          return Some(c);
        }
      }
      if let Some(s) = v.as_str().and_then(|x| x.trim().parse::<i64>().ok()) {
        return Some(s);
      }
    }
  }
  None
}

fn pick_bool(obj: &Value, keys: &[&str]) -> Option<bool> {
  for k in keys {
    if let Some(v) = obj.get(*k) {
      if let Some(b) = v.as_bool() {
        return Some(b);
      }
      if let Some(s) = v.as_str() {
        let t = s.trim().to_lowercase();
        if t == "true" || t == "enabled" || t == "running" {
          return Some(true);
        }
        if t == "false" || t == "disabled" || t == "stopped" {
          return Some(false);
        }
      }
      if let Some(n) = v.as_u64() {
        return Some(n != 0);
      }
    }
  }
  None
}

fn job_risk_flags(job: &CronJobItem) -> Vec<String> {
  let mut flags = Vec::<String>::new();
  if !job.enabled {
    flags.push("disabled".to_string());
  }
  if job.consecutive_failures >= 3 {
    flags.push("failing".to_string());
  }
  if job.next_run_at.as_deref().unwrap_or("").is_empty() {
    flags.push("no-next-run".to_string());
  }
  if let Some(s) = job.last_status.as_deref() {
    let low = s.to_lowercase();
    if low.contains("fail") || low.contains("error") || low.contains("timeout") {
      flags.push("last-run-failed".to_string());
    }
  } else {
    flags.push("never-ran".to_string());
  }
  flags
}

fn parse_cron_scheduler_status(v: &Value) -> CronSchedulerStatus {
  let state = pick_string(v, &["state", "status", "schedulerState"]);
  let running = pick_bool(v, &["running", "isRunning"]).or_else(|| {
    state
      .as_deref()
      .map(|s| s.eq_ignore_ascii_case("running") || s.eq_ignore_ascii_case("active"))
  });
  let enabled = pick_bool(v, &["enabled", "isEnabled"]).or_else(|| {
    state
      .as_deref()
      .map(|s| !s.eq_ignore_ascii_case("disabled"))
  });
  CronSchedulerStatus {
    state,
    enabled,
    running,
    timezone: pick_string(v, &["timezone", "tz"]),
    last_tick_at: pick_string(v, &["lastTickAt", "lastTick", "lastHeartbeatAt"]),
    next_tick_at: pick_string(v, &["nextTickAt", "nextTick"]),
    worker_count: pick_u64(v, &["workerCount", "workers"])
      .and_then(|n| u32::try_from(n).ok()),
  }
}

fn parse_cron_jobs(v: &Value) -> Vec<CronJobItem> {
  let arr = v
    .get("jobs")
    .and_then(|x| x.as_array())
    .or_else(|| v.get("items").and_then(|x| x.as_array()))
    .or_else(|| v.as_array());
  let mut out = Vec::<CronJobItem>::new();
  if let Some(list) = arr {
    for raw in list {
      let node = raw.get("job").unwrap_or(raw);
      let id = pick_string(node, &["id", "jobId", "cronId"]).unwrap_or_default();
      if id.is_empty() {
        continue;
      }

      let last_run = node.get("lastRun").unwrap_or(&Value::Null);
      let mut item = CronJobItem {
        id: id.clone(),
        name: pick_string(node, &["name", "title", "label"]).or_else(|| Some(id.clone())),
        schedule: pick_string(node, &["schedule", "cron", "cronExpr", "expression", "rrule"]),
        timezone: pick_string(node, &["timezone", "tz"]),
        enabled: pick_bool(node, &["enabled", "isEnabled", "active"])
          .or_else(|| {
            pick_string(node, &["status"]).map(|s| !s.eq_ignore_ascii_case("disabled"))
          })
          .unwrap_or(true),
        next_run_at: pick_string(node, &["nextRunAt", "nextRun", "nextDueAt", "nextExecutionAt"]),
        last_run_at: pick_string(node, &["lastRunAt", "lastExecutedAt", "lastFinishedAt"])
          .or_else(|| pick_string(last_run, &["finishedAt", "startedAt", "createdAt"])),
        last_status: pick_string(node, &["lastStatus"])
          .or_else(|| pick_string(last_run, &["status", "result", "outcome"]))
          .or_else(|| pick_string(node, &["status"])),
        last_duration_ms: pick_u64(node, &["lastDurationMs", "durationMs", "lastDuration"])
          .or_else(|| pick_u64(last_run, &["durationMs", "duration"])),
        consecutive_failures: pick_u64(node, &["consecutiveFailures", "failureStreak", "failures"])
          .unwrap_or(0) as u32,
        workspace: pick_string(node, &["workspace", "workspaceDir", "cwd", "path"]),
        prompt: pick_string(node, &["prompt", "task", "command", "message"]),
        updated_at: pick_string(node, &["updatedAt", "updatedTs", "updated"]),
        risk_flags: Vec::new(),
      };
      item.risk_flags = job_risk_flags(&item);
      out.push(item);
    }
  }
  out.sort_by(|a, b| b.next_run_at.cmp(&a.next_run_at));
  out
}

fn parse_cron_runs_output(job_id: &str, stdout: &str) -> Vec<CronRunItem> {
  let mut out = Vec::<CronRunItem>::new();
  for line in stdout.lines() {
    let t = line.trim();
    if t.is_empty() {
      continue;
    }
    if let Ok(v) = serde_json::from_str::<Value>(t) {
      let status = pick_string(&v, &["status", "result", "outcome"]).or_else(|| {
        pick_bool(&v, &["ok", "success"]).map(|b| if b { "ok".to_string() } else { "failed".to_string() })
      });
      out.push(CronRunItem {
        job_id: pick_string(&v, &["id", "jobId", "cronId"]).unwrap_or_else(|| job_id.to_string()),
        started_at: pick_string(&v, &["startedAt", "startAt", "timestamp", "ts", "createdAt"]),
        finished_at: pick_string(&v, &["finishedAt", "endAt", "completedAt"]),
        status,
        duration_ms: pick_u64(&v, &["durationMs", "duration"]),
        exit_code: pick_i64(&v, &["exitCode", "code"]),
        error: pick_string(&v, &["error", "errorMessage"]),
        summary: pick_string(&v, &["summary", "message", "text"]),
        raw_line: Some(t.to_string()),
      });
      continue;
    }
    let low = t.to_lowercase();
    let status = if low.contains("fail") || low.contains("error") {
      Some("failed".to_string())
    } else if low.contains("success") || low.contains("ok") {
      Some("ok".to_string())
    } else {
      None
    };
    out.push(CronRunItem {
      job_id: job_id.to_string(),
      started_at: None,
      finished_at: None,
      status,
      duration_ms: None,
      exit_code: None,
      error: None,
      summary: Some(t.to_string()),
      raw_line: Some(t.to_string()),
    });
  }
  out
}

#[tauri::command]
fn health() -> Value {
  serde_json::json!({ "ok": true })
}

#[tauri::command]
fn local_status(gateway_port: Option<u16>) -> Result<LocalStatus, String> {
  let sessions_dir = sessions_dir()?;
  let sessions_file = sessions_file()?;
  let mut errors = Vec::<String>::new();

  let openclaw_path = resolve_openclaw_path();
  let (openclaw_found, openclaw_version) = if let Some(bin) = openclaw_path.as_ref() {
    let mut cmd = Command::new(bin);
    cmd.arg("--version");
    apply_runtime_env(&mut cmd, gateway_port);
    match cmd.output() {
      Ok(o) => {
        let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
        (true, if text.is_empty() { None } else { Some(text) })
      }
      Err(_) => (false, None),
    }
  } else {
    (false, None)
  };
  if !openclaw_found {
    errors.push("openclaw command not found. checked /opt/homebrew/bin, /usr/local/bin and PATH".to_string());
  }

  let sessions_file_exists = sessions_file.exists();
  let sessions_file_readable = fs::read_to_string(&sessions_file).is_ok();
  if !sessions_file_exists {
    errors.push(format!("sessions file not found: {}", sessions_file.display()));
  } else if !sessions_file_readable {
    errors.push(format!("sessions file is not readable: {}", sessions_file.display()));
  }

  let (sessions_count, latest_session_updated_at) = match read_sessions_map() {
    Ok(map) => {
      let latest = map
        .values()
        .filter_map(|x| x.updated_at)
        .max()
        .map(|x| x.to_string());
      (map.len(), latest)
    }
    Err(e) => {
      errors.push(e);
      (0, None)
    }
  };

  let mut gateway_state: Option<String> = None;
  let mut gateway_rpc_ok = false;
  let mut gateway_rpc_error: Option<String> = None;
  let mut resolved_gateway_port = gateway_port;
  let mut current_channel: Option<String> = None;
  let mut configured_channels: Vec<String> = Vec::new();

  if let Ok((j, _stderr)) = run_openclaw_json(["gateway", "status", "--json"], gateway_port) {
    if resolved_gateway_port.is_none() {
      resolved_gateway_port = extract_gateway_port(&j);
    }
    gateway_state = j
      .get("service")
      .and_then(|x| x.get("runtime"))
      .and_then(|x| x.get("status"))
      .and_then(|x| x.as_str())
      .map(|s| s.to_string());
    gateway_rpc_ok = j
      .get("rpc")
      .and_then(|x| x.get("ok"))
      .and_then(|x| x.as_bool())
      .unwrap_or(false);
    gateway_rpc_error = j
      .get("rpc")
      .and_then(|x| x.get("error"))
      .and_then(|x| x.as_str())
      .map(|s| s.to_string());
  }

  if let Ok((j, _stderr)) = run_openclaw_json(["channels", "list", "--json"], gateway_port) {
    if let Some(chat) = j.get("chat").and_then(|x| x.as_object()) {
      'outer: for (provider, names) in chat {
        if let Some(list) = names.as_array() {
          for item in list {
            if let Some(name) = item.as_str() {
              configured_channels.push(format!("{}:{}", provider, name));
              current_channel = Some(format!("{}:{}", provider, name));
              break 'outer;
            }
          }
        }
      }
    }
  }

  Ok(LocalStatus {
    openclaw_found,
    openclaw_version,
    openclaw_path: openclaw_path.map(|p| p.display().to_string()),
    resolved_gateway_port,
    sessions_dir: sessions_dir.display().to_string(),
    sessions_file_exists,
    sessions_file_readable,
    sessions_count,
    latest_session_updated_at,
    errors,
    gateway_state,
    gateway_rpc_ok,
    gateway_rpc_error,
    current_channel,
    configured_channels,
  })
}

#[tauri::command]
fn list_gateway_ports() -> Vec<GatewayPortItem> {
  discover_gateway_ports()
}

#[tauri::command]
fn local_status_quick(gateway_port: Option<u16>) -> Result<LocalStatus, String> {
  let sessions_dir = sessions_dir()?;
  let sessions_file = sessions_file()?;
  let mut errors = Vec::<String>::new();

  let openclaw_path = resolve_openclaw_path();
  let (openclaw_found, openclaw_version) = if let Some(bin) = openclaw_path.as_ref() {
    let mut cmd = Command::new(bin);
    cmd.arg("--version");
    apply_runtime_env(&mut cmd, gateway_port);
    match cmd.output() {
      Ok(o) => {
        let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
        (true, if text.is_empty() { None } else { Some(text) })
      }
      Err(_) => (false, None),
    }
  } else {
    (false, None)
  };
  if !openclaw_found {
    errors.push("openclaw command not found. checked /opt/homebrew/bin, /usr/local/bin and PATH".to_string());
  }

  let sessions_file_exists = sessions_file.exists();
  let sessions_file_readable = fs::read_to_string(&sessions_file).is_ok();
  if !sessions_file_exists {
    errors.push(format!("sessions file not found: {}", sessions_file.display()));
  } else if !sessions_file_readable {
    errors.push(format!("sessions file is not readable: {}", sessions_file.display()));
  }

  let (sessions_count, latest_session_updated_at) = match read_sessions_map() {
    Ok(map) => {
      let latest = map
        .values()
        .filter_map(|x| x.updated_at)
        .max()
        .map(|x| x.to_string());
      (map.len(), latest)
    }
    Err(e) => {
      errors.push(e);
      (0, None)
    }
  };

  Ok(LocalStatus {
    openclaw_found,
    openclaw_version,
    openclaw_path: openclaw_path.map(|p| p.display().to_string()),
    resolved_gateway_port: gateway_port,
    sessions_dir: sessions_dir.display().to_string(),
    sessions_file_exists,
    sessions_file_readable,
    sessions_count,
    latest_session_updated_at,
    errors,
    gateway_state: None,
    gateway_rpc_ok: false,
    gateway_rpc_error: None,
    current_channel: None,
    configured_channels: Vec::new(),
  })
}

fn parse_json_from_mixed(s: &str) -> Result<Value, String> {
  if let Ok(v) = serde_json::from_str::<Value>(s) {
    return Ok(v);
  }
  if let Some(i) = s.find('{') {
    let tail = &s[i..];
    if let Ok(v) = serde_json::from_str::<Value>(tail) {
      return Ok(v);
    }
  }
  Err("json payload not found in output".to_string())
}

fn run_openclaw_with_port(args: &[&str], gateway_port: Option<u16>) -> Result<(String, String, bool), String> {
  let openclaw = resolve_openclaw_path()
    .ok_or_else(|| "openclaw command not found. checked /opt/homebrew/bin, /usr/local/bin and PATH".to_string())?;
  let mut cmd = Command::new(openclaw);
  cmd.args(args);
  apply_runtime_env(&mut cmd, gateway_port);
  let output = cmd.output().map_err(|e| format!("failed to run openclaw {:?}: {}", args, e))?;
  Ok((
    String::from_utf8_lossy(&output.stdout).to_string(),
    String::from_utf8_lossy(&output.stderr).to_string(),
    output.status.success(),
  ))
}

fn run_openclaw_json<const N: usize>(args: [&str; N], gateway_port: Option<u16>) -> Result<(Value, String), String> {
  let (stdout, stderr, ok) = run_openclaw_with_port(&args, gateway_port)?;
  if !ok {
    let data = if stderr.trim().is_empty() { stdout.clone() } else { stderr.clone() };
    return Err(data);
  }
  let parsed = parse_json_from_mixed(&stdout)?;
  Ok((parsed, stderr))
}

#[tauri::command]
fn list_agents() -> Result<Vec<AgentItem>, String> {
  let root = agents_root_dir()?;
  if !root.exists() {
    return Ok(Vec::new());
  }

  let mut out = Vec::<AgentItem>::new();
  let entries = fs::read_dir(&root).map_err(|e| format!("failed reading {}: {}", root.display(), e))?;
  for entry in entries {
    let entry = entry.map_err(|e| format!("read_dir entry error: {}", e))?;
    let agent_id = entry.file_name().to_string_lossy().to_string();
    let sessions_path = entry.path().join("sessions/sessions.json");
    if !sessions_path.exists() {
      continue;
    }

    let raw = match fs::read_to_string(&sessions_path) {
      Ok(v) => v,
      Err(_) => continue,
    };
    let map = match serde_json::from_str::<HashMap<String, SessionRecord>>(&raw) {
      Ok(v) => v,
      Err(_) => continue,
    };

    let latest = map
      .values()
      .filter_map(|x| x.updated_at)
      .max()
      .map(|x| x.to_string());
    out.push(AgentItem {
      agent_id,
      sessions_count: map.len(),
      latest_updated_at: latest,
      sessions_path: sessions_path.display().to_string(),
    });
  }

  out.sort_by(|a, b| b.latest_updated_at.cmp(&a.latest_updated_at));
  Ok(out)
}

#[tauri::command]
fn agent_panel_status(gateway_port: Option<u16>) -> Result<AgentPanelData, String> {
  let mut errors = Vec::<String>::new();
  let mut default_agent_id: Option<String> = None;
  let mut heartbeat_map = HashMap::<String, bool>::new();
  let mut items = Vec::<AgentStatusItem>::new();

  let (json, _stderr) = run_openclaw_json(["status", "--json"], gateway_port)
    .map_err(|e| format!("failed to fetch agent status: {}", e))?;

  if let Some(v) = json
    .get("agents")
    .and_then(|x| x.get("defaultId"))
    .and_then(|x| x.as_str())
  {
    default_agent_id = Some(v.to_string());
  }

  if let Some(arr) = json
    .get("heartbeat")
    .and_then(|x| x.get("agents"))
    .and_then(|x| x.as_array())
  {
    for it in arr {
      let id = it.get("agentId").and_then(|x| x.as_str()).unwrap_or("");
      if id.is_empty() {
        continue;
      }
      let enabled = it.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false);
      heartbeat_map.insert(id.to_string(), enabled);
    }
  }

  if let Some(arr) = json
    .get("agents")
    .and_then(|x| x.get("agents"))
    .and_then(|x| x.as_array())
  {
    for it in arr {
      let id = it.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
      if id.is_empty() {
        continue;
      }
      let sessions_count = it
        .get("sessionsCount")
        .and_then(|x| x.as_u64())
        .unwrap_or(0) as usize;
      let latest_updated_at = it
        .get("lastUpdatedAt")
        .and_then(|x| x.as_u64())
        .map(|x| x.to_string());
      let bootstrap_pending = it
        .get("bootstrapPending")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
      let running = *heartbeat_map.get(&id).unwrap_or(&false);
      let status = if bootstrap_pending {
        "bootstrap-pending".to_string()
      } else if running {
        "running".to_string()
      } else {
        "idle".to_string()
      };

      items.push(AgentStatusItem {
        agent_id: id,
        running,
        status,
        sessions_count,
        latest_updated_at,
        bootstrap_pending,
      });
    }
  } else {
    errors.push("agent list missing from status output".to_string());
  }

  items.sort_by(|a, b| b.latest_updated_at.cmp(&a.latest_updated_at));
  Ok(AgentPanelData {
    default_agent_id,
    items,
    errors,
  })
}

#[tauri::command]
fn restart_gateway(gateway_port: Option<u16>) -> Result<ActionResult, String> {
  let (stdout, stderr, ok) = run_openclaw_with_port(&["gateway", "restart", "--json"], gateway_port)?;
  if ok {
    return Ok(ActionResult {
      ok: true,
      command: "openclaw gateway restart --json".to_string(),
      stdout,
      stderr,
      error: None,
    });
  }
  let msg = if stderr.trim().is_empty() { stdout.clone() } else { stderr.clone() };
  Ok(ActionResult {
    ok: false,
    command: "openclaw gateway restart --json".to_string(),
    stdout,
    stderr,
    error: Some(msg),
  })
}

#[tauri::command]
fn run_doctor(gateway_port: Option<u16>) -> Result<ActionResult, String> {
  let (stdout, stderr, ok) = run_openclaw_with_port(&["doctor", "--non-interactive"], gateway_port)?;
  if ok {
    return Ok(ActionResult {
      ok: true,
      command: "openclaw doctor --non-interactive".to_string(),
      stdout,
      stderr,
      error: None,
    });
  }
  let msg = if stderr.trim().is_empty() { stdout.clone() } else { stderr.clone() };
  Ok(ActionResult {
    ok: false,
    command: "openclaw doctor --non-interactive".to_string(),
    stdout,
    stderr,
    error: Some(msg),
  })
}

#[tauri::command]
fn list_sessions() -> Result<Vec<SessionView>, String> {
  let mut out = Vec::new();
  let map = read_sessions_map()?;
  for (session_key, item) in map {
    out.push(SessionView {
      session_key,
      session_id: item.session_id,
      label: item.label,
      kind: item.kind,
      model: item.model,
      model_provider: item.model_provider,
      updated_at: format_updated_at(item.updated_at),
    });
  }

  out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
  Ok(out)
}

#[tauri::command]
fn session_history(session_key: String, limit: Option<usize>) -> Result<Vec<Value>, String> {
  let map = read_sessions_map()?;
  let session = map
    .get(&session_key)
    .ok_or_else(|| format!("session not found: {}", session_key))?;

  let transcript = sessions_dir()?.join(format!("{}.jsonl", session.session_id));
  let raw = fs::read_to_string(&transcript)
    .map_err(|e| format!("failed reading {}: {}", transcript.display(), e))?;

  let limit = limit.unwrap_or(200);
  let mut lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
  if lines.len() > limit {
    lines = lines.split_off(lines.len() - limit);
  }

  let mut items = Vec::new();
  for line in lines {
    match serde_json::from_str::<Value>(line) {
      Ok(v) => items.push(v),
      Err(_) => items.push(serde_json::json!({ "type": "raw", "raw": line })),
    }
  }

  Ok(items)
}

#[tauri::command]
async fn send_message(
  session_key: String,
  message: String,
  gateway_port: Option<u16>,
) -> Result<Value, String> {
  let msg = message.trim().to_string();
  if msg.is_empty() {
    return Err("message required".to_string());
  }

  tauri::async_runtime::spawn_blocking(move || {
    let map = read_sessions_map()?;
    let session = map
      .get(&session_key)
      .ok_or_else(|| format!("session not found: {}", session_key))?;

    let openclaw = resolve_openclaw_path()
      .ok_or_else(|| "openclaw command not found. checked /opt/homebrew/bin, /usr/local/bin and PATH".to_string())?;
    let args: Vec<&str> = vec![
      "agent",
      "--session-id",
      session.session_id.as_str(),
      "--message",
      msg.as_str(),
      "--json",
    ];
    let mut cmd = Command::new(&openclaw);
    cmd.args(&args);
    apply_runtime_env(&mut cmd, gateway_port);
    let output = cmd.output().map_err(|e| format!("failed to run openclaw: {}", e))?;

    if !output.status.success() {
      let err = String::from_utf8_lossy(&output.stderr).to_string();
      return Err(if err.trim().is_empty() {
        format!("openclaw exited with {}", output.status)
      } else {
        err
      });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str(&stdout).or_else(|_| Ok(serde_json::json!({ "raw": stdout })))
  })
  .await
  .map_err(|e| format!("send task join error: {}", e))?
}

#[tauri::command]
async fn gateway_logs(limit: Option<usize>, gateway_port: Option<u16>) -> Result<GatewayLogs, String> {
  let lim = limit.unwrap_or(120).clamp(20, 500).to_string();
  tauri::async_runtime::spawn_blocking(move || {
    let openclaw = resolve_openclaw_path()
      .ok_or_else(|| "openclaw command not found. checked /opt/homebrew/bin, /usr/local/bin and PATH".to_string())?;
    let mut cmd = Command::new(openclaw);
    cmd.args(["logs", "--plain", "--limit", lim.as_str(), "--timeout", "8000"]);
    apply_runtime_env(&mut cmd, gateway_port);
    let output = cmd.output().map_err(|e| format!("failed to read gateway logs: {}", e))?;
    if !output.status.success() {
      let err = String::from_utf8_lossy(&output.stderr).to_string();
      return Err(if err.trim().is_empty() {
        format!("openclaw logs exited with {}", output.status)
      } else {
        err
      });
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let lines = stdout
      .lines()
      .map(|s| s.to_string())
      .filter(|s| !s.trim().is_empty())
      .collect::<Vec<_>>();
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as i64)
      .unwrap_or(0);
    Ok(GatewayLogs {
      lines,
      fetched_at: now,
    })
  })
  .await
  .map_err(|e| format!("gateway logs task join error: {}", e))?
}

#[tauri::command]
async fn list_skills(gateway_port: Option<u16>) -> Result<SkillsPayload, String> {
  tauri::async_runtime::spawn_blocking(move || {
    let (stdout, stderr, ok) = run_openclaw_with_port(&["skills", "list", "--json"], gateway_port)?;
    if !ok {
      let msg = if stderr.trim().is_empty() { stdout } else { stderr };
      return Err(msg);
    }
    let j = parse_json_from_mixed(&stdout)?;
    let workspace_dir = j.get("workspaceDir").and_then(|x| x.as_str()).map(|s| s.to_string());
    let managed_skills_dir = j.get("managedSkillsDir").and_then(|x| x.as_str()).map(|s| s.to_string());
    let skills = j
      .get("skills")
      .and_then(|x| x.as_array())
      .cloned()
      .unwrap_or_default();
    Ok(SkillsPayload {
      workspace_dir,
      managed_skills_dir,
      skills,
    })
  })
  .await
  .map_err(|e| format!("list skills task join error: {}", e))?
}

#[tauri::command]
async fn cron_panel_status(limit_runs_per_job: Option<usize>, gateway_port: Option<u16>) -> Result<CronPanelData, String> {
  let per_job_limit = limit_runs_per_job.unwrap_or(8).clamp(2, 20);
  tauri::async_runtime::spawn_blocking(move || {
    let mut errors = Vec::<String>::new();
    let mut scheduler = CronSchedulerStatus::default();
    let mut jobs = Vec::<CronJobItem>::new();
    let mut recent_runs = Vec::<CronRunItem>::new();

    match run_openclaw_json(["cron", "status", "--json"], gateway_port) {
      Ok((j, _stderr)) => {
        scheduler = parse_cron_scheduler_status(&j);
      }
      Err(e) => errors.push(format!("cron status failed: {}", e)),
    }

    match run_openclaw_json(["cron", "list", "--all", "--json"], gateway_port) {
      Ok((j, _stderr)) => {
        jobs = parse_cron_jobs(&j);
      }
      Err(e) => errors.push(format!("cron list failed: {}", e)),
    }

    let candidate_ids = jobs
      .iter()
      .filter(|j| j.enabled || j.consecutive_failures > 0)
      .take(10)
      .map(|j| j.id.clone())
      .collect::<Vec<_>>();
    for id in candidate_ids {
      let lim = per_job_limit.to_string();
      let args = vec!["cron", "runs", "--id", id.as_str(), "--limit", lim.as_str()];
      match run_openclaw_with_port(&args, gateway_port) {
        Ok((stdout, stderr, ok)) => {
          if ok {
            recent_runs.extend(parse_cron_runs_output(&id, &stdout));
          } else {
            let msg = if stderr.trim().is_empty() { stdout } else { stderr };
            errors.push(format!("cron runs failed for {}: {}", id, msg.trim()));
          }
        }
        Err(e) => errors.push(format!("cron runs exec failed for {}: {}", id, e)),
      }
    }

    recent_runs.sort_by(|a, b| {
      b.started_at
        .as_ref()
        .or(b.finished_at.as_ref())
        .cmp(&a.started_at.as_ref().or(a.finished_at.as_ref()))
    });
    if recent_runs.len() > 120 {
      recent_runs.truncate(120);
    }

    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| d.as_millis() as i64)
      .unwrap_or(0);
    Ok(CronPanelData {
      scheduler,
      jobs,
      recent_runs,
      errors,
      fetched_at: now,
    })
  })
  .await
  .map_err(|e| format!("cron panel task join error: {}", e))?
}

#[tauri::command]
async fn cron_job_runs(job_id: String, limit: Option<usize>, gateway_port: Option<u16>) -> Result<Vec<CronRunItem>, String> {
  let lim = limit.unwrap_or(20).clamp(1, 100).to_string();
  tauri::async_runtime::spawn_blocking(move || {
    let args = vec!["cron", "runs", "--id", job_id.as_str(), "--limit", lim.as_str()];
    let (stdout, stderr, ok) = run_openclaw_with_port(&args, gateway_port)?;
    if !ok {
      let msg = if stderr.trim().is_empty() { stdout } else { stderr };
      return Err(msg);
    }
    Ok(parse_cron_runs_output(&job_id, &stdout))
  })
  .await
  .map_err(|e| format!("cron job runs task join error: {}", e))?
}

#[tauri::command]
fn cron_run_now(job_id: String, gateway_port: Option<u16>) -> Result<ActionResult, String> {
  let args = vec!["cron", "run", job_id.as_str()];
  let (stdout, stderr, ok) = run_openclaw_with_port(&args, gateway_port)?;
  let msg = if stderr.trim().is_empty() { stdout.clone() } else { stderr.clone() };
  Ok(ActionResult {
    ok,
    command: format!("openclaw cron run {}", job_id),
    stdout,
    stderr,
    error: if ok { None } else { Some(msg) },
  })
}

#[tauri::command]
fn cron_set_enabled(job_id: String, enabled: bool, gateway_port: Option<u16>) -> Result<ActionResult, String> {
  let sub = if enabled { "enable" } else { "disable" };
  let args = vec!["cron", sub, job_id.as_str()];
  let (stdout, stderr, ok) = run_openclaw_with_port(&args, gateway_port)?;
  let msg = if stderr.trim().is_empty() { stdout.clone() } else { stderr.clone() };
  Ok(ActionResult {
    ok,
    command: format!("openclaw cron {} {}", sub, job_id),
    stdout,
    stderr,
    error: if ok { None } else { Some(msg) },
  })
}

#[tauri::command]
fn cron_delete(job_id: String, gateway_port: Option<u16>) -> Result<ActionResult, String> {
  let args = vec!["cron", "rm", job_id.as_str(), "--json"];
  let (stdout, stderr, ok) = run_openclaw_with_port(&args, gateway_port)?;
  let msg = if stderr.trim().is_empty() { stdout.clone() } else { stderr.clone() };
  Ok(ActionResult {
    ok,
    command: format!("openclaw cron rm {} --json", job_id),
    stdout,
    stderr,
    error: if ok { None } else { Some(msg) },
  })
}

#[tauri::command]
fn list_models(_gateway_port: Option<u16>) -> Result<Vec<ModelOption>, String> {
  let mut out = Vec::<ModelOption>::new();
  let mut seen = std::collections::HashSet::<String>::new();

  // Primary source: ~/.openclaw/openclaw.json models.providers
  let cfg = openclaw_config_file()?;
  if let Ok(raw) = fs::read_to_string(&cfg) {
    if let Ok(j) = serde_json::from_str::<Value>(&raw) {
      if let Some(providers) = j
        .get("models")
        .and_then(|x| x.get("providers"))
        .and_then(|x| x.as_object())
      {
        for (provider, pnode) in providers {
          if let Some(models) = pnode.get("models").and_then(|x| x.as_array()) {
            for m in models {
              let model_id = m
                .get("id")
                .and_then(|x| x.as_str())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty());
              if let Some(model) = model_id {
                let value = format!("{}/{}", provider, model);
                if seen.insert(value.clone()) {
                  let name = m.get("name").and_then(|x| x.as_str()).unwrap_or(model);
                  out.push(ModelOption {
                    provider: provider.to_string(),
                    model: model.to_string(),
                    value: value.clone(),
                    label: format!("{} ({})", name, value),
                    source: "config".to_string(),
                  });
                }
              }
            }
          }
        }
      }
    }
  }

  out.sort_by(|a, b| a.value.cmp(&b.value));
  Ok(out)
}

#[tauri::command]
fn get_openclaw_config() -> Result<OpenclawConfigPayload, String> {
  let path = openclaw_config_file()?;
  let content = fs::read_to_string(&path)
    .map_err(|e| format!("failed reading {}: {}", path.display(), e))?;
  Ok(OpenclawConfigPayload {
    path: path.display().to_string(),
    content,
  })
}

#[tauri::command]
fn save_openclaw_config(content: String) -> Result<OpenclawConfigPayload, String> {
  let path = openclaw_config_file()?;
  // Validate JSON before writing.
  let v: Value = serde_json::from_str(&content).map_err(|e| format!("invalid JSON: {}", e))?;
  let pretty = serde_json::to_string_pretty(&v).map_err(|e| format!("format json failed: {}", e))?;
  fs::write(&path, format!("{}\n", pretty))
    .map_err(|e| format!("failed writing {}: {}", path.display(), e))?;
  Ok(OpenclawConfigPayload {
    path: path.display().to_string(),
    content: format!("{}\n", pretty),
  })
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      health,
      list_gateway_ports,
      local_status,
      local_status_quick,
      restart_gateway,
      run_doctor,
      list_agents,
      agent_panel_status,
      list_sessions,
      session_history,
      send_message,
      gateway_logs,
      list_skills,
      cron_panel_status,
      cron_job_runs,
      cron_run_now,
      cron_set_enabled,
      cron_delete,
      list_models,
      get_openclaw_config,
      save_openclaw_config,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
