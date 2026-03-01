use std::collections::VecDeque;

use crate::ir_gen::variable::Variable;

#[derive(Clone)]
pub struct Scope {
    vars: VecDeque<Variable>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            vars: VecDeque::new(),
        }
    }
    
    pub fn lookup(&self, name: &str) -> Option<&Variable> {
        for var in &self.vars {
            if var.name == name {
                return Some(&var)
            }
        }
        
        None
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Variable> {
        for var in &mut self.vars {
            if var.name == name {
                return Some(var)
            }
        }
        
        None
    }

    pub fn add(&mut self, var: Variable) {
        self.vars.push_front(var);
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, Variable> {
        self.vars.iter()
    }
}

