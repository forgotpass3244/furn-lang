use std::ops::RangeInclusive;

use crate::ir_gen::{global::GlobalInfo, ir::IRNode};



pub struct CompiledProgram<'a> {
    ir: Vec<IRNode>,
    globals: Vec<GlobalInfo<'a>>,
    package_name: Option<&'a str>,
}

impl<'a> CompiledProgram<'a> {
    pub fn new() -> Self {
        Self {
            ir: Vec::new(),
            globals: Vec::new(),
            package_name: None,
        }
    }

    pub fn get_package_name(&self) -> Option<&'a str> {
        self.package_name.clone()
    }

    pub fn set_package_name(&mut self, name: Option<&'a str>) {
        self.package_name = name;
    }

    /// Append IRNode
    pub fn app_node(&mut self, node: IRNode) {
        self.ir.push(node);
    }

    pub fn count_ir(&self) -> usize {
        self.ir.len()
    }

    pub fn node_clone_at(&self, pos: usize) -> IRNode {
        self.ir[pos].clone()
    }

    pub fn node_at(&self, pos: usize) -> &IRNode {
        &self.ir[pos]
    }

    pub fn node_mut_at(&mut self, pos: usize) -> &mut IRNode {
        &mut self.ir[pos]
    }

    pub fn ir_iter(&self) -> std::slice::Iter<'_, IRNode> {
        self.ir.iter()
    }

    pub fn globals_iter(&self) -> std::slice::Iter<'_, GlobalInfo<'_>> {
        self.globals.iter()
    }

    pub fn any_global_exists(&self) -> bool {
        !self.globals.is_empty()
    }

    pub fn global_count(&self) -> usize {
        self.globals.len()
    }
    
    pub fn add_global(&mut self, global: GlobalInfo<'a>) {
        self.globals.push(global);
    }

    pub fn shift_nodes(&mut self, range: RangeInclusive<usize>) {
        for i in range.clone().rev() {
            self.ir.remove(i);
        }

        self.realign_addresses(*range.start(), false);
    }

    pub fn insert_node(&mut self, pos: usize, node: IRNode) {
        self.ir.insert(pos, node);
        self.realign_addresses(pos, true);
    }

    fn realign_addresses(&mut self, pos: usize, switch: bool) {
        let mut i: usize = 0;
        for node in &mut *self.ir {
            match node {
                IRNode::CallFromOffset(offset) => {
                    if i < pos {
                        if *offset >= (pos as i16) {
                            *offset -= if switch { 1 } else { -1 };
                        }
                    } else {
                        if *offset < (pos as i16) {
                            *offset += if switch { 1 } else { -1 };
                        }
                    }
                },
                _ => (),
            }

            i += 1;
        }
    }
}

