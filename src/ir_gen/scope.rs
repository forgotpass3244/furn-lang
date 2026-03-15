use std::collections::VecDeque;

pub trait Named {
    fn get_name(&self) -> &String;
}

#[derive(Clone)]
pub struct Scope<T> {
    vars: VecDeque<T>,
}

impl<T> Scope<T>
where T: Named {
    pub fn new() -> Self {
        Self {
            vars: VecDeque::new(),
        }
    }
    
    pub fn lookup(&self, name: &str) -> Option<&T> {
        for var in &self.vars {
            let var_name = var.get_name();
            if (
                var_name.chars().nth(0).unwrap_or_default()
                == name.chars().nth(0).unwrap_or_default()
            ) && (
                var_name.replace("_", "").to_lowercase()
                == name.replace("_", "").to_lowercase()
            ) {
                return Some(&var)
            }
        }
        
        None
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut T> {
        for var in &mut self.vars {
            if var.get_name() == name {
                return Some(var)
            }
        }
        
        None
    }

    pub fn add(&mut self, var: T) {
        self.vars.push_front(var);
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.vars.iter()
    }
}

