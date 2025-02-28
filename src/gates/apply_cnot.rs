use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::{DdNode, NodePtr, QuantumCircuit, QubitIter};
use crate::constants::{TOL};
use super::apply_h;
impl QuantumCircuit {
    pub fn apply_cnot(&mut self, control: usize, target: usize) {
        let iter = QubitIter::new(self.root.clone(), control);

        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { zero, one, .. } = &mut *node_ref {
                if Rc::ptr_eq(&zero.1, &one.1) {
                    // let one_branch = one.1.clone();
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_swap(&one.1, target, &mut visited);
                    one.1 = copied_branch;
                    println!("Inside matching the arms case of CNOT gate.");

                } else {
                    // when there is a merger of the two branches before the target qubit,
                    // we need to deep_copy
                    let mut marked:HashSet<usize> = HashSet::new();
                    mark_nodes_of_other_branch(&zero.1, target, &mut marked);
                    println!("Marked nodes: {:?}", marked);
                    let mut visited: HashSet<usize> = HashSet::new();
                    self.traverse_and_swap(&one.1, target, &mut visited, & marked);
                    let similar = compare_subtrees(&zero.1, &one.1);
                    println!("Are the two branches similar, after CNOT: {similar}");
                    if similar {
                        one.1 = zero.1.clone();
                    }
                }
            }
        }
    }

    // ToDo: could be more optimized, there could be some weird cases
    fn traverse_and_swap(
        &self, node: &NodePtr, target: usize,
        visited: &mut HashSet<usize>,
        marked: &HashSet<usize>
    ) {
        let mut node_ref = node.borrow_mut();

        match &mut *node_ref {
            DdNode::Terminal(_) => { return }
            DdNode::NonTerminal { qubit, zero, one } => {
                let key1 = Rc::as_ptr(&one.1) as usize; // checking 'one' branch
                let key0 = Rc::as_ptr(&zero.1) as usize; // checking 'zero' branch
                // check if the current node is in other branch also
                println!("Inside traverse_and_swap: qubit = {qubit}");
                if marked.contains(&key1) {
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_swap(&one.1, target, &mut visited);
                    one.1 = copied_branch;
                    println!("------- From ONE branch ------");
                    return
                }
                else if marked.contains(&key0) {
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_swap(&zero.1, target, &mut visited);
                    zero.1 = copied_branch;
                    println!("------- From ZERO branch ------");
                    return
                }
                if *qubit == target {
                    let key = Rc::as_ptr(node) as usize;
                    if !visited.contains(&key) {
                        visited.insert(key);
                        std::mem::swap(zero, one);
                    }
                }  else if *qubit > target {
                    // println!("from inside else if");
                    self.traverse_and_swap(&one.1, target, visited, marked);
                    self.traverse_and_swap(&zero.1, target, visited, marked);
                } else { return }
            }
        }
    }

    /// Deep copies the branch starting at `one_branch` node until the target qubit is reached.
    /// When the target qubit is reached, a new node is created with swapped children,
    /// but the children pointers are not deep copied—this preserves sharing.
    fn deep_copy_with_swap(&self, node: &NodePtr, target: usize, visited: &mut HashMap<*const RefCell<DdNode>, NodePtr>) -> NodePtr {
        let node_ref = node.borrow();
        match &*node_ref {
            DdNode::Terminal(amp) => {
                // Terminal nodes are copied directly.
                Rc::new(RefCell::new(DdNode::Terminal(*amp)))
            }
            DdNode::NonTerminal { qubit, zero, one } => {
                if *qubit == target {
                    // At the target qubit, swap the arms and reuse the original children.
                    let new_zero = (one.0, one.1.clone());
                    let new_one = (zero.0, zero.1.clone());
                    Rc::new(RefCell::new(DdNode::NonTerminal {
                        qubit: *qubit,
                        zero: new_zero,
                        one: new_one,
                    }))
                } else {
                    // For qubits above target, deep copy recursively.
                    let new_zero;
                    let new_one;
                    let key0 = Rc::as_ptr(&zero.1);
                    match visited.get(&key0) {
                        Some(existing_node) => {
                            new_zero = (zero.0, existing_node.clone());
                        }
                        None => {
                            let new_branch = self.deep_copy_with_swap(&zero.1, target, visited);
                            visited.insert(key0, new_branch.clone());
                            new_zero = (zero.0, new_branch);
                        }
                    }
                    let key1 = Rc::as_ptr(&one.1);
                    match visited.get(&key1) {
                        Some(existing_node) => {
                            new_one = (one.0, existing_node.clone());
                        }
                        None => {
                            let new_branch = self.deep_copy_with_swap(&one.1, target, visited);
                            visited.insert(key1, new_branch.clone());
                            new_one = (one.0, new_branch);
                        }
                    }
                    Rc::new(RefCell::new(DdNode::NonTerminal {
                        qubit: *qubit,
                        zero: new_zero,
                        one: new_one,
                    }))
                }
            }
        }
    }
}

pub(crate) fn compare_subtrees(node_a: &NodePtr, node_b: &NodePtr) -> bool {
    let a = node_a.borrow();
    let b = node_b.borrow();
    match (&*a, &*b) {
        (DdNode::Terminal(amp_a), DdNode::Terminal(amp_b)) => {
            // Compare amplitudes (exactly or within tolerance)
            (amp_a - amp_b).norm() < TOL
        },
        (
            DdNode::NonTerminal { qubit: qa, zero: (w0a, child0a), one: (w1a, child1a) },
            DdNode::NonTerminal { qubit: qb, zero: (w0b, child0b), one: (w1b, child1b) }
        ) => {
            if qa != qb {
                return false;
            }
            // Compare the weights of both branches (within tolerance)
            if (w0a - w0b).norm() > TOL || (w1a - w1b).norm() > TOL {
                return false;
            }
            // Recursively compare the children.
            compare_subtrees(child0a, child0b) && compare_subtrees(child1a, child1b)
        },
        _ => false,
    }
    // true
}

/// When the node has two branches and they merger at some point before
/// the target is reached, we mark nodes of the 'zero' branch to match where
/// they merge and then call deep_copy_with_swap function at that node.
pub(crate) fn mark_nodes_of_other_branch(node: &NodePtr, target: usize, marked: &mut HashSet<usize>) {
    let mut node_ref = node.borrow();
    match & *node_ref {
        DdNode::Terminal(_) => { return }
        DdNode::NonTerminal { qubit, .. } => {
            println!("marking --- qubit = {qubit}");
            let key = Rc::as_ptr(node) as usize;
            if !marked.contains(&key) {
                marked.insert(key);
            }
            if *qubit > target {
                let next_node = apply_h::active_child(node_ref);
                mark_nodes_of_other_branch(&next_node, target, marked);
            } else { return }
        }
    }
}
