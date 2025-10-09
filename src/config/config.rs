pub struct Config {
    /// Anonymous packages. Parser expects no package declaration and will not raise error.
    pub anon_packages: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            anon_packages: false,
        }
    }

    pub fn test() -> Self {
        Self {
            anon_packages: true,
        }
    }
}
