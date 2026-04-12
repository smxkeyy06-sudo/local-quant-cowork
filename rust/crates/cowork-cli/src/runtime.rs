use anyhow::Result;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub config: Config,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: Config::load(),
        })
    }
}
