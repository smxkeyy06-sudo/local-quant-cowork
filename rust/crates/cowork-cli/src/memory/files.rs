use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const REQUIRED_MEMORY_FILES: [&str; 3] = ["mission.md", "context.md", "tasks.json"];
pub const VALID_TASK_STATUSES: [&str; 4] = ["queued", "active", "done", "blocked"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub goal: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<TaskNote>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskNote {
    pub timestamp: String,
    pub text: String,
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
pub struct TaskNoteAppend {
    pub id: String,
    pub note_timestamp: String,
    pub note_length: usize,
    pub total_notes: usize,
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
    append_task_with_timestamp(memory_dir, goal, &unix_timestamp()?)
}

fn append_task_with_timestamp(memory_dir: &Path, goal: &str, timestamp: &str) -> Result<TaskItem> {
    let goal = goal.trim();
    if goal.is_empty() {
        anyhow::bail!("goal is required");
    }
    let timestamp = timestamp.trim();
    if timestamp.is_empty() {
        anyhow::bail!("timestamp is required");
    }

    let mut tasks = read_tasks(memory_dir)?;

    let id = next_task_id(&tasks.tasks);
    let new_task = TaskItem {
        id,
        goal: goal.to_string(),
        status: "queued".to_string(),
        created_at: Some(timestamp.to_string()),
        updated_at: Some(timestamp.to_string()),
        notes: None,
    };

    tasks.tasks.push(new_task.clone());
    write_tasks(memory_dir, &tasks)?;

    Ok(new_task)
}

pub fn append_task_note(memory_dir: &Path, task_id: &str, note: &str) -> Result<TaskNoteAppend> {
    append_task_note_with_timestamp(memory_dir, task_id, note, &unix_timestamp()?)
}

fn append_task_note_with_timestamp(
    memory_dir: &Path,
    task_id: &str,
    note: &str,
    timestamp: &str,
) -> Result<TaskNoteAppend> {
    let task_id = task_id.trim();
    if task_id.is_empty() {
        anyhow::bail!("task id is required");
    }

    let note = note.trim();
    if note.is_empty() {
        anyhow::bail!("note is required");
    }

    let timestamp = timestamp.trim();
    if timestamp.is_empty() {
        anyhow::bail!("timestamp is required");
    }

    let mut tasks = read_tasks(memory_dir)?;
    let task = tasks
        .tasks
        .iter_mut()
        .find(|task| task.id == task_id)
        .with_context(|| format!("task id not found: {task_id}"))?;

    let notes = task.notes.get_or_insert_with(Vec::new);
    notes.push(TaskNote {
        timestamp: timestamp.to_string(),
        text: note.to_string(),
    });
    let total_notes = notes.len();
    task.updated_at = Some(timestamp.to_string());

    let appended = TaskNoteAppend {
        id: task.id.clone(),
        note_timestamp: timestamp.to_string(),
        note_length: note.len(),
        total_notes,
    };

    write_tasks(memory_dir, &tasks)?;

    Ok(appended)
}

pub fn update_task_status(
    memory_dir: &Path,
    task_id: &str,
    status: &str,
) -> Result<TaskStatusUpdate> {
    update_task_status_with_timestamp(memory_dir, task_id, status, &unix_timestamp()?)
}

fn update_task_status_with_timestamp(
    memory_dir: &Path,
    task_id: &str,
    status: &str,
    timestamp: &str,
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
    let timestamp = timestamp.trim();
    if timestamp.is_empty() {
        anyhow::bail!("timestamp is required");
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
    task.updated_at = Some(timestamp.to_string());
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

fn unix_timestamp() -> Result<String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_secs();
    Ok(format!("unix:{seconds}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        append_context_note, append_task, append_task_note, append_task_note_with_timestamp,
        append_task_with_timestamp, audit_tasks, read_tasks, update_task_status,
        update_task_status_with_timestamp, TaskFile, TaskItem,
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

        let created =
            append_task_with_timestamp(&dir, "new objective", "unix:100").expect("append task");
        assert_eq!(created.id, "task-0001");
        assert_eq!(created.goal, "new objective");
        assert_eq!(created.status, "queued");
        assert_eq!(created.created_at.as_deref(), Some("unix:100"));
        assert_eq!(created.updated_at.as_deref(), Some("unix:100"));

        let loaded = read_tasks(&dir).expect("read after append");
        assert_eq!(loaded.tasks.len(), 2);
        assert_eq!(loaded.tasks[1].id, "task-0001");
        assert_eq!(loaded.tasks[1].created_at.as_deref(), Some("unix:100"));
        assert_eq!(loaded.tasks[1].updated_at.as_deref(), Some("unix:100"));

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_task_adds_current_timestamps() {
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

        let created = append_task(&dir, "timestamped objective").expect("append task");
        assert!(created
            .created_at
            .as_deref()
            .unwrap_or("")
            .starts_with("unix:"));
        assert!(created
            .updated_at
            .as_deref()
            .unwrap_or("")
            .starts_with("unix:"));

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn old_tasks_without_timestamps_still_parse() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        fs::write(
            dir.join("tasks.json"),
            r#"{
  "version": 1,
  "tasks": [
    {
      "id": "task-0001",
      "goal": "legacy",
      "status": "queued"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let loaded = read_tasks(&dir).expect("read legacy task");
        assert_eq!(loaded.tasks[0].id, "task-0001");
        assert_eq!(loaded.tasks[0].created_at, None);
        assert_eq!(loaded.tasks[0].updated_at, None);
        assert_eq!(loaded.tasks[0].notes, None);

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn old_tasks_without_notes_still_parse() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).expect("create dir");

        fs::write(
            dir.join("tasks.json"),
            r#"{
  "version": 1,
  "tasks": [
    {
      "id": "task-0001",
      "goal": "legacy",
      "status": "queued",
      "created_at": "unix:100",
      "updated_at": "unix:100"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let loaded = read_tasks(&dir).expect("read legacy task");
        assert_eq!(loaded.tasks[0].notes, None);

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_task_note_adds_first_note_and_updates_timestamp() {
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
      "status": "queued",
      "created_at": "unix:100",
      "updated_at": "unix:100"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let appended = append_task_note_with_timestamp(&dir, "task-0001", "first note", "unix:200")
            .expect("append note");
        assert_eq!(appended.id, "task-0001");
        assert_eq!(appended.note_timestamp, "unix:200");
        assert_eq!(appended.note_length, "first note".len());
        assert_eq!(appended.total_notes, 1);

        let loaded = read_tasks(&dir).expect("read after note");
        let notes = loaded.tasks[0].notes.as_ref().expect("notes");
        assert_eq!(loaded.tasks[0].created_at.as_deref(), Some("unix:100"));
        assert_eq!(loaded.tasks[0].updated_at.as_deref(), Some("unix:200"));
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].timestamp, "unix:200");
        assert_eq!(notes[0].text, "first note");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_task_note_preserves_existing_notes() {
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
      "status": "queued",
      "created_at": "unix:100",
      "updated_at": "unix:200",
      "notes": [
        {
          "timestamp": "unix:200",
          "text": "first note"
        }
      ]
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        let appended =
            append_task_note_with_timestamp(&dir, "task-0001", "second note", "unix:300")
                .expect("append note");
        assert_eq!(appended.total_notes, 2);

        let loaded = read_tasks(&dir).expect("read after note");
        let notes = loaded.tasks[0].notes.as_ref().expect("notes");
        assert_eq!(loaded.tasks[0].created_at.as_deref(), Some("unix:100"));
        assert_eq!(loaded.tasks[0].updated_at.as_deref(), Some("unix:300"));
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].text, "first note");
        assert_eq!(notes[1].timestamp, "unix:300");
        assert_eq!(notes[1].text, "second note");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn append_task_note_missing_task_id_errors() {
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

        let err = append_task_note(&dir, "missing-task", "note")
            .expect_err("should reject missing task id");
        assert!(err.to_string().contains("task id not found: missing-task"));

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
                    created_at: None,
                    updated_at: None,
                    notes: None,
                },
                TaskItem {
                    id: "task-0001".to_string(),
                    goal: " ".to_string(),
                    status: "bad".to_string(),
                    created_at: None,
                    updated_at: None,
                    notes: None,
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

        let updated = update_task_status_with_timestamp(&dir, "task-0001", "done", "unix:200")
            .expect("update status");
        assert_eq!(updated.id, "task-0001");
        assert_eq!(updated.old_status, "queued");
        assert_eq!(updated.new_status, "done");
        assert_eq!(updated.goal, "first");

        let loaded = read_tasks(&dir).expect("read after update");
        assert_eq!(loaded.tasks[0].status, "done");
        assert_eq!(loaded.tasks[0].created_at, None);
        assert_eq!(loaded.tasks[0].updated_at.as_deref(), Some("unix:200"));
        assert_eq!(loaded.tasks[1].status, "blocked");

        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn update_task_status_preserves_created_at_and_updates_updated_at() {
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
      "status": "queued",
      "created_at": "unix:100",
      "updated_at": "unix:100"
    }
  ]
}
"#,
        )
        .expect("seed tasks.json");

        update_task_status_with_timestamp(&dir, "task-0001", "done", "unix:300")
            .expect("update status");

        let loaded = read_tasks(&dir).expect("read after update");
        assert_eq!(loaded.tasks[0].status, "done");
        assert_eq!(loaded.tasks[0].created_at.as_deref(), Some("unix:100"));
        assert_eq!(loaded.tasks[0].updated_at.as_deref(), Some("unix:300"));

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
