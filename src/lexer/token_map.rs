use std::collections::HashMap;


pub struct TokenMap<T> {
    keywords: HashMap<String, T>,
    map: HashMap<String, T>,
}

impl<T> TokenMap<T> {
    pub fn new() -> Self {
        Self {
            keywords: HashMap::new(),
            map: HashMap::new(),
        }
    }

    pub fn make(&mut self, k: &str, v: T) {
        self.map.insert(k.to_string(), v);
    }

    pub fn make_keyword(&mut self, k: &str, v: T) {
        self.keywords.insert(k.to_string(), v);
    }
    
    pub fn get(&self, k: &String) -> Option<&T> {
        self.map.get(k)
    }

    pub fn get_keyword(&self, k: &String) -> Option<&T> {
        self.keywords.get(k)
    }

    pub fn any_key<F>(&self, f: F) -> bool
    where F: FnMut(&String) -> bool {
        self.map.keys().any(f)
    }
}

