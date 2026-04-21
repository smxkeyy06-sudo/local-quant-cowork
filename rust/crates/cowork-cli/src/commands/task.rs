use std::process::Command;

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
    try_obsidian_sync(runtime);

    Ok(())
}

fn try_obsidian_sync(runtime: &Runtime) {
    let sync_script = runtime.config.repo_file("scripts/sync_obsidian.py");
    if !sync_script.is_file() {
        return;
    }

    let output = Command::new("python3")
        .arg(&sync_script)
        .current_dir(&runtime.config.repo_root)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            println!("obsidian_sync: ok");
        }
        Ok(out) => {
            let code = out
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "terminated".to_string());
            eprintln!("obsidian_sync: warning (exit {code})");
        }
        Err(err) => {
            eprintln!("obsidian_sync: warning ({err})");
        }
    }
}
