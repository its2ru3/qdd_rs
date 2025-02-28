use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::{DdNode, NodePtr, QuantumCircuit};

impl QuantumCircuit {
    /// Returns a tuple (terminal_count, nonterminal_count) for the DD.
    /// If two arms point to the same terminal node, it is counted only once;
    /// if they point to two distinct terminal nodes (even if the values are the same),
    /// they are counted separately.
    pub fn count_nodes(&self) -> (usize, usize) {
        let mut visited = HashSet::new();
        Self::count_nodes_rec(&self.root, &mut visited)
    }

    fn count_nodes_rec(
        node: &NodePtr,
        visited: &mut HashSet<*const RefCell<DdNode>>,
    ) -> (usize, usize) {
        // Use the raw pointer address as a key.
        let key = Rc::as_ptr(node);
        if visited.contains(&key) {
            return (0, 0);
        }
        visited.insert(key);

        let node_ref = node.borrow();
        match &*node_ref {
            DdNode::Terminal(_) => (1, 0),
            DdNode::NonTerminal { zero, one, .. } => {
                let (t0, n0) = Self::count_nodes_rec(&zero.1, visited);
                let (t1, n1) = Self::count_nodes_rec(&one.1, visited);
                (t0 + t1, n0 + n1 + 1)
            }
        }
    }
}
