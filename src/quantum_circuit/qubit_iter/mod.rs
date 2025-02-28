use std::collections::HashSet;
use std::ops::Deref;
use crate::{DdNode, NodePtr, QubitIter};

impl QubitIter {
    pub fn new(root: NodePtr, target: usize) -> Self {
        Self {
            stack: vec![root],
            target,
            visited: HashSet::new(),
        }
    }
}

impl Iterator for QubitIter {
    type Item = NodePtr;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            let node_ptr = {
                let node_ref = node.borrow();
                node_ref.deref() as *const DdNode
            };
            if self.visited.contains(&node_ptr) {
                continue;
            }
            self.visited.insert(node_ptr);

            let node_ref = node.borrow();
            match &*node_ref {
                DdNode::Terminal(_) => { continue; }
                DdNode::NonTerminal { qubit, zero, one } => {
                    if *qubit == self.target {
                        return Some(node.clone());
                    } else if *qubit > self.target {
                        // since children have lower qubit values
                        self.stack.push(one.1.clone());
                        self.stack.push(zero.1.clone());
                    }
                }
            }
        }
        None
    }
}