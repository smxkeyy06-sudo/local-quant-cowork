use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub goal: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFile {
    pub version: u32,
    pub tasks: Vec<TaskItem>,
}

pub fn assert_required_files(memory_dir: &Path) -> Result<()> {
    for rel in ["mission.md", "context.md", "tasks.json"] {
        let p = memory_dir.join(rel);
        if !p.exists() {
            anyhow::bail!("missing required memory file: {}", p.display());
        }
    }
    Ok(())
}

pub fn read_tasks(memory_dir: &Path) -> Result<TaskFile> {
    let file = memory_dir.join("tasks.json");
    let raw = fs::read_to_string(&file)
        .with_context(|| format!("failed to read tasks file: {}", file.display()))?;
    let parsed: TaskFile = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse tasks file: {}", file.display()))?;
    Ok(parsed)
}
