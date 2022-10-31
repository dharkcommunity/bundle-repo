#[allow(dead_code)]
#[derive(Debug)]
pub enum Build {
    Development,
    Production,
}

#[cfg(debug_assertions)]
pub const BUILD: Build = Build::Development;

#[cfg(not(debug_assertions))]
pub const BUILD: Build = Build::Production;
