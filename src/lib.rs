use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_DISPLAY: &str = ":99";
const DEFAULT_WIDTH: u32 = 1024;
const DEFAULT_HEIGHT: u32 = 768;
const DEFAULT_DEPTH: u32 = 24;
const DEFAULT_VIEW_INTERVAL_MS: u64 = 200;
const DEFAULT_START_WAIT_MS: u64 = 12_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Start { url: Option<String> },
    View,
    Click { x: u32, y: u32 },
    Move { x: u32, y: u32 },
    Down { x: u32, y: u32 },
    Up { x: u32, y: u32 },
    Type { text: String },
    Go { url: String },
    Back,
    Forward,
    Reload,
    Stop,
    Status,
    Help,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub display: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub view_interval: Duration,
    pub start_wait: Duration,
    pub state_dir: PathBuf,
    pub run_dir: PathBuf,
    pub profile_dir: PathBuf,
    pub capture_path: PathBuf,
    pub xvfb_bin: String,
    pub chromium_bin: String,
    pub scrot_bin: String,
    pub chafa_bin: String,
    pub xdotool_bin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStatus {
    pub xvfb_pid: Option<u32>,
    pub xvfb_alive: bool,
    pub chromium_pid: Option<u32>,
    pub chromium_alive: bool,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    let command = parse_args(&args)?;
    let config = Config::from_env()?;

    match command {
        CliCommand::Start { url } => start(&config, url.as_deref()),
        CliCommand::View => view(&config),
        CliCommand::Click { x, y } => pointer_command(&config, x, y, PointerAction::Click),
        CliCommand::Move { x, y } => pointer_command(&config, x, y, PointerAction::Move),
        CliCommand::Down { x, y } => pointer_command(&config, x, y, PointerAction::Down),
        CliCommand::Up { x, y } => pointer_command(&config, x, y, PointerAction::Up),
        CliCommand::Type { text } => type_text(&config, &text),
        CliCommand::Go { url } => go_url(&config, &url),
        CliCommand::Back => key(&config, "Alt+Left"),
        CliCommand::Forward => key(&config, "Alt+Right"),
        CliCommand::Reload => key(&config, "ctrl+r"),
        CliCommand::Stop => stop(&config),
        CliCommand::Status => print_status(&config),
        CliCommand::Help => {
            print_help();
            Ok(())
        }
    }
}

pub fn parse_args(args: &[String]) -> Result<CliCommand, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(CliCommand::Help);
    };

    match command {
        "start" => Ok(CliCommand::Start {
            url: args.get(1).cloned(),
        }),
        "view" => no_extra(args, CliCommand::View),
        "click" => coords(args).map(|(x, y)| CliCommand::Click { x, y }),
        "move" => coords(args).map(|(x, y)| CliCommand::Move { x, y }),
        "down" | "mousedown" => coords(args).map(|(x, y)| CliCommand::Down { x, y }),
        "up" | "mouseup" => coords(args).map(|(x, y)| CliCommand::Up { x, y }),
        "type" => one_text(args, "type").map(|text| CliCommand::Type { text }),
        "go" => one_text(args, "go").map(|url| CliCommand::Go { url }),
        "back" => no_extra(args, CliCommand::Back),
        "forward" => no_extra(args, CliCommand::Forward),
        "reload" => no_extra(args, CliCommand::Reload),
        "stop" => no_extra(args, CliCommand::Stop),
        "status" => no_extra(args, CliCommand::Status),
        "help" | "-h" | "--help" => Ok(CliCommand::Help),
        other => Err(format!("unknown command `{other}`")),
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let display = read_env("BRAUSI_DISPLAY").unwrap_or_else(|| DEFAULT_DISPLAY.to_string());
        let width = read_u32("BRAUSI_WIDTH", DEFAULT_WIDTH)?;
        let height = read_u32("BRAUSI_HEIGHT", DEFAULT_HEIGHT)?;
        let depth = read_u32("BRAUSI_DEPTH", DEFAULT_DEPTH)?;
        let view_interval = Duration::from_millis(read_u64(
            "BRAUSI_VIEW_INTERVAL_MS",
            DEFAULT_VIEW_INTERVAL_MS,
        )?);
        let start_wait =
            Duration::from_millis(read_u64("BRAUSI_START_WAIT_MS", DEFAULT_START_WAIT_MS)?);
        let state_dir = read_env("BRAUSI_STATE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(default_state_dir);
        let run_dir = state_dir.join("run");
        let profile_dir = state_dir.join("profile");
        let capture_path = run_dir.join("live.png");

        Ok(Self {
            display,
            width,
            height,
            depth,
            view_interval,
            start_wait,
            state_dir,
            run_dir,
            profile_dir,
            capture_path,
            xvfb_bin: read_env("BRAUSI_XVFB").unwrap_or_else(|| "Xvfb".to_string()),
            chromium_bin: read_env("BRAUSI_CHROMIUM").unwrap_or_else(default_chromium_bin),
            scrot_bin: read_env("BRAUSI_SCROT").unwrap_or_else(|| "scrot".to_string()),
            chafa_bin: read_env("BRAUSI_CHAFA").unwrap_or_else(|| "chafa".to_string()),
            xdotool_bin: read_env("BRAUSI_XDOTOOL").unwrap_or_else(|| "xdotool".to_string()),
        })
    }

    fn screen_arg(&self) -> String {
        format!("{}x{}x{}", self.width, self.height, self.depth)
    }
}

fn start(config: &Config, url: Option<&str>) -> Result<(), String> {
    ensure_dirs(config)?;
    stop_tracked(config)?;

    let xvfb = spawn_xvfb(config)?;
    write_pid(&pid_path(config, "xvfb"), xvfb.id())?;
    thread::sleep(Duration::from_millis(750));

    let chromium = match spawn_chromium(config, url) {
        Ok(child) => child,
        Err(error) => {
            let _ = stop_tracked(config);
            return Err(error);
        }
    };
    write_pid(&pid_path(config, "chromium"), chromium.id())?;

    thread::sleep(config.start_wait);
    println!("started");
    println!("display={}", config.display);
    println!("size={}x{}", config.width, config.height);
    println!("state={}", config.state_dir.display());
    Ok(())
}

fn view(config: &Config) -> Result<(), String> {
    ensure_running(config)?;
    ensure_dirs(config)?;

    loop {
        capture(config)?;
        if config.capture_path.is_file() {
            clear_screen();
            render(config)?;
            io::stdout().flush().map_err(|e| e.to_string())?;
        }
        thread::sleep(config.view_interval);
    }
}

#[derive(Copy, Clone)]
enum PointerAction {
    Click,
    Move,
    Down,
    Up,
}

fn pointer_command(config: &Config, x: u32, y: u32, action: PointerAction) -> Result<(), String> {
    validate_coords(config, x, y)?;
    ensure_running(config)?;

    let x = x.to_string();
    let y = y.to_string();
    let mut args = vec!["mousemove", x.as_str(), y.as_str()];
    match action {
        PointerAction::Click => args.extend(["click", "1"]),
        PointerAction::Move => {}
        PointerAction::Down => args.extend(["mousedown", "1"]),
        PointerAction::Up => args.extend(["mouseup", "1"]),
    }
    run_display_command(&config.xdotool_bin, &args, config)
}

fn type_text(config: &Config, text: &str) -> Result<(), String> {
    ensure_running(config)?;
    run_display_command(&config.xdotool_bin, &["type", text], config)
}

fn go_url(config: &Config, url: &str) -> Result<(), String> {
    ensure_running(config)?;
    run_display_command(&config.xdotool_bin, &["key", "ctrl+l"], config)?;
    run_display_command(&config.xdotool_bin, &["type", url], config)?;
    run_display_command(&config.xdotool_bin, &["key", "Return"], config)
}

fn key(config: &Config, key_name: &str) -> Result<(), String> {
    ensure_running(config)?;
    run_display_command(&config.xdotool_bin, &["key", key_name], config)
}

fn stop(config: &Config) -> Result<(), String> {
    stop_tracked(config)?;
    let _ = fs::remove_file(&config.capture_path);
    println!("stopped");
    Ok(())
}

fn print_status(config: &Config) -> Result<(), String> {
    let status = read_status(config);
    println!("state_dir={}", config.state_dir.display());
    println!("display={}", config.display);
    println!("xvfb={}", process_line(status.xvfb_pid, status.xvfb_alive));
    println!(
        "chromium={}",
        process_line(status.chromium_pid, status.chromium_alive)
    );
    Ok(())
}

fn spawn_xvfb(config: &Config) -> Result<Child, String> {
    Command::new(&config.xvfb_bin)
        .arg(&config.display)
        .args(["-screen", "0"])
        .arg(config.screen_arg())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to start Xvfb (`{}`): {e}", config.xvfb_bin))
}

fn spawn_chromium(config: &Config, url: Option<&str>) -> Result<Child, String> {
    let size = format!("{},{}", config.width, config.height);
    let user_data_dir = config.profile_dir.to_string_lossy().to_string();

    let mut command = Command::new(&config.chromium_bin);
    command
        .env("DISPLAY", &config.display)
        .args([
            "--kiosk",
            "--no-first-run",
            "--disable-gpu",
            "--no-sandbox",
            "--disable-dev-shm-usage",
            "--mute-audio",
            "--disable-software-rasterizer",
            "--disable-gpu-sandbox",
            "--use-gl=swiftshader",
        ])
        .arg(format!("--window-size={size}"))
        .arg(format!("--user-data-dir={user_data_dir}"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(url) = url.filter(|value| !value.trim().is_empty()) {
        command.arg(url);
    }

    command
        .spawn()
        .map_err(|e| format!("failed to start Chromium (`{}`): {e}", config.chromium_bin))
}

fn capture(config: &Config) -> Result<(), String> {
    let output = config.capture_path.to_string_lossy().to_string();
    run_display_command(&config.scrot_bin, &["-o", &output], config)
}

fn render(config: &Config) -> Result<(), String> {
    let size = terminal_size();
    let image = config.capture_path.to_string_lossy().to_string();
    let status = Command::new(&config.chafa_bin)
        .args([
            "--symbols=space",
            "--color-space=din99d",
            "--size",
            &size,
            &image,
        ])
        .status()
        .map_err(|e| format!("failed to run chafa (`{}`): {e}", config.chafa_bin))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("chafa exited with status {status}"))
    }
}

fn run_display_command<S: AsRef<OsStr>>(
    program: &str,
    args: &[S],
    config: &Config,
) -> Result<(), String> {
    let status = Command::new(program)
        .env("DISPLAY", &config.display)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run `{program}`: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program}` exited with status {status}"))
    }
}

fn stop_tracked(config: &Config) -> Result<(), String> {
    for name in ["chromium", "xvfb"] {
        let path = pid_path(config, name);
        if let Some(pid) = read_pid(&path)? {
            terminate_pid(pid);
        }
        let _ = fs::remove_file(path);
    }
    Ok(())
}

fn terminate_pid(pid: u32) {
    let pid_text = pid.to_string();
    let _ = Command::new("kill").arg(&pid_text).status();
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(2) {
        if !pid_alive(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    let _ = Command::new("kill").args(["-9", &pid_text]).status();
}

fn ensure_dirs(config: &Config) -> Result<(), String> {
    fs::create_dir_all(&config.run_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&config.profile_dir).map_err(|e| e.to_string())
}

fn ensure_running(config: &Config) -> Result<(), String> {
    let status = read_status(config);
    if status.xvfb_alive && status.chromium_alive {
        Ok(())
    } else {
        Err("browser is not running; use `brausi start [url]` first".to_string())
    }
}

fn read_status(config: &Config) -> ProcessStatus {
    let xvfb_pid = read_pid(&pid_path(config, "xvfb")).ok().flatten();
    let chromium_pid = read_pid(&pid_path(config, "chromium")).ok().flatten();
    ProcessStatus {
        xvfb_alive: xvfb_pid.map(pid_alive).unwrap_or(false),
        chromium_alive: chromium_pid.map(pid_alive).unwrap_or(false),
        xvfb_pid,
        chromium_pid,
    }
}

fn process_line(pid: Option<u32>, alive: bool) -> String {
    match (pid, alive) {
        (Some(pid), true) => format!("running pid={pid}"),
        (Some(pid), false) => format!("stale pid={pid}"),
        (None, _) => "stopped".to_string(),
    }
}

fn pid_path(config: &Config, name: &str) -> PathBuf {
    config.run_dir.join(format!("{name}.pid"))
}

fn write_pid(path: &Path, pid: u32) -> Result<(), String> {
    fs::write(path, format!("{pid}\n")).map_err(|e| e.to_string())
}

fn read_pid(path: &Path) -> Result<Option<u32>, String> {
    match fs::read_to_string(path) {
        Ok(value) => value
            .trim()
            .parse::<u32>()
            .map(Some)
            .map_err(|e| format!("invalid pid file `{}`: {e}", path.display())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn validate_coords(config: &Config, x: u32, y: u32) -> Result<(), String> {
    if x >= config.width || y >= config.height {
        return Err(format!(
            "coordinates out of bounds: {x},{y}; expected x=0..{} y=0..{}",
            config.width.saturating_sub(1),
            config.height.saturating_sub(1)
        ));
    }
    Ok(())
}

fn coords(args: &[String]) -> Result<(u32, u32), String> {
    if args.len() != 3 {
        return Err(format!("usage: brausi {} <x> <y>", args[0]));
    }
    let x = args[1]
        .parse::<u32>()
        .map_err(|_| format!("invalid x coordinate `{}`", args[1]))?;
    let y = args[2]
        .parse::<u32>()
        .map_err(|_| format!("invalid y coordinate `{}`", args[2]))?;
    Ok((x, y))
}

fn one_text(args: &[String], name: &str) -> Result<String, String> {
    if args.len() < 2 {
        return Err(format!("usage: brausi {name} <text>"));
    }
    Ok(args[1..].join(" "))
}

fn no_extra(args: &[String], command: CliCommand) -> Result<CliCommand, String> {
    if args.len() == 1 {
        Ok(command)
    } else {
        Err(format!("usage: brausi {}", args[0]))
    }
}

fn read_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_u32(name: &str, default: u32) -> Result<u32, String> {
    read_env(name)
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| format!("{name} must be an unsigned integer"))
        })
        .unwrap_or(Ok(default))
}

fn read_u64(name: &str, default: u64) -> Result<u64, String> {
    read_env(name)
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| format!("{name} must be an unsigned integer"))
        })
        .unwrap_or(Ok(default))
}

fn default_state_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".brausi")
}

fn default_chromium_bin() -> String {
    if cfg!(target_os = "macos") {
        "/Applications/Chromium.app/Contents/MacOS/Chromium".to_string()
    } else {
        "chromium".to_string()
    }
}

fn terminal_size() -> String {
    let cols = read_env("COLUMNS").and_then(|value| value.parse::<u32>().ok());
    let lines = read_env("LINES").and_then(|value| value.parse::<u32>().ok());
    match (cols, lines) {
        (Some(cols), Some(lines)) if cols > 0 && lines > 0 => format!("{cols}x{lines}"),
        _ => "80x24".to_string(),
    }
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
}

fn print_help() {
    println!(
        "brausi - lightweight terminal browser controller

Usage:
  brausi start [url]
  brausi view
  brausi click <x> <y>
  brausi move <x> <y>
  brausi down <x> <y>
  brausi up <x> <y>
  brausi type <text>
  brausi go <url>
  brausi back
  brausi forward
  brausi reload
  brausi status
  brausi stop

Environment:
  BRAUSI_DISPLAY=:99
  BRAUSI_WIDTH=1024
  BRAUSI_HEIGHT=768
  BRAUSI_STATE_DIR=~/.brausi
  BRAUSI_CHROMIUM=chromium
  BRAUSI_XVFB=Xvfb
  BRAUSI_SCROT=scrot
  BRAUSI_CHAFA=chafa
  BRAUSI_XDOTOOL=xdotool"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_start_with_url() {
        assert_eq!(
            parse_args(&strings(&["start", "https://example.com"])).unwrap(),
            CliCommand::Start {
                url: Some("https://example.com".to_string())
            }
        );
    }

    #[test]
    fn parses_click_coords() {
        assert_eq!(
            parse_args(&strings(&["click", "320", "240"])).unwrap(),
            CliCommand::Click { x: 320, y: 240 }
        );
    }

    #[test]
    fn joins_type_text() {
        assert_eq!(
            parse_args(&strings(&["type", "hello", "world"])).unwrap(),
            CliCommand::Type {
                text: "hello world".to_string()
            }
        );
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(parse_args(&strings(&["wat"])).is_err());
    }

    #[test]
    fn validates_framebuffer_bounds() {
        let mut config = Config::from_env().unwrap();
        config.width = 1024;
        config.height = 768;
        assert!(validate_coords(&config, 1023, 767).is_ok());
        assert!(validate_coords(&config, 1024, 767).is_err());
        assert!(validate_coords(&config, 1023, 768).is_err());
    }
}
