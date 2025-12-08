pub struct Config {
    /// If true, the TypeContext object is printed after type checking.
    pub dump_type_context: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            dump_type_context: false,
        }
    }

    pub fn test() -> Self {
        Self {
            dump_type_context: false,
        }
    }

    pub fn debug() -> Self {
        Self {
            dump_type_context: true,
        }
    }
}
