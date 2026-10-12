use buddy_core::{
    Engine,
    notifications::{Notification, Priority},
};
use buddy_ipc::{Endpoint, notify};
use std::{env, error::Error, process::Command};

struct CliError {
    code: i32,
    message: String,
}
impl From<Box<dyn Error>> for CliError {
    fn from(e: Box<dyn Error>) -> Self {
        Self {
            code: 2,
            message: e.to_string(),
        }
    }
}
fn cli_endpoint() -> std::io::Result<Endpoint> {
    match env::var("BENI_IPC_ENDPOINT") {
        Ok(name) => Ok(Endpoint(name)),
        Err(_) => Endpoint::for_user(),
    }
}
fn status_command(args: &[String]) -> Result<i32, CliError> {
    if args.first().map(String::as_str) == Some("run") {
        if args.get(1).map(String::as_str) != Some("--") || args.len() < 3 {
            return Err(CliError {
                code: 2,
                message: "run -- <Programm> [Argumente] erwartet".into(),
            });
        }
        let label = std::path::Path::new(&args[2])
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .chars()
            .take(80)
            .collect::<String>();
        let endpoint = cli_endpoint();
        let mut start = Notification::new(format!("Läuft: {label}"));
        start.state = Some("nachdenken".into());
        start.source = "beni-run".into();
        start.ttl_s = 10;
        match &endpoint {
            Ok(endpoint) => {
                if let Err(e) = notify(endpoint, start) {
                    eprintln!("Beni: {e}");
                }
            }
            Err(e) => eprintln!("Beni: Status-Endpunkt nicht verfügbar: {e}"),
        }
        let result = Command::new(&args[2]).args(&args[3..]).status();
        let code = match result {
            Ok(status) => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    status.code().unwrap_or(128 + status.signal().unwrap_or(1))
                }
                #[cfg(not(unix))]
                {
                    status.code().unwrap_or(1)
                }
            }
            Err(e) => {
                eprintln!("{label}: {e}");
                127
            }
        };
        let mut end = Notification::new(if code == 0 {
            format!("Fertig: {label}")
        } else {
            format!("Fehlgeschlagen: {label} (Exitcode {code})")
        });
        end.source = "beni-run".into();
        end.state = Some(if code == 0 { "daumen" } else { "stopp" }.into());
        if let Ok(endpoint) = &endpoint
            && let Err(e) = notify(endpoint, end)
        {
            eprintln!("Beni: {e}");
        }
        return Ok(code);
    }
    let text = args.get(1).ok_or_else(|| CliError {
        code: 2,
        message: "notify <Text> erwartet".into(),
    })?;
    let mut message = Notification::new(text);
    let mut endpoint = None;
    let mut seen = std::collections::BTreeSet::new();
    let mut options = args[2..].iter();
    while let Some(flag) = options.next() {
        if !seen.insert(flag) {
            return Err(CliError {
                code: 2,
                message: format!("Doppelte Option: {flag}"),
            });
        }
        let value = options.next().ok_or_else(|| CliError {
            code: 2,
            message: format!("Wert fehlt für {flag}"),
        })?;
        match flag.as_str() {
            "--state" => message.state = Some(value.clone()),
            "--source" => message.source = value.clone(),
            "--priority" => {
                message.priority = match value.as_str() {
                    "normal" => Priority::Normal,
                    "high" => Priority::High,
                    _ => {
                        return Err(CliError {
                            code: 2,
                            message: "Priorität: normal oder high".into(),
                        });
                    }
                }
            }
            "--ttl" => {
                message.ttl_s = value.parse().map_err(|_| CliError {
                    code: 2,
                    message: "TTL: ganze Sekunden".into(),
                })?
            }
            "--endpoint" => endpoint = Some(Endpoint(value.clone())),
            _ => {
                return Err(CliError {
                    code: 2,
                    message: format!("Unbekannte Option: {flag}"),
                });
            }
        }
    }
    let endpoint = endpoint
        .map(Ok)
        .unwrap_or_else(cli_endpoint)
        .map_err(|e| CliError {
            code: 3,
            message: e.to_string(),
        })?;
    notify(&endpoint, message).map_err(|e| CliError {
        code: e.exit_code(),
        message: e.to_string(),
    })?;
    println!("Beni: Meldung angenommen.");
    Ok(0)
}
fn legacy_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let engine = Engine::embedded()?;
    match args.first().map(String::as_str) {
        Some("validate-assets") if args.len() == 1 => println!(
            "Beni: {} Posen, {} Animationen, Atlas {} × {} — gültig.",
            engine.atlas.frames.len(),
            engine.animations.len(),
            engine.image_size[0],
            engine.image_size[1]
        ),
        Some("states") if args.len() == 1 => {
            for (name, animation) in &engine.animations {
                println!(
                    "{name:16} {:18} {} ms",
                    animation.label,
                    animation.duration_ms()
                );
            }
        }
        Some("sample") if (2..=3).contains(&args.len()) => {
            let time = args
                .get(2)
                .map(|s| s.parse::<f64>())
                .transpose()?
                .unwrap_or(0.0);
            println!(
                "{}",
                serde_json::to_string_pretty(&engine.pose_at(&args[1], time, 0.7)?)?
            );
        }
        Some("--help" | "-h") | None => println!(
            "Beni Buddy CLI\n\n  validate-assets\n  states\n  sample <state> [time_ms]\n  notify <Text> [--state X] [--source Y] [--priority normal|high] [--ttl Sekunden]\n  run -- <Programm> [Argumente]\n\nnotify: 0 angenommen, 2 ungültig, 3 nicht erreichbar, 4 abgelehnt, 5 Timeout.\nrun: Exitcode des Programms; Beni ist optional.\nDesktop: cargo run --manifest-path apps/desktop/Cargo.toml"
        ),
        _ => return Err("Unbekannter Befehl. Nutze --help.".into()),
    }
    Ok(())
}
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = if matches!(args.first().map(String::as_str), Some("notify" | "run")) {
        status_command(&args)
    } else {
        legacy_command(&args).map(|_| 0).map_err(CliError::from)
    };
    match result {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("Beni: {}", error.message);
            std::process::exit(error.code);
        }
    }
}
