use std::path::PathBuf;

use anyhow::Result;

use crate::memory::files;
use crate::runtime::Runtime;
use crate::tools::registry;

const REQUIRED_REPO_FILES: [&str; 7] = [
    "README.md",
    "ROADMAP.md",
    "ARCHITECTURE.md",
    "DECISIONS.md",
    "SETUP.md",
    "OPERATIONS.md",
    "rust/Cargo.toml",
];

#[derive(Debug)]
struct Check {
    name: String,
    path: PathBuf,
    ok: bool,
}

pub fn run(runtime: &Runtime) -> Result<()> {
    println!("cowork doctor");

    let mut checks = Vec::new();

    for rel in REQUIRED_REPO_FILES {
        let path = runtime.config.repo_file(rel);
        checks.push(Check {
            name: format!("repo:{rel}"),
            ok: path.exists(),
            path,
        });
    }

    for name in files::REQUIRED_MEMORY_FILES {
        let path = runtime.config.memory_file(name);
        checks.push(Check {
            name: format!("memory:{name}"),
            ok: path.exists(),
            path,
        });
    }

    let missing_count = checks.iter().filter(|check| !check.ok).count();

    println!("repo_root: {}", runtime.config.repo_root.display());
    println!("memory_dir: {}", runtime.config.memory_dir.display());
    println!("checks_total: {}", checks.len());
    println!("checks_missing: {missing_count}");

    for check in &checks {
        let status = if check.ok { "ok" } else { "missing" };
        println!("- {} [{}] {}", check.name, status, check.path.display());
    }

    if missing_count > 0 {
        anyhow::bail!("doctor found missing required files");
    }

    let tasks = files::parse_tasks(&runtime.config.memory_dir)?;
    if tasks.version == 0 {
        anyhow::bail!("invalid tasks file version: 0");
    }
    let audit = files::audit_tasks(&tasks);

    println!("tasks_version: {}", tasks.version);
    println!("tasks_count: {}", audit.total);
    println!("status_counts:");
    for status in files::VALID_TASK_STATUSES {
        let count = audit.status_counts.get(status).copied().unwrap_or(0);
        println!("- {status}: {count}");
    }

    if !audit.duplicate_ids.is_empty() {
        println!("duplicate_task_ids: {}", audit.duplicate_ids.join(", "));
    } else {
        println!("duplicate_task_ids: none");
    }

    if !audit.invalid_statuses.is_empty() {
        println!(
            "invalid_task_statuses: {}",
            audit.invalid_statuses.join(", ")
        );
    } else {
        println!("invalid_task_statuses: none");
    }

    if !audit.empty_goal_ids.is_empty() {
        println!("empty_task_goals: {}", audit.empty_goal_ids.join(", "));
    } else {
        println!("empty_task_goals: none");
    }

    let env_model = std::env::var("COWORK_MODEL").unwrap_or_else(|_| "(unset)".to_string());
    let env_data_dir = std::env::var("COWORK_DATA_DIR").unwrap_or_else(|_| "(unset)".to_string());
    println!("env.COWORK_MODEL: {env_model}");
    println!("env.COWORK_DATA_DIR: {env_data_dir}");

    let tools = registry::default_tools();
    let _tool_descriptions_are_configured = tools.iter().all(|tool| !tool.description.is_empty());

    println!("tool_names:");
    for tool in tools {
        println!("- {}", tool.name);
    }

    if !audit.is_valid() {
        anyhow::bail!("doctor found invalid task records");
    }

    Ok(())
}
