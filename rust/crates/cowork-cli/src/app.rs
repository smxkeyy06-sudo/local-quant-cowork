use anyhow::Result;

use crate::commands::{chat, context, doctor, task, task_status, tasks};
use crate::runtime::Runtime;

pub fn run(args: Vec<String>) -> Result<()> {
    let runtime = Runtime::new()?;

    match args.get(1).map(String::as_str) {
        Some("chat") => chat::run(&runtime),
        Some("context") => context::run(&runtime, args.get(2).map(String::as_str)),
        Some("task") => task::run(&runtime, args.get(2).map(String::as_str)),
        Some("task-status") => task_status::run(
            &runtime,
            args.get(2).map(String::as_str),
            args.get(3).map(String::as_str),
        ),
        Some("tasks") => tasks::run(&runtime),
        Some("doctor") => doctor::run(&runtime),
        _ => {
            println!("Usage: cowork <chat|context|task|task-status|tasks|doctor> [args]");
            Ok(())
        }
    }
}
