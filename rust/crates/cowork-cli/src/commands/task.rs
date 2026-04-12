use anyhow::{Context, Result};
use serde::Serialize;

use crate::memory::files;
use crate::runtime::Runtime;

#[derive(Debug, Serialize)]
struct TaskAppendResult {
    id: String,
    goal: String,
    status: String,
    total_task_count: usize,
}

pub fn run(runtime: &Runtime, goal: Option<&str>) -> Result<()> {
    let goal = goal
        .map(str::trim)
        .filter(|goal| !goal.is_empty())
        .context("usage: cowork task \"<goal>\" (goal is required)")?;

    let created = files::append_task(&runtime.config.memory_dir, goal)?;
    let updated = files::read_tasks(&runtime.config.memory_dir)?;

    let result = TaskAppendResult {
        id: created.id,
        goal: created.goal,
        status: created.status,
        total_task_count: updated.tasks.len(),
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
