//! Service installation helpers: macOS (LaunchAgent), Linux systemd, Linux runit.

#[cfg(target_os = "linux")]
use std::path::Path;

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum InitSystem {
    #[default]
    Auto,
    Systemd,
    #[clap(name = "systemd-user")]
    SystemdUser,
    Runit,
}

#[cfg(target_os = "linux")]
fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

#[cfg(target_os = "linux")]
fn check_config_presence(system_wide: bool) {
    let dir = if system_wide {
        "/etc/keyd/".to_string()
    } else {
        crate::config::get_config_dir()
    };
    let n = std::fs::read_dir(&dir)
        .ok().into_iter().flatten().flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "conf"))
        .count();
    if n == 0 {
        println!("WARNING: No .conf files found in {dir}. The daemon will have no effect until a config is added.");
    }
}

// ── macOS ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
const PLIST_LABEL: &str = "com.argenkiwi.keydo";

#[cfg(target_os = "macos")]
fn plist_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(std::path::PathBuf::from(home)
        .join("Library/LaunchAgents")
        .join(format!("{PLIST_LABEL}.plist")))
}

#[cfg(target_os = "macos")]
pub fn install(_init: InitSystem) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("failed to resolve binary path: {e}"))?;
    let exe_str = exe.to_string_lossy();
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let log_path = format!("{home}/Library/Logs/keydo.log");

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{PLIST_LABEL}</string>

    <key>ProgramArguments</key>
    <array>
        <string>{exe_str}</string>
        <string>daemon</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>{log_path}</string>

    <key>StandardErrorPath</key>
    <string>{log_path}</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
</dict>
</plist>
"#
    );

    let path = plist_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create LaunchAgents directory: {e}"))?;
    }
    std::fs::write(&path, plist)
        .map_err(|e| format!("failed to write plist: {e}"))?;

    let status = std::process::Command::new("launchctl")
        .arg("load")
        .arg(&path)
        .status()
        .map_err(|e| format!("failed to run launchctl: {e}"))?;
    if !status.success() {
        return Err("launchctl load failed".to_string());
    }

    println!("keydo installed and started.");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall(_init: InitSystem) -> Result<(), String> {
    let path = plist_path()?;
    // Ignore errors — may not be loaded yet.
    let _ = std::process::Command::new("launchctl")
        .arg("unload")
        .arg(&path)
        .status();
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("failed to remove plist: {e}"))?;
    }
    println!("keydo uninstalled.");
    Ok(())
}

// ── Linux ──────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn detect_init() -> Result<InitSystem, String> {
    if Path::new("/run/systemd/system").exists() {
        return Ok(InitSystem::Systemd);
    }
    if Path::new("/run/runit").exists() || Path::new("/etc/sv").exists() {
        return Ok(InitSystem::Runit);
    }
    Err("could not auto-detect init system; use --init systemd or --init runit".to_string())
}

#[cfg(target_os = "linux")]
pub fn install(init: InitSystem) -> Result<(), String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("failed to resolve binary path: {e}"))?;
    let resolved = match init {
        InitSystem::Auto => detect_init()?,
        other => other,
    };
    match resolved {
        InitSystem::Systemd      => { check_config_presence(true); install_systemd(&exe) },
        InitSystem::SystemdUser  => { check_config_presence(false); install_systemd_user(&exe) },
        InitSystem::Runit        => { check_config_presence(true); install_runit(&exe) },
        InitSystem::Auto         => unreachable!(),
    }
}

#[cfg(target_os = "linux")]
fn install_systemd(exe: &Path) -> Result<(), String> {
    if !is_root() {
        return Err("System-wide installation requires root privileges (try running with sudo or use --init systemd-user).".to_string());
    }
    let unit = format!(
        "[Unit]\nDescription=keydo keyboard remapping daemon\nAfter=local-fs.target\n\n\
         [Service]\nExecStart={} daemon\nRestart=on-failure\nRestartSec=5\n\n\
         [Install]\nWantedBy=multi-user.target\n",
        exe.display()
    );
    std::fs::write("/etc/systemd/system/keydo.service", unit)
        .map_err(|e| format!("failed to write unit file: {e}"))?;
    run_cmd("systemctl", &["daemon-reload"])?;
    run_cmd("systemctl", &["enable", "--now", "keydo"])?;
    println!("keydo installed and started (systemd).");
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_systemd_user(exe: &Path) -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let user_unit_dir = Path::new(&home).join(".config/systemd/user");
    std::fs::create_dir_all(&user_unit_dir)
        .map_err(|e| format!("failed to create user unit directory: {e}"))?;

    let unit_path = user_unit_dir.join("keydo.service");
    let unit = format!(
        "[Unit]\nDescription=keydo keyboard remapping daemon (user)\n\n\
         [Service]\nExecStart={} daemon\nRestart=on-failure\nRestartSec=5\n\n\
         [Install]\nWantedBy=default.target\n",
        exe.display()
    );
    std::fs::write(&unit_path, unit)
        .map_err(|e| format!("failed to write user unit file: {e}"))?;

    run_cmd("systemctl", &["--user", "daemon-reload"])?;
    run_cmd("systemctl", &["--user", "enable", "--now", "keydo"])?;

    println!("keydo installed and started (systemd user service).");
    println!("Note: Ensure your user has access to /dev/input/* and /dev/uinput (e.g. by being in the 'input' group).");
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_runit(exe: &Path) -> Result<(), String> {
    if !is_root() {
        return Err("Runit installation requires root privileges (try running with sudo).".to_string());
    }
    use std::os::unix::fs::PermissionsExt;

    let sv_dir = Path::new("/etc/sv/keydo");
    std::fs::create_dir_all(sv_dir)
        .map_err(|e| format!("failed to create /etc/sv/keydo: {e}"))?;

    let run_script = format!("#!/bin/sh\nexec {} daemon 2>&1\n", exe.display());
    let run_path = sv_dir.join("run");
    std::fs::write(&run_path, run_script)
        .map_err(|e| format!("failed to write run script: {e}"))?;
    std::fs::set_permissions(&run_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("failed to chmod run script: {e}"))?;

    let link = Path::new("/var/service/keydo");
    if !link.exists() {
        std::os::unix::fs::symlink(sv_dir, link)
            .map_err(|e| format!("failed to create /var/service/keydo (try running as root): {e}"))?;
    }

    println!("keydo installed and started (runit).");
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn uninstall(init: InitSystem) -> Result<(), String> {
    let resolved = match init {
        InitSystem::Auto => detect_init()?,
        other => other,
    };
    match resolved {
        InitSystem::Systemd     => uninstall_systemd(),
        InitSystem::SystemdUser => uninstall_systemd_user(),
        InitSystem::Runit       => uninstall_runit(),
        InitSystem::Auto        => unreachable!(),
    }
}

#[cfg(target_os = "linux")]
fn uninstall_systemd() -> Result<(), String> {
    if !is_root() {
        return Err("System-wide uninstallation requires root privileges (try running with sudo).".to_string());
    }
    // Ignore failure — service may already be stopped or disabled.
    let _ = run_cmd("systemctl", &["disable", "--now", "keydo"]);
    let service = Path::new("/etc/systemd/system/keydo.service");
    if service.exists() {
        std::fs::remove_file(service)
            .map_err(|e| format!("failed to remove unit file: {e}"))?;
    }
    run_cmd("systemctl", &["daemon-reload"])?;
    println!("keydo uninstalled (systemd).");
    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_systemd_user() -> Result<(), String> {
    let _ = run_cmd("systemctl", &["--user", "disable", "--now", "keydo"]);
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let service = Path::new(&home).join(".config/systemd/user/keydo.service");
    if service.exists() {
        std::fs::remove_file(service)
            .map_err(|e| format!("failed to remove user unit file: {e}"))?;
    }
    run_cmd("systemctl", &["--user", "daemon-reload"])?;
    println!("keydo uninstalled (systemd user service).");
    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_runit() -> Result<(), String> {
    if !is_root() {
        return Err("Runit uninstallation requires root privileges (try running with sudo).".to_string());
    }
    let link = Path::new("/var/service/keydo");
    if link.exists() {
        std::fs::remove_file(link)
            .map_err(|e| format!("failed to remove /var/service/keydo: {e}"))?;
    }
    let sv_dir = Path::new("/etc/sv/keydo");
    if sv_dir.exists() {
        std::fs::remove_dir_all(sv_dir)
            .map_err(|e| format!("failed to remove /etc/sv/keydo: {e}"))?;
    }
    println!("keydo uninstalled (runit).");
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_cmd(program: &str, args: &[&str]) -> Result<(), String> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run `{program}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{program} {}` failed with {status}", args.join(" ")))
    }
}

// ── Unsupported platforms ──────────────────────────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn install(_init: InitSystem) -> Result<(), String> {
    Err("install is not supported on this platform".to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn uninstall(_init: InitSystem) -> Result<(), String> {
    Err("uninstall is not supported on this platform".to_string())
}
