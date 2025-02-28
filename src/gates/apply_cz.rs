use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::{DdNode, NodePtr, QuantumCircuit, QubitIter};
use crate::constants::{MINUS_ONE, TOL};
use super::{apply_cnot, apply_h};
impl QuantumCircuit {
    pub fn apply_cz(&mut self, control: usize, target: usize) {
        let iter = QubitIter::new(self.root.clone(), control);

        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { zero, one, .. } = &mut *node_ref {
                if Rc::ptr_eq(&zero.1, &one.1) {
                    // let one_branch = one.1.clone();
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_z(&one.1, target, &mut visited);
                    one.1 = copied_branch;
                    // println!("Inside matching the arms case of CNOT gate.");
                } else {
                    // when there is a merger of the two branches before the target qubit,
                    // we need to deep_copy
                    let mut marked:HashSet<usize> = HashSet::new();
                    apply_cnot::mark_nodes_of_other_branch(&zero.1, target, &mut marked);
                    // println!("Marked nodes: {:?}", marked);
                    let mut visited: HashSet<usize> = HashSet::new();
                    let mut done = false;
                    self.traverse_and_z(&one.1, target, &mut visited, & marked, &mut done);
                    let similar = apply_cnot::compare_subtrees(&zero.1, &one.1);
                    println!("Are the two branches similar, after CZ: {similar}");
                    if similar {
                        one.1 = zero.1.clone();
                    }
                }
            }
        }
    }

    // ToDo: could be more optimized, there could be some weird cases
    fn traverse_and_z(
        &self, node: &NodePtr, target: usize,
        visited: &mut HashSet<usize>,
        marked: &HashSet<usize>,
        done: &mut bool
    ) {
        let mut node_ref = node.borrow_mut();

        match &mut *node_ref {
            DdNode::Terminal(_) => { return }
            DdNode::NonTerminal { qubit, zero, one } => {
                let key1 = Rc::as_ptr(&one.1) as usize; // checking 'one' branch
                let key0 = Rc::as_ptr(&zero.1) as usize; // checking 'zero' branch
                // check if the current node is in other branch also
                println!("Inside traverse_and_z: qubit = {qubit}");
                if marked.contains(&key0) && marked.contains(&key1) {
                    println!("------- From ZERO and ONE branch ------");
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_z(&one.1, target, &mut visited);
                    one.1 = copied_branch.clone();
                    zero.1 = copied_branch;
                    *done = true;
                    return
                }
                else if marked.contains(&key1) {
                    println!("------- From ONE branch ------");
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_z(&one.1, target, &mut visited);
                    one.1 = copied_branch;
                    *done = true;
                    return
                }
                else if marked.contains(&key0) {
                    println!("------- From ZERO branch ------");
                    let mut visited = HashMap::new();
                    let copied_branch = self.deep_copy_with_z(&zero.1, target, &mut visited);
                    zero.1 = copied_branch;
                    *done = true;
                    return
                }
                if *qubit == target {
                    let key = Rc::as_ptr(node) as usize;
                    if !visited.contains(&key) {
                        visited.insert(key);
                        let mut node_ref = node.borrow_mut();
                        if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                            one.0 = one.0 * MINUS_ONE;
                        }
                    }
                }  else if *qubit > target {
                    if *done == true { return }
                    // println!("from inside else if");
                    println!("Inside else if q > target -- going ZERO from q = {qubit}");
                    self.traverse_and_z(&zero.1, target, visited, marked, done);
                    println!("Inside else if q > target -- going ONE from q = {qubit}");
                    self.traverse_and_z(&one.1, target, visited, marked, done);
                } else { return }
            }
        }
    }

    /// Deep copies the branch starting at `one_branch` node until the target qubit is reached.
    /// When the target qubit is reached, a new node is created with swapped children,
    /// but the children pointers are not deep copied—this preserves sharing.
    fn deep_copy_with_z(&self, node: &NodePtr, target: usize, visited: &mut HashMap<*const RefCell<DdNode>, NodePtr>) -> NodePtr {
        let node_ref = node.borrow();
        match &*node_ref {
            DdNode::Terminal(amp) => {
                // Terminal nodes are copied directly.
                Rc::new(RefCell::new(DdNode::Terminal(*amp)))
            }
            DdNode::NonTerminal { qubit, zero, one } => {
                println!("Inside CZ gate - deep copying - qubit = {qubit}");
                if *qubit == target {
                    // At the target qubit, swap the arms and reuse the original children.
                    let new_zero = (zero.0, zero.1.clone());
                    let new_one = (MINUS_ONE*one.0, one.1.clone());
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
                            let new_branch = self.deep_copy_with_z(&zero.1, target, visited);
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
                            let new_branch = self.deep_copy_with_z(&one.1, target, visited);
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

