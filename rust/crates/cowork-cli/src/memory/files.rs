use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

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

    use super::{append_task, audit_tasks, read_tasks, TaskFile, TaskItem};

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
}
