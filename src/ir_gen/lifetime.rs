
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Lifetime {
    id: usize,
    pub scope_depth: isize,
}

impl Lifetime {
    pub fn new(id: usize, scope_depth: isize) -> Self {
        Self {
            id,
            scope_depth,
        }
    }
}

impl PartialEq for Lifetime {
    fn eq(&self, other: &Self) -> bool {
        self.scope_depth == other.scope_depth
    }
}

impl PartialOrd for Lifetime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.scope_depth.cmp(&other.scope_depth).reverse())
    }
}
