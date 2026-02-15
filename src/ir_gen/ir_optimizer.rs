use crate::ir_gen::{cmpld_program::CompiledProgram, ir::IRNode};


pub struct IROptimizer<'a> {
    cprog: &'a mut CompiledProgram<'a>,
}

impl<'a> IROptimizer<'a> {
    pub fn new(cprog: &'a mut CompiledProgram<'a>) -> Self {
        Self {
            cprog,
        }
    }

    pub fn optimize(&mut self) -> &CompiledProgram<'a> {
        loop {
            if self.do_pass() <= 0{
                break
            }
        }

        self.cprog
    }

    /// returns optimization count
    #[warn(unused_results)]
    pub fn do_pass(&mut self) -> usize {

        let mut optimize_count = 0;
    
        let mut i = 0;
        while (i + 1) < self.cprog.count_ir() {

            match self.cprog.node_clone_at(i) {

                IRNode::StackAlloc(0) => {
                    self.cprog.shift_nodes(i..=(i));
                    optimize_count += 1;
                    continue
                },

                IRNode::StackDealloc(0) => {
                    self.cprog.shift_nodes(i..=(i));
                    optimize_count += 1;
                    continue
                },

                IRNode::StackAlloc(size) => match self.cprog.node_clone_at(i+1) {

                    IRNode::StackAlloc(other_size) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::StackAlloc(size + other_size));
                        optimize_count += 1;
                        continue
                    },

                    IRNode::StackDealloc(other_size) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        if size > other_size {
                            self.cprog.insert_node(i, IRNode::StackAlloc(size - other_size));
                        } else {
                            self.cprog.insert_node(i, IRNode::StackDealloc(other_size - size));
                        }
                        optimize_count += 1;
                        continue
                    },

                    IRNode::Load64ToStack(int, 0) if size == 8 => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::Push64(int));
                        optimize_count += 1;
                        continue
                    },

                    _ => (),
                },

                IRNode::StackDealloc(size) => match self.cprog.node_clone_at(i+1) {

                    IRNode::StackDealloc(other_size) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::StackDealloc(size + other_size));
                        optimize_count += 1;
                        continue
                    },

                    IRNode::StackAlloc(other_size) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        if size > other_size {
                            self.cprog.insert_node(i, IRNode::StackDealloc(size - other_size));
                        } else {
                            self.cprog.insert_node(i, IRNode::StackAlloc(other_size - size));
                        }
                        optimize_count += 1;
                        continue
                    },

                    _ => (),
                },

                IRNode::Push64(int) => match self.cprog.node_clone_at(i+1) {

                    IRNode::StackDealloc(8) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        optimize_count += 1;
                        continue
                    },

                    IRNode::Pop64ToStack(offset) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::Load64ToStack(int, offset - 8));
                        optimize_count += 1;
                        continue
                    },

                    _ => (),
                },

                IRNode::GlobalReadPush64(global_pos) => match self.cprog.node_clone_at(i+1) {

                    IRNode::StackDealloc(8) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        optimize_count += 1;
                        continue
                    },

                    IRNode::Pop64ToStack(offset) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::GlobalReadLoad64ToStack(global_pos, offset - 8));
                        optimize_count += 1;
                        continue
                    },

                    _ => (),
                },

                IRNode::StackReadPush64(src_offset) => match self.cprog.node_clone_at(i+1) {

                    IRNode::StackDealloc(8) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        optimize_count += 1;
                        continue
                    },

                    IRNode::Pop64ToStack(dst_offset) => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::StackReadLoad64ToStack(src_offset, dst_offset - 8));
                        optimize_count += 1;
                        continue
                    },

                    _ => (),
                },

                IRNode::PushAddressFromOffset(offset) => match self.cprog.node_clone_at(i+1) {

                    IRNode::Call => {
                        self.cprog.shift_nodes(i..=(i+1));
                        self.cprog.insert_node(i, IRNode::CallFromOffset(offset + 1));
                        optimize_count += 1;
                        continue;
                    },

                    _ => (),
                },

                _ => (),
            }

            i += 1;

        }

        optimize_count
    }
}

