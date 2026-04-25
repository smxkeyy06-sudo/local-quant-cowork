use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const REQUIRED_MEMORY_FILES: [&str; 3] = ["mission.md", "context.md", "tasks.json"];
pub const VALID_TASK_STATUSES: [&str; 4] = ["queued", "active", "done", "blocked"];

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

#[derive(Debug, Clone)]
pub struct ContextAppend {
    pub path: PathBuf,
    pub timestamp: String,
    pub note_length: usize,
}

#[derive(Debug, Clone)]
pub struct TaskStatusUpdate {
    pub id: String,
    pub old_status: String,
    pub new_status: String,
    pub goal: String,
}

#[derive(Debug, Clone)]
pub struct TaskAudit {
    pub total: usize,
    pub status_counts: BTreeMap<String, usize>,
    pub duplicate_ids: Vec<String>,
    pub invalid_statuses: Vec<String>,
    pub empty_goal_ids: Vec<String>,
}

impl TaskAudit {
    pub fn is_valid(&self) -> bool {
        self.duplicate_ids.is_empty()
            && self.invalid_statuses.is_empty()
            && self.empty_goal_ids.is_empty()
    }
}

pub fn parse_tasks(memory_dir: &Path) -> Result<TaskFile> {
    let file = memory_dir.join("tasks.json");
    let raw = fs::read_to_string(&file)
        .with_context(|| format!("failed to read tasks file: {}", file.display()))?;
    let parsed: TaskFile = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse tasks file: {}", file.display()))?;
    Ok(parsed)
}

pub fn read_tasks(memory_dir: &Path) -> Result<TaskFile> {
    let parsed = parse_tasks(memory_dir)?;
    validate_task_file(&parsed)?;
    Ok(parsed)
}

pub fn write_tasks(memory_dir: &Path, tasks: &TaskFile) -> Result<()> {
    validate_task_file(tasks)?;

    let file = memory_dir.join("tasks.json");
    let body = serde_json::to_string_pretty(tasks).context("failed to encode tasks as json")?;
    fs::write(&file, format!("{body}\n"))
        .with_context(|| format!("failed to write tasks file: {}", file.display()))?;
    Ok(())
}

pub fn append_context_note(
    memory_dir: &Path,
    note: &str,
    timestamp: &str,
) -> Result<ContextAppend> {
    let note = note.trim();
    if note.is_empty() {
        anyhow::bail!("note is required");
    }
    let timestamp = timestamp.trim();
    if timestamp.is_empty() {
        anyhow::bail!("timestamp is required");
    }

    let file = memory_dir.join("context.md");
    let block = format!("\n\n## Context Note - {timestamp}\n\n{note}\n");
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .with_context(|| format!("failed to open context file: {}", file.display()))?
        .write_all(block.as_bytes())
        .with_context(|| format!("failed to append context file: {}", file.display()))?;

    Ok(ContextAppend {
        path: file,
        timestamp: timestamp.to_string(),
        note_length: note.len(),
    })
}

pub fn append_task(memory_dir: &Path, goal: &str) -> Result<TaskItem> {
    let goal = goal.trim();
    if goal.is_empty() {
        anyhow::bail!("goal is required");
    }

    let mut tasks = read_tasks(memory_dir)?;

    let id = next_task_id(&tasks.tasks);
    let new_task = TaskItem {
        id,
        goal: goal.to_string(),
        status: "queued".to_string(),
    };

    tasks.tasks.push(new_task.clone());
    write_tasks(memory_dir, &tasks)?;

    Ok(new_task)
}

pub fn update_task_status(
    memory_dir: &Path,
    task_id: &str,
    status: &str,
) -> Result<TaskStatusUpdate> {
    let task_id = task_id.trim();
    if task_id.is_empty() {
        anyhow::bail!("task id is required");
    }

    let status = status.trim();
    if !VALID_TASK_STATUSES.contains(&status) {
        anyhow::bail!(
            "invalid task status: {status} (allowed: {})",
            VALID_TASK_STATUSES.join(", ")
        );
    }

    let mut tasks = read_tasks(memory_dir)?;
    let task = tasks
        .tasks
        .iter_mut()
        .find(|task| task.id == task_id)
        .with_context(|| format!("task id not found: {task_id}"))?;

    let updated = TaskStatusUpdate {
        id: task.id.clone(),
        old_status: task.status.clone(),
        new_status: status.to_string(),
        goal: task.goal.clone(),
    };

    task.status = status.to_string();
    write_tasks(memory_dir, &tasks)?;

    Ok(updated)
}

pub fn audit_tasks(tasks: &TaskFile) -> TaskAudit {
    let mut status_counts = BTreeMap::new();
    let mut seen = HashSet::new();
    let mut duplicate_ids = Vec::new();
    let mut invalid_statuses = Vec::new();
    let mut empty_goal_ids = Vec::new();

    for task in &tasks.tasks {
        *status_counts.entry(task.status.clone()).or_insert(0) += 1;

        if !seen.insert(task.id.clone()) {
            duplicate_ids.push(task.id.clone());
        }

        if !VALID_TASK_STATUSES.contains(&task.status.as_str()) {
            invalid_statuses.push(format!("{}:{}", task.id, task.status));
        }

        if task.goal.trim().is_empty() {
            empty_goal_ids.push(task.id.clone());
        }
    }

    duplicate_ids.sort();
    duplicate_ids.dedup();
    invalid_statuses.sort();
    invalid_statuses.dedup();
    empty_goal_ids.sort();
    empty_goal_ids.dedup();

    TaskAudit {
        total: tasks.tasks.len(),
        status_counts,
        duplicate_ids,
        invalid_statuses,
        empty_goal_ids,
    }
}

fn validate_task_file(tasks: &TaskFile) -> Result<()> {
    if tasks.version == 0 {
        anyhow::bail!("invalid tasks file version: 0");
    }

    for task in &tasks.tasks {
        if task.id.trim().is_empty() {
            anyhow::bail!("task id cannot be empty");
        }
        if task.goal.trim().is_empty() {
            anyhow::bail!("task goal cannot be empty");
        }
        if task.status.trim().is_empty() {
            anyhow::bail!("task status cannot be empty");
        }
        if !VALID_TASK_STATUSES.contains(&task.status.as_str()) {
            anyhow::bail!("invalid task status: {}", task.status);
        }
    }

    Ok(())
}

fn next_task_id(tasks: &[TaskItem]) -> String {
    let mut max_n = 0u32;

    for task in tasks {
        if let Some(suffix) = task.id.strip_prefix("task-") {
            if let Ok(n) = suffix.parse::<u32>() {
                if n > max_n {
                    max_n = n;
                }
            }
        }
    }

    let mut candidate = max_n + 1;
    loop {
        let id = format!("task-{candidate:04}");
        if tasks.iter().all(|t| t.id != id) {
            return id;
        }
        candidate += 1;
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        append_context_note, append_task, audit_tasks, read_tasks, update_task_status, TaskFile,
        TaskItem,
    };

    fn temp_dir() -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("cowork-cli-tests-{uniq}"))
    }

    #[test]
    fn append_task_rejects_empty_goal() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        fs::write(
            dir.join("tasks.json"),
            r#"{
  "version": 1,
  "tasks": []
}
"#,
        )
        .expect("seed tasks.json");

        let err = append_task(&dir, "   ").expect_err("should reject empty goal");
        assert!(err.to_string().contains("goal is required"));

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_context_note_preserves_existing_content() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(dir.join("context.md"), "# Context\n\nExisting").expect("seed context.md");

        let appended =
            append_context_note(&dir, "durable context", "unix:123").expect("append context");
        assert_eq!(appended.path, dir.join("context.md"));
        assert_eq!(appended.timestamp, "unix:123");
        assert_eq!(appended.note_length, "durable context".len());

        let body = fs::read_to_string(dir.join("context.md")).expect("read context.md");

        assert!(body.starts_with("# Context\n\nExisting"));
        assert!(body.contains("## Context Note - unix:123"));
        assert!(body.contains("durable context"));

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_context_note_rejects_empty_note() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        let err = append_context_note(&dir, "   ", "unix:123")
            .expect_err("should reject empty context note");
        assert!(err.to_string().contains("note is required"));

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_task_persists_and_increments_count() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        fs::write(
            dir.join("tasks.json"),
            r#"{
  "version": 1,
  "tasks": [
    {
      "id": "seed-001",
      "goal": "existing",
      "status": "queued"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let created = append_task(&dir, "new objective").expect("append task");
        assert_eq!(created.id, "task-0001");
        assert_eq!(created.goal, "new objective");
        assert_eq!(created.status, "queued");

        let loaded = read_tasks(&dir).expect("read after append");
        assert_eq!(loaded.tasks.len(), 2);
        assert_eq!(loaded.tasks[1].id, "task-0001");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn audit_tasks_reports_duplicates_and_invalids() {
        let tasks = TaskFile {
            version: 1,
            tasks: vec![
                TaskItem {
                    id: "task-0001".to_string(),
                    goal: "a".to_string(),
                    status: "queued".to_string(),
                },
                TaskItem {
                    id: "task-0001".to_string(),
                    goal: " ".to_string(),
                    status: "bad".to_string(),
                },
            ],
        };

        let audit = audit_tasks(&tasks);
        assert_eq!(audit.total, 2);
        assert_eq!(audit.duplicate_ids, vec!["task-0001"]);
        assert_eq!(audit.invalid_statuses, vec!["task-0001:bad"]);
        assert_eq!(audit.empty_goal_ids, vec!["task-0001"]);
    }

    #[test]
    fn update_task_status_persists_matching_task_only() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        fs::write(
            dir.join("tasks.json"),
            r#"{
  "version": 1,
  "tasks": [
    {
      "id": "task-0001",
      "goal": "first",
      "status": "queued"
    },
    {
      "id": "task-0002",
      "goal": "second",
      "status": "blocked"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let updated = update_task_status(&dir, "task-0001", "done").expect("update status");
        assert_eq!(updated.id, "task-0001");
        assert_eq!(updated.old_status, "queued");
        assert_eq!(updated.new_status, "done");
        assert_eq!(updated.goal, "first");

        let loaded = read_tasks(&dir).expect("read after update");
        assert_eq!(loaded.tasks[0].status, "done");
        assert_eq!(loaded.tasks[1].status, "blocked");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn update_task_status_rejects_invalid_status() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        fs::write(
            dir.join("tasks.json"),
            r#"{
  "version": 1,
  "tasks": [
    {
      "id": "task-0001",
      "goal": "first",
      "status": "queued"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let err = update_task_status(&dir, "task-0001", "waiting")
            .expect_err("should reject invalid status");
        assert!(err.to_string().contains("invalid task status"));

        let loaded = read_tasks(&dir).expect("read after failed update");
        assert_eq!(loaded.tasks[0].status, "queued");

        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
