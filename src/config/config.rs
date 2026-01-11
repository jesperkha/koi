pub struct Config {
    /// Print TypeContext after type checking.
    pub dump_type_context: bool,
    /// Print symbol tables after type checking.
    pub print_symbol_tables: bool,
    /// Dont mangle any symbol names, used primarily for testing.
    pub no_mangle_names: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: false,
            print_symbol_tables: false,
        }
    }

    pub fn test() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: true,
            print_symbol_tables: false,
        }
    }

    pub fn debug() -> Self {
        Self {
            dump_type_context: true,
            no_mangle_names: false,
            print_symbol_tables: true,
        }
    }
}
