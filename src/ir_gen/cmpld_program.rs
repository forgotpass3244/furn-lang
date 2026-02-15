use std::ops::RangeInclusive;

use crate::ir_gen::{ctimeval::CTimeVal, external::ExternalInfo, global::GlobalInfo, ir::IRNode};


#[derive(Clone)]
pub struct CompiledProgram<'a> {
    ir: Vec<IRNode<'a>>,
    globals: Vec<GlobalInfo<'a>>,
    externals: Vec<ExternalInfo<'a>>,
    package_name: Option<&'a str>,
    static_strings: Vec<(&'a str, usize)>,
    static_strings_len: usize,
}

impl<'a> CompiledProgram<'a> {
    pub fn new() -> Self {
        Self {
            ir: Vec::new(),
            globals: Vec::new(),
            externals: Vec::new(),
            package_name: None,
            static_strings: Vec::new(),
            static_strings_len: 0,
        }
    }

    pub fn get_package_name(&self) -> Option<&'a str> {
        self.package_name.clone()
    }

    pub fn set_package_name(&mut self, name: Option<&'a str>) {
        self.package_name = name;
    }

    /// Append IRNode
    pub fn app_node(&mut self, node: IRNode<'a>) {
        self.ir.push(node);
    }

    pub fn count_ir(&self) -> usize {
        self.ir.len()
    }

    pub fn ir_pos(&self) -> usize {
        if self.ir.len() > 0 {
            self.ir.len() - 1
        } else {
            0
        }
    }

    pub fn node_clone_at(&self, pos: usize) -> IRNode<'a> {
        self.ir[pos].clone()
    }

    pub fn node_at(&self, pos: usize) -> &IRNode<'a> {
        &self.ir[pos]
    }

    pub fn node_mut_at(&mut self, pos: usize) -> &mut IRNode<'a> {
        &mut self.ir[pos]
    }

    pub fn ir_iter(&self) -> std::slice::Iter<'_, IRNode<'_>> {
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

    pub fn first_global(&self) -> Option<&GlobalInfo<'a>> {
        self.globals.first()
    }
    
    pub fn add_global(&mut self, global: GlobalInfo<'a>) {
        self.globals.push(global);
    }

    pub fn add_external(&mut self, external: ExternalInfo<'a>) {
        self.externals.push(external);
    }

    pub fn externals_iter(&self) -> std::slice::Iter<'_, ExternalInfo<'_>> {
        self.externals.iter()
    }

    #[must_use]
    pub fn add_static_string(&mut self, string: &'a str) -> usize {
        for (existing_string, pos) in &self.static_strings {
            if string == *existing_string {
                return *pos
            }
        }
        
        let string_pos = self.static_strings_len;
        self.static_strings.push((string, string_pos));
        self.static_strings_len += string.len();
        string_pos
    }

    pub fn static_strings_iter(&self) -> std::slice::Iter<'_, (&str, usize)> {
        self.static_strings.iter()
    }

    pub fn shift_nodes(&mut self, range: RangeInclusive<usize>) {
        for i in range.clone().rev() {
            self.ir.remove(i);
        }

        self.realign_addresses(*range.start(), range.count(), false);
    }

    pub fn insert_node(&mut self, pos: usize, node: IRNode<'a>) {
        self.ir.insert(pos, node);
        self.realign_addresses(pos, 1, true);
    }

    pub fn realign_stack_offsets(&mut self, pos: usize, stack_offset: usize, alignment: usize) {
        let mut i: usize = 0;
        for node in &mut *self.ir {
            if i > pos {
                match node {

                    IRNode::Pop64ToStack(offset) => {
                        if *offset > stack_offset {
                            *offset += alignment;
                        }
                    },

                    IRNode::Load64ToStack(_, offset) => {
                        if *offset > stack_offset {
                            *offset += alignment;
                        }
                    },

                    IRNode::StackReadPush64(offset) => {
                        if *offset > stack_offset {
                            *offset += alignment;
                        }
                    },

                    IRNode::GlobalReadLoad64ToStack(_, offset) => {
                        if *offset > stack_offset {
                            *offset += alignment;
                        }
                    },

                    IRNode::StackReadLoad64ToStack(src_offset, dst_offset) => {
                        if *src_offset > stack_offset {
                            *src_offset += alignment;
                        }
                        if *dst_offset > stack_offset {
                            *dst_offset += alignment;
                        }
                    },

                    _ => (),
                }
            }

            i += 1;
        }
    }

    fn realign_addresses(&mut self, pos: usize, count: usize, switch: bool) {
        for global in self.globals.iter_mut() {
            match &mut global.init {
                CTimeVal::Function { address, .. } => {
                    if *address > pos {
                        if switch {
                            *address += count;
                        } else {
                            *address -= count;
                        }
                    }
                }

                _ => (),
            }
        }

        let mut i: usize = 0;
        for node in &mut *self.ir {
            match node {

                IRNode::CallFromOffset(offset) => {
                    if i < pos {
                        if *offset >= (pos as i16) {
                            *offset -= if switch { count as i16 } else { -(count as i16) };
                        }
                    } else {
                        if *offset < (pos as i16) {
                            *offset -= if switch { count as i16 } else { -(count as i16) };
                        }
                    }
                },

                IRNode::PushAddressFromOffset(offset) => {
                    if i < pos {
                        if *offset >= (pos as i16) {
                            *offset -= if switch { count as i16 } else { -(count as i16) };
                        }
                    } else {
                        if *offset < (pos as i16) {
                            *offset -= if switch { count as i16 } else { -(count as i16) };
                        }
                    }
                },

                IRNode::JumpFromOffset(offset) => {
                    if i < pos {
                        if *offset >= (pos as i16) {
                            *offset -= if switch { count as i16 } else { -(count as i16) };
                        }
                    } else {
                        if *offset < (pos as i16) {
                            *offset -= if switch { count as i16 } else { -(count as i16) };
                        }
                    }
                },

                _ => (),
            }

            i += 1;
        }
    }
}

