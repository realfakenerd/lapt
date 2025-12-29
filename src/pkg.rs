#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub id: String, // ID único do PackageKit
    pub name: String,
    pub version: String,
    pub arch: String,
    pub data: String, // Repositório ou dados extras
    pub status: String,
    pub summary: String,
    pub description: String,
    pub license: String,
    pub size: u64,
    pub url: String,
}

impl Package {
    pub fn from_packagekit(id: &str, status: &str, summary: &str) -> Self {
        let parts: Vec<&str> = id.split(';').collect();

        let name = parts.get(0).unwrap_or(&"?").to_string();
        let version = parts.get(1).unwrap_or(&"?").to_string();
        let arch = parts.get(2).unwrap_or(&"?").to_string();
        let data = parts.get(3).unwrap_or(&"?").to_string();

        Self {
            id: id.to_string(),
            name,
            version,
            arch,
            data,
            status: status.to_string(),
            summary: summary.to_string(),
            description: String::new(),
            license: String::new(),
            size: 0,
            url: String::new(),
        }
    }

    pub fn update_details(
        &mut self,
        description: &str,
        license: &str,
        size: u64,
        url: &str,
    ) {
        self.description = description.to_string();
        self.license = license.to_string();
        self.size = size;
        self.url = url.to_string();
    }
}