

#[derive(Debug, Clone)]
pub struct ExternalInfo<'a> {
    pub name: &'a str,
    pub package_name: &'a str,
    pub is_const: bool,
}

impl<'a> ExternalInfo<'a> {
    pub fn new(name: &'a str, package_name: &'a str, is_const: bool) -> Self {
        Self {
            name,
            package_name,
            is_const,
        }
    }
}
