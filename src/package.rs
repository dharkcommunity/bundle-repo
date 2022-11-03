use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Package {
    title: String,
    author: u32,
    releases: Vec<String>,
}

impl Package {
    pub const fn new(title: String, author: u32, releases: Vec<String>) -> Self {
        Self {
            title,
            author,
            releases,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub const fn author(&self) -> u32 {
        self.author
    }

    pub const fn releases(&self) -> &Vec<String> {
        &self.releases
    }
}
