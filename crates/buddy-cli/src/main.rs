use buddy_core::Engine;
use std::{env, error::Error, process::ExitCode};

fn run() -> Result<(), Box<dyn Error>> {
    let engine = Engine::embedded()?;
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("validate-assets") if args.len() == 1 => {
            println!(
                "Beni: {} Posen, {} Animationen, Atlas {} × {} — gültig.",
                engine.atlas.frames.len(),
                engine.animations.len(),
                engine.image_size[0],
                engine.image_size[1]
            );
        }
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
            let pose = engine.pose_at(&args[1], time, 0.7)?;
            println!("{}", serde_json::to_string_pretty(&pose)?);
        }
        Some("--help" | "-h") | None => {
            println!(
                "Beni Buddy CLI\n\n  validate-assets\n  states\n  sample <state> [time_ms]\n\nDesktop: cargo run --manifest-path apps/desktop/Cargo.toml"
            );
        }
        _ => return Err("Unbekannter Befehl. Nutze --help.".into()),
    }
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Beni: {e}");
            ExitCode::FAILURE
        }
    }
}
