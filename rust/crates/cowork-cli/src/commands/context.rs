use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::task;
use crate::memory::files;
use crate::runtime::Runtime;

#[derive(Debug, Serialize)]
struct ContextAppendResult {
    context_file: String,
    timestamp: String,
    note_length: usize,
}

pub fn run(runtime: &Runtime, note: Option<&str>) -> Result<()> {
    let note = note
        .map(str::trim)
        .filter(|note| !note.is_empty())
        .context("usage: cowork context \"<note>\" (note is required)")?;
    let timestamp = unix_timestamp()?;

    let appended = files::append_context_note(&runtime.config.memory_dir, note, &timestamp)?;
    let result = ContextAppendResult {
        context_file: appended.path.display().to_string(),
        timestamp: appended.timestamp,
        note_length: appended.note_length,
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    task::try_obsidian_sync(runtime);

    Ok(())
}

fn unix_timestamp() -> Result<String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_secs();
    Ok(format!("unix:{seconds}"))
}
