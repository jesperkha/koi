mod modules;
mod symbols;
mod types;

use crate::config::Config;

pub use modules::*;
pub use symbols::*;
pub use types::*;

pub struct Context {
    pub types: TypeInterner,
    pub modules: ModuleInterner,
    pub symbols: SymbolInterner,
    pub config: Config,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            symbols: SymbolInterner::new(),
            types: TypeInterner::new(),
            modules: ModuleInterner::new(),
            config,
        }
    }
}
