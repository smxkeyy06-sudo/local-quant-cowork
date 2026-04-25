use anyhow::Result;

use crate::memory::files;
use crate::runtime::Runtime;

pub fn run(runtime: &Runtime) -> Result<()> {
    let tasks = files::read_tasks(&runtime.config.memory_dir)?;
    let audit = files::audit_tasks(&tasks);

    println!("cowork tasks");
    println!("total: {}", audit.total);
    println!("by_status: {}", format_status_counts(&audit));

    for task in tasks.tasks {
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
        println!(
            "- {} [{}] {}{}{}",
            task.id, task.status, task.goal, timestamp, notes
        );
    }

    Ok(())
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

    use super::format_status_counts;

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
}
