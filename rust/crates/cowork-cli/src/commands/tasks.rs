use anyhow::Result;

use crate::memory::files;
use crate::runtime::Runtime;

pub fn run(runtime: &Runtime, args: &[String]) -> Result<()> {
    let status_filter = parse_status_filter(args)?;
    let tasks = files::read_tasks(&runtime.config.memory_dir)?;
    let audit = files::audit_tasks(&tasks);

    println!("cowork tasks");
    if let Some(status) = status_filter {
        let filtered = tasks
            .tasks
            .into_iter()
            .filter(|task| task.status == status)
            .collect::<Vec<_>>();

        println!("status: {status}");
        println!("total: {}", filtered.len());
        for task in filtered {
            println!("{}", format_task_line(&task));
        }
    } else {
        println!("total: {}", audit.total);
        println!("by_status: {}", format_status_counts(&audit));

        for task in tasks.tasks {
            println!("{}", format_task_line(&task));
        }
    }

    Ok(())
}

fn parse_status_filter(args: &[String]) -> Result<Option<&str>> {
    match args {
        [] => Ok(None),
        [flag, status] if flag == "--status" => {
            let status = status.trim();
            if files::VALID_TASK_STATUSES.contains(&status) {
                Ok(Some(status))
            } else {
                anyhow::bail!(
                    "invalid task status filter: {status} (allowed: {})",
                    files::VALID_TASK_STATUSES.join(", ")
                );
            }
        }
        _ => anyhow::bail!("usage: cowork tasks [--status <queued|active|done|blocked>]"),
    }
}

fn format_task_line(task: &files::TaskItem) -> String {
    let timestamp = task
        .updated_at
        .as_deref()
        .map(|updated_at| format!(" updated:{updated_at}"))
        .unwrap_or_default();
    let notes = task
        .notes
        .as_ref()
        .map(|notes| format!(" notes:{}", notes.len()))
        .unwrap_or_default();
    format!(
        "- {} [{}] {}{}{}",
        task.id, task.status, task.goal, timestamp, notes
    )
}

fn format_status_counts(audit: &files::TaskAudit) -> String {
    files::VALID_TASK_STATUSES
        .iter()
        .map(|status| {
            let count = audit.status_counts.get(*status).copied().unwrap_or(0);
            format!("{status}:{count}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use crate::memory::files::{self, TaskAudit};
    use std::collections::BTreeMap;

    use super::{format_status_counts, parse_status_filter};

    #[test]
    fn format_status_counts_includes_all_known_statuses() {
        let mut counts = BTreeMap::new();
        counts.insert("queued".to_string(), 2);
        counts.insert("done".to_string(), 1);

        let audit = TaskAudit {
            total: 3,
            status_counts: counts,
            duplicate_ids: vec![],
            invalid_statuses: vec![],
            empty_goal_ids: vec![],
        };

        let summary = format_status_counts(&audit);
        let expected = files::VALID_TASK_STATUSES
            .iter()
            .map(|s| {
                let c = if *s == "queued" {
                    2
                } else if *s == "done" {
                    1
                } else {
                    0
                };
                format!("{s}:{c}")
            })
            .collect::<Vec<_>>()
            .join(" ");

        assert_eq!(summary, expected);
    }

    #[test]
    fn parse_status_filter_accepts_no_filter() {
        let args = Vec::new();
        assert_eq!(parse_status_filter(&args).expect("parse args"), None);
    }

    #[test]
    fn parse_status_filter_accepts_known_status() {
        let args = vec!["--status".to_string(), "queued".to_string()];
        assert_eq!(
            parse_status_filter(&args).expect("parse args"),
            Some("queued")
        );
    }

    #[test]
    fn parse_status_filter_rejects_invalid_status() {
        let args = vec!["--status".to_string(), "waiting".to_string()];
        let err = parse_status_filter(&args).expect_err("should reject invalid status");
        assert!(err.to_string().contains("invalid task status filter"));
    }

    #[test]
    fn parse_status_filter_rejects_malformed_args() {
        let args = vec!["--status".to_string()];
        let err = parse_status_filter(&args).expect_err("should reject malformed args");
        assert!(err.to_string().contains("usage: cowork tasks"));
    }
}
