use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::task;
use crate::memory::files;
use crate::runtime::Runtime;

#[derive(Debug, Serialize)]
struct TaskStatusResult {
    id: String,
    old_status: String,
    new_status: String,
    goal: String,
}

pub fn run(runtime: &Runtime, task_id: Option<&str>, status: Option<&str>) -> Result<()> {
    let task_id = task_id
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .context("usage: cowork task-status <task-id> <status> (task id is required)")?;
    let status = status
        .map(str::trim)
        .filter(|status| !status.is_empty())
        .context("usage: cowork task-status <task-id> <status> (status is required)")?;

    let updated = files::update_task_status(&runtime.config.memory_dir, task_id, status)?;
    let result = TaskStatusResult {
        id: updated.id,
        old_status: updated.old_status,
        new_status: updated.new_status,
        goal: updated.goal,
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    task::try_obsidian_sync(runtime);

    Ok(())
}
