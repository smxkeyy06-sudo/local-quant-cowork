use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::task;
use crate::memory::files;
use crate::runtime::Runtime;

#[derive(Debug, Serialize)]
struct TaskNoteResult {
    id: String,
    note_timestamp: String,
    note_length: usize,
    total_notes: usize,
}

pub fn run(runtime: &Runtime, task_id: Option<&str>, note: Option<&str>) -> Result<()> {
    let task_id = task_id
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .context("usage: cowork task-note <task-id> \"<note>\" (task id is required)")?;
    let note = note
        .map(str::trim)
        .filter(|note| !note.is_empty())
        .context("usage: cowork task-note <task-id> \"<note>\" (note is required)")?;

    let appended = files::append_task_note(&runtime.config.memory_dir, task_id, note)?;
    let result = TaskNoteResult {
        id: appended.id,
        note_timestamp: appended.note_timestamp,
        note_length: appended.note_length,
        total_notes: appended.total_notes,
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    task::try_obsidian_sync(runtime);

    Ok(())
}
