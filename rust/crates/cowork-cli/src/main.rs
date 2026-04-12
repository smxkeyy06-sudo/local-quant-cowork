mod app;
mod commands;
mod config;
mod memory;
mod prompts;
mod runtime;
mod tools;

use anyhow::Result;

fn main() -> Result<()> {
    app::run(std::env::args().collect())
}
