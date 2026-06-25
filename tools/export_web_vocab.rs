//! Traceability: AXIOM_CLOSED_ACTIONS, AXIOM_BRAID_CANONICAL.
use std::{env, fs, io, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = braid_vocab_web::registry_v0();
    let verbs = registry
        .terms()
        .map(|term| term.id.as_str())
        .collect::<Vec<_>>();
    let json = serde_json::to_string_pretty(&verbs)?;
    let output = format!("{json}\n");

    match env::args_os().nth(1) {
        Some(path) => write_if_changed(PathBuf::from(path), output.as_bytes())?,
        None => print!("{output}"),
    }

    Ok(())
}

fn write_if_changed(path: PathBuf, bytes: &[u8]) -> io::Result<()> {
    if fs::read(&path).is_ok_and(|current| current == bytes) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}
