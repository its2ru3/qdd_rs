use num::complex::Complex64;
use crate::{DdNode, QuantumCircuit, QubitIter};
use crate::constants::MINUS_ONE;

mod apply_h;
mod apply_cnot;
mod apply_u;
mod apply_cz;
impl QuantumCircuit {
    /// Apply Pauli-X gate (bit flip)
    pub fn apply_x(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            // println!("{}", node_ref);
            if let DdNode::NonTerminal { zero, one, .. } = &mut *node_ref {
                // Swap zero and one edges
                std::mem::swap(zero, one);
            }
        }
    }

    /// Z gate: |0⟩ unchanged, |1⟩ gets a -1 phase.
    pub fn apply_z(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        let z_phase = MINUS_ONE;
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                one.0 = one.0 * z_phase;
            }
        }
    }
    /// Y gate: swaps arms with phases (Y = [[0, -i], [i, 0]]).
    pub fn apply_y(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        let i_phase = Complex64::new(0.0, 1.0);
        let minus_i_phase = Complex64::new(0.0, -1.0);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { zero, one, .. } = &mut *node_ref {
                let new_zero = (one.0 * i_phase, one.1.clone());
                let new_one  = (zero.0 * minus_i_phase, zero.1.clone());
                *zero = new_zero;
                *one  = new_one;
            }
        }
    }

    /// S gate: applies a π/2 phase (i) on the |1⟩ branch.
    pub fn apply_s(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        let s_phase = Complex64::new(0.0, 1.0);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                one.0 = one.0 * s_phase;
            }
        }
    }

    /// Apply Sdg gate: phase of -i on |1⟩ branch.
    pub fn apply_sdg(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        let s_dg_phase = Complex64::new(0.0, -1.0);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                one.0 *= s_dg_phase;
            }
        }
    }

    /// Apply T gate: multiplies the |1⟩ branch by exp(i*pi/4).
    pub fn apply_t(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        let t_phase = Complex64::from_polar(1.0, std::f64::consts::PI / 4.0);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                one.0 *= t_phase;
            }
        }
    }

    /// Apply Tdg gate: multiplies the |1⟩ branch by exp(-i*pi/4).
    pub fn apply_tdg(&mut self, target_qubit: usize) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        let tdg_phase = Complex64::from_polar(1.0, -std::f64::consts::PI / 4.0);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                one.0 *= tdg_phase;
            }
        }
    }

    /// Apply P gate: corresponds to the phase gate. Parameter theta.
    /// In Qiskit, U1(theta) is equivalent to P(theta).
    pub fn apply_p(&mut self, target_qubit: usize, theta: f64) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        // The global phase convention from Qiskit is that:
        // U1(theta) = P(theta) = diag(1, exp(i*theta))
        let phase = Complex64::from_polar(1.0, theta);
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { one, .. } = &mut *node_ref {
                one.0 *= phase;
            }
        }
    }

}