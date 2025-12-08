pub struct Config {
    /// If true, the TypeContext object is printed after type checking.
    pub dump_type_context: bool,
    /// Dont mangle any function names
    pub no_mangle_names: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: false,
        }
    }

    pub fn test() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: true,
        }
    }

    pub fn debug() -> Self {
        Self {
            dump_type_context: true,
            no_mangle_names: false,
        }
    }
}
