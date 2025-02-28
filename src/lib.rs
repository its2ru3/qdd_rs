mod quantum_circuit;
mod gates;
mod draw;
mod dd_node;
mod constants;

// use lazy_static::lazy_static;
// lazy_static! { // --> doesn't work
//     pub static ref TERMINAL: NodePtr = Rc::new(RefCell::new(dd_node::Terminal(ZERO)));
// }
// const TERMINAL: dd_node = dd_node::Terminal(ZERO);
// let terminal: NodePtr = Rc::new(RefCell::new(TERMINAL));
// let terminal = TERMINAL.clone();
// ======= use Arc<Mutex<_>> for one global TERMINAL node ========
// use std::sync::{Arc, Mutex};
// type NodePtr = Arc<Mutex<dd_node>>;
// lazy_static::lazy_static! {
//     pub static ref TERMINAL: NodePtr = Arc::new(Mutex::new(dd_node::Terminal(ZERO)));

use num::complex::Complex64;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

type NodePtr = Rc<RefCell<DdNode>>;
#[derive(Debug)]
pub enum DdNode {
    Terminal(Complex64),
    NonTerminal {
        qubit: usize,
        zero: (Complex64, NodePtr),
        one: (Complex64, NodePtr),
    },
}

pub struct QuantumCircuit {
    root: NodePtr,
    pub num_qubits: usize,
}
pub struct QubitIter {
    stack: Vec<NodePtr>,
    target: usize,
    visited: HashSet<*const DdNode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;
    use num::complex::Complex64;
    use crate::constants::{ONE, ZERO};

    const TOL: f64 = 1e-6;

    fn assert_complex_eq(a: Complex64, b: Complex64) {
        assert!(
            approx_eq!(f64, a.re, b.re, epsilon = TOL) &&
                approx_eq!(f64, a.im, b.im, epsilon = TOL),
            "Expected {} got {}", b, a
        );
    }

    #[test]
    fn test_h_and_cx_gates() {
        // |0⟩ -> H -> |+⟩
        let mut sim = QuantumCircuit::new(3);
        sim.apply_h(2);
        sim.apply_cnot(2,0);

        sim.apply_cnot(2,0);
        sim.apply_h(2);
        let state = sim.get_state_vector();

        let mut expected = vec![ZERO; 8];
        expected[0] = ONE;

        assert_eq!(state.len(), 8);
        for q in 0..8 {
            assert_complex_eq(state[q], expected[q]);
        }
        let expected_adjacency_list = "
            |000⟩: 0.707+0.000i
            |100⟩: 0.707+0.000i
            Node: L00_Q2
              Edge: 0: 0.707+0.000i -> L01_Q1
              Edge: 1: 0.707+0.000i -> L04_Q1
            Node: L01_Q1
              Edge: 0: 1.000+0.000i -> L02_Q0
              Edge: 1: 0.000+0.000i -> T
            Node: L02_Q0
              Edge: 0: 1.000+0.000i -> Sink
              Edge: 1: 0.000+0.000i -> T
            Node: L04_Q1
              Edge: 0: 1.000+0.000i -> L05_Q0
              Edge: 1: 0.000+0.000i -> T
            Node: L05_Q0
              Edge: 0: 1.000+0.000i -> Sink
              Edge: 1: 0.000+0.000i -> T
            Node: Sink
              Edge: 0: 0.000+0.000i -> T
              Edge: 1: 0.000+0.000i -> T
            Node: T";

    }
}
