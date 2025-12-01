pub struct Config {
    /// Anonymous packages. Parser expects no package declaration and will not raise error.
    pub anon_packages: bool,
    /// If true, the TypeContext object is printed after type checking.
    pub dump_type_context: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            anon_packages: false,
            dump_type_context: false,
        }
    }

    pub fn test() -> Self {
        Self {
            anon_packages: true,
            dump_type_context: false,
        }
    }

    pub fn debug() -> Self {
        Self {
            anon_packages: false,
            dump_type_context: true,
        }
    }
}
