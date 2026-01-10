pub struct Config {
    /// If true, the TypeContext object is printed after type checking.
    pub dump_type_context: bool,
    /// If true, all symbols found and used in each module is printed after checking.
    pub print_symbols: bool,
    /// Dont mangle any symbol names, used primarily for testing.
    pub no_mangle_names: bool,
}

impl Config {
    /// Config for normal program execution.
    pub fn default() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: false,
            print_symbols: false,
        }
    }

    /// Config for unit tests.
    pub fn test() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: true,
            print_symbols: false,
        }
    }

    /// Config for running the compiler in debug mode.
    pub fn debug() -> Self {
        Self {
            dump_type_context: true,
            no_mangle_names: false,
            print_symbols: true,
        }
    }
}
