use anyhow::Result;

use crate::commands::{chat, doctor, task};
use crate::runtime::Runtime;

pub fn run(args: Vec<String>) -> Result<()> {
    let runtime = Runtime::new()?;

    match args.get(1).map(String::as_str) {
        Some("chat") => chat::run(&runtime),
        Some("task") => task::run(&runtime, args.get(2).map(String::as_str)),
        Some("doctor") => doctor::run(&runtime),
        _ => {
            println!("Usage: cowork <chat|task|doctor> [args]");
            Ok(())
        }
    }
}
