use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use crate::{DdNode, NodePtr, QuantumCircuit, QubitIter};
use crate::constants::{INV_ROOT_TWO, ZERO};

impl QuantumCircuit {
    /// Apply Hadamard gate to a qubit
    pub fn apply_h(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        // println!("Size of iter: {}", iter.stack.len());
        for node in iter {
            // println!("For qubit {target_qubit}");
            Self::apply_h_on_node(node);
        }
    }
    fn apply_h_on_node(node: NodePtr) {
        let terminal = Rc::new(RefCell::new(DdNode::Terminal(ZERO)));
        // let terminal = Rc::new(RefCell::new(TERMINAL));
        // let terminal = TERMINAL.clone();
        let mut node_ref = node.borrow_mut();
        if let DdNode::NonTerminal { zero, one, .. } = &mut *node_ref {
            let new_zero_weight = (zero.0 + one.0) * INV_ROOT_TWO;
            let new_one_weight = (zero.0 - one.0) * INV_ROOT_TWO;
            // println!("For qubit: {qubit}");
            // println!("{} {}", one.0, zero.0);
            // println!("{} \n\n\n {}", one.1.borrow(), zero.1.borrow());

            if Rc::ptr_eq(&zero.1, &one.1) {
                // Same child node case
                assert!((zero.0.norm() - one.0.norm()).abs() < 1e-12,
                        "Weights must have the same magnitude in this case");
                // Warning: if the weights have phase, that phase need to be propagated
                if (zero.0 - one.0).norm() < 1e-12 { // same sign
                    *zero = (new_zero_weight, zero.1.clone());
                    *one = (ZERO, terminal.clone());
                } else { // opposite sign
                    *zero = (ZERO, terminal.clone());
                    *one = (new_one_weight, one.1.clone());
                }
            } else {
                let zero_terminal = matches!(&*zero.1.borrow(), DdNode::Terminal(_));
                let one_terminal = matches!(&*one.1.borrow(), DdNode::Terminal(_));

                match (zero_terminal, one_terminal) {
                    (true, true) => { return }
                    // Zero is TERMINAL - use one's child, with opposite signed weight of one
                    (true, false) => {
                        let new_child = one.1.clone();
                        zero.0 = new_zero_weight;
                        zero.1 = new_child.clone();
                        one.0 = new_one_weight;
                        one.1 = new_child;
                    }
                    // One is TERMINAL - use zero's child, with same signed weights
                    (false, true) => {
                        let new_child = zero.1.clone();
                        zero.0 = new_zero_weight;
                        zero.1 = new_child.clone();
                        one.0 = new_one_weight; // it is minus already
                        one.1 = new_child;
                    }

                    // Both non-TERMINAL -
                    (false, false) => {
                        let (parent1, parent2) = find_parent_of_common_node(&zero.1, &one.1);
                        Self::apply_h_on_node(parent1);
                        Self::apply_h_on_node(parent2);
                    }
                }
            }
        }
    }
}



/// Helper: returns the active child (the branch with a nonzero weight) of a NonTerminal node.
pub(crate) fn active_child(node_ref: Ref<DdNode>) -> NodePtr {
    // let node_ref = node.borrow();
    if let DdNode::NonTerminal { zero, one, .. } = &*node_ref {
        // We assume that exactly one branch is active (nonzero weight).
        if zero.0 != ZERO {
            zero.1.clone()
        } else if one.0 != ZERO {
            one.1.clone()
        } else {
            panic!("Both branches have zero weight; invalid decision diagram state.")
        }
    } else {
        panic!("active_child called on a Terminal node.")
    }
}

/// Recursively follows the active child of each branch until they converge.
/// Returns a pair of nodes (one from each branch) such that their active children are identical.
/// These nodes are the parents immediately before the meeting point.
pub fn find_parent_of_common_node(zero_node: &NodePtr, one_node: &NodePtr) -> (NodePtr, NodePtr) {
    let next_zero = active_child(zero_node.borrow());
    let next_one  = active_child(one_node.borrow());
    if Rc::ptr_eq(&next_zero, &next_one) {
        // The active children meet, so return the current parent nodes.
        (zero_node.clone(), one_node.clone())
    } else {
        // Otherwise, continue recursing down the active paths.
        find_parent_of_common_node(&next_zero, &next_one)
    }
}

