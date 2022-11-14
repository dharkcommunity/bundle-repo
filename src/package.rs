use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PackageInfo {
    name: String,
    author: u32,
    versions: Vec<String>,
}

#[allow(unused)]
impl PackageInfo {
    pub const fn new(name: String, author: u32, versions: Vec<String>) -> Self {
        Self {
            name,
            author,
            versions,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn author(&self) -> u32 {
        self.author
    }

    pub const fn versions(&self) -> &Vec<String> {
        &self.versions
    }
}
