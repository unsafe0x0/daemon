use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;

const API_URL: &str = "https://coredump.vercel.app/api/activity";
const CHECK_INTERVAL: Duration = Duration::from_secs(5);
const SEND_INTERVAL: Duration = Duration::from_secs(45);
const MIN_SEND_DURATION: Duration = Duration::from_secs(30);
const IDLE_THRESHOLD: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    private_key: String,
}

#[derive(Debug, Clone)]
struct ActivityTracker {
    language_times: HashMap<String, Duration>,
    last_activity: Instant,
    last_sent: Instant,
    current_language: Option<String>,
    current_file: Option<String>,
}

impl ActivityTracker {
    fn new() -> Self {
        Self {
            language_times: HashMap::new(),
            last_activity: Instant::now(),
            last_sent: Instant::now(),
            current_language: None,
            current_file: None,
        }
    }

    fn record_activity(&mut self, language: String, filename: String) -> bool {
        let now = Instant::now();
        let time_since_last = now.duration_since(self.last_activity);
        let mut activity_changed = false;

        if time_since_last < IDLE_THRESHOLD {
            if let Some(lang) = &self.current_language {
                let entry = self
                    .language_times
                    .entry(lang.clone())
                    .or_insert(Duration::ZERO);
                *entry += time_since_last;
            }
        }

        if self.current_file.as_ref() != Some(&filename)
            || self.current_language.as_ref() != Some(&language)
        {
            activity_changed = true;
        }

        self.current_language = Some(language);
        self.current_file = Some(filename);
        self.last_activity = now;

        activity_changed
    }

    fn should_send(&self) -> bool {
        Instant::now().duration_since(self.last_sent) >= SEND_INTERVAL
    }

    fn get_and_reset(&mut self) -> HashMap<String, Duration> {
        self.last_sent = Instant::now();
        let data = self.language_times.clone();
        self.language_times.clear();
        data
    }
}

fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config/coredump/config.toml")
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        eprintln!("Config file not found at: {}", config_path.display());
        eprintln!("Please create it with your private key:");
        eprintln!("  mkdir -p ~/.config/coredump");
        eprintln!("  echo 'private_key = \"your-key-here\"' > ~/.config/coredump/config.toml");
        std::process::exit(1);
    }

    let content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

fn get_active_window_pid() -> Option<u32> {
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowpid"])
        .output()
        .ok()?;

    if output.status.success() {
        let pid_str = String::from_utf8_lossy(&output.stdout);
        pid_str.trim().parse().ok()
    } else {
        None
    }
}

fn get_process_name(pid: u32) -> Option<String> {
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let cmdline = fs::read_to_string(cmdline_path).ok()?;

    let name = cmdline.split('\0').next()?.split('/').last()?.to_string();

    Some(name)
}

fn is_zed_active() -> bool {
    if let Some(pid) = get_active_window_pid() {
        if let Some(name) = get_process_name(pid) {
            return name.contains("zed") || name == "Zed";
        }
    }
    false
}

fn get_current_file() -> Option<String> {
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
        .ok()?;

    if output.status.success() {
        let title = String::from_utf8_lossy(&output.stdout);
        let title = title.trim();

        let filename = if title.contains(" — ") {
            title.split(" — ").last()
        } else {
            title.split(" - ").next()
        };

        if let Some(filename) = filename {
            let filename = filename.trim();
            if !filename.is_empty() && filename != "Zed" {
                return Some(filename.to_string());
            }
        }
    }
    None
}

fn detect_language(filename: &str) -> String {
    let extension = filename.split('.').last().unwrap_or("");

    match extension {
        "rs" => "rust",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "jsx" => "javascriptreact",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "cpp" | "cc" | "cxx" => "cpp",
        "c" => "c",
        "h" | "hpp" => "cpp",
        "cs" => "csharp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "sh" | "bash" => "bash",
        "html" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "plaintext",
        "xml" => "plaintext",
        "md" => "markdown",
        "sql" => "sql",
        "vim" => "plaintext",
        "lua" => "lua",
        "r" => "r",
        "dart" => "dart",
        "ex" | "exs" => "plaintext",
        "erl" => "plaintext",
        "clj" | "cljs" => "plaintext",
        "hs" => "haskell",
        "ml" => "plaintext",
        "elm" => "plaintext",
        "vue" => "plaintext",
        "svelte" => "plaintext",
        _ => "plaintext",
    }
    .to_string()
}

fn get_display_name(lang: &str) -> &str {
    match lang {
        "rust" => "Rust",
        "javascript" => "JS",
        "typescript" => "TS",
        "typescriptreact" => "TSX",
        "javascriptreact" => "JSX",
        "python" => "Python",
        "go" => "Go",
        "java" => "Java",
        "cpp" => "C++",
        "c" => "C",
        "csharp" => "C#",
        "ruby" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kotlin" => "Kotlin",
        "scala" => "Scala",
        "bash" => "Bash",
        "html" => "HTML",
        "css" => "CSS",
        "scss" => "SCSS",
        "json" => "JSON",
        "yaml" => "YAML",
        "markdown" => "MD",
        "sql" => "SQL",
        "lua" => "Lua",
        "r" => "R",
        "dart" => "Dart",
        "haskell" => "Haskell",
        "plaintext" => "Text",
        _ => "Unknown",
    }
}

async fn send_activity(
    config: &Config,
    language: String,
    minutes: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "privateKey": config.private_key,
        "languageName": language,
        "timeSpent": minutes
    });

    let response = client
        .post(API_URL)
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;

    let display = get_display_name(&language);
    if response.status().is_success() {
        println!("✓ Sent {:.2}m of {}", minutes, display);
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read response".to_string());
        eprintln!("✗ Failed to send: {} - {}", status, body);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("CoreDump - Starting...");

    if Command::new("xdotool").arg("--version").output().is_err() {
        eprintln!("Error: xdotool is required but not installed.");
        eprintln!("Install it with: sudo apt-get install xdotool");
        std::process::exit(1);
    }

    let config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Config loaded");
    println!("✓ Monitoring...");

    tokio::time::sleep(Duration::from_millis(100)).await;

    if is_zed_active() {
        if let Some(filename) = get_current_file() {
            let language = detect_language(&filename);
            let display = get_display_name(&language);
            println!("→ {} [{}]", filename, display);
        }
    }

    let tracker = Arc::new(Mutex::new(ActivityTracker::new()));
    let tracker_clone = tracker.clone();

    tokio::spawn(async move {
        let mut ticker = interval(CHECK_INTERVAL);
        let mut last_periodic_log = Instant::now();

        loop {
            ticker.tick().await;

            if is_zed_active() {
                if let Some(filename) = get_current_file() {
                    let language = detect_language(&filename);
                    let mut tracker = tracker_clone.lock().unwrap();
                    let activity_changed =
                        tracker.record_activity(language.clone(), filename.clone());

                    let display = get_display_name(&language);

                    if activity_changed {
                        println!("→ {} [{}]", filename, display);
                        last_periodic_log = Instant::now();
                    } else if last_periodic_log.elapsed() >= Duration::from_secs(300) {
                        println!("→ {} [{}]", filename, display);
                        last_periodic_log = Instant::now();
                    }
                }
            }
        }
    });

    let mut send_ticker = interval(SEND_INTERVAL);
    loop {
        send_ticker.tick().await;

        let mut tracker = tracker.lock().unwrap();
        if tracker.should_send() {
            let data = tracker.get_and_reset();
            drop(tracker);

            for (language, duration) in data {
                if duration >= MIN_SEND_DURATION {
                    let minutes = duration.as_secs_f64() / 60.0;
                    if let Err(e) = send_activity(&config, language, minutes).await {
                        eprintln!("Error sending activity: {}", e);
                    }
                }
            }
        }
    }
}
