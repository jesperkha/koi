mod modules;
mod types;

use crate::config::Config;

pub use modules::*;
pub use types::*;

pub struct Context {
    pub types: TypeInterner,
    pub modules: ModuleInterner,
    pub config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            types: TypeInterner::new(),
            modules: ModuleInterner::new(),
            config,
        }
    }
}
