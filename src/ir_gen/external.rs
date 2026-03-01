

#[derive(Debug, Clone)]
pub struct ExternalInfo {
    pub name: String,
    pub package_name: String,
    pub is_const: bool,
}

impl ExternalInfo {
    pub fn new(name: String, package_name: String, is_const: bool) -> Self {
        Self {
            name,
            package_name,
            is_const,
        }
    }
}
