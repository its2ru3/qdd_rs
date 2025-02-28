use num::complex::Complex64;
use crate::{DdNode, NodePtr, QuantumCircuit};

impl QuantumCircuit {
    /// Returns the full state vector of the circuit.
    /// The state vector is a vector of length 2^(num_qubits) where the index bits
    /// represent the qubit states in order: q3,q2,q1,q0 for a 4‐qubit system.
    pub fn get_state_vector(&self) -> Vec<Complex64> {
        // Initialize state vector with 0 amplitude.
        let mut state_vector = vec![Complex64::new(0.0, 0.0); 1 << self.num_qubits];
        // Start the depth-first traversal from the root.
        // current_amplitude starts at 1, current_index = 0 (no bits set),
        // and mask = 0 (no qubits encountered yet).
        self.traverse_for_statevector(
            &self.root,
            Complex64::new(1.0, 0.0),
            0,  // current index
            0,  // mask of visited qubits
            &mut state_vector,
        );
        state_vector
    }

    /// Recursively traverses the decision diagram in depth-first order.
    ///
    /// - `current_amplitude` accumulates the product of edge weights along the path.
    /// - `current_index` holds the partial basis state (bits set when taking a one-edge).
    /// - `mask` records which qubit positions have been fixed.
    fn traverse_for_statevector(
        &self,
        node: &NodePtr,
        current_amplitude: Complex64,
        current_index: usize,
        mask: usize,
        state_vector: &mut [Complex64],
    ) {
        let node_ref = node.borrow();
        match &*node_ref {
            // Terminal node: amplitude is multiplied by the terminal weight.
            // (In our design, terminal nodes always store ZERO, so this yields 0.)
            DdNode::Terminal(amp) => {
                let final_amp = current_amplitude * amp;
                self.distribute_amplitude(final_amp, current_index, mask, state_vector);
            }
            // NonTerminal node.
            DdNode::NonTerminal { qubit, zero, one } => {
                if *qubit == usize::MAX {
                    // Sink node: this is the final node. Its children are Terminal with weight 0.
                    // Here, the accumulated amplitude is considered final.
                    self.distribute_amplitude(current_amplitude, current_index, mask, state_vector);
                } else {
                    let q = *qubit;
                    // Mark that qubit q has been decided.
                    let new_mask = mask | (1 << q);
                    // Process the zero branch:
                    let amp_zero = current_amplitude * zero.0;
                    // For zero branch, bit for qubit q remains 0.
                    let index_zero = current_index;
                    self.traverse_for_statevector(&zero.1, amp_zero, index_zero, new_mask, state_vector);

                    // Process the one branch:
                    let amp_one = current_amplitude * one.0;
                    // For one branch, set bit for qubit q.
                    let index_one = current_index | (1 << q);
                    self.traverse_for_statevector(&one.1, amp_one, index_one, new_mask, state_vector);
                }
            }
        }
    }

    /// Distributes the given amplitude over all complete basis state indices
    /// that are consistent with the partial assignment.
    ///
    /// - `base_index` is the partial index built so far.
    /// - `mask` indicates which qubit positions have been fixed.
    /// For each unfixed qubit, both 0 and 1 are allowed.
    fn distribute_amplitude(
        &self,
        amplitude: Complex64,
        base_index: usize,
        mask: usize,
        state_vector: &mut [Complex64],
    ) {
        // Determine the list of qubit positions that have not been fixed.
        let mut remaining_qubits = Vec::new();
        for q in 0..self.num_qubits {
            if (mask & (1 << q)) == 0 {
                remaining_qubits.push(q);
            }
        }
        let num_remaining = remaining_qubits.len();
        let combinations = 1 << num_remaining; // 2^(# remaining)
        // For each combination, assign bits to the remaining positions.
        for combo in 0..combinations {
            let mut final_index = base_index;
            for (i, &q) in remaining_qubits.iter().enumerate() {
                // If the i-th bit of combo is 1, set qubit q to 1.
                if (combo >> i) & 1 == 1 {
                    final_index |= 1 << q;
                }
            }
            // Accumulate the amplitude into the state vector.
            state_vector[final_index] += amplitude;
        }
    }
}


#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::*;
    use float_cmp::approx_eq;
    use num::complex::Complex64;
    use crate::constants::{HALF, INV_ROOT_TWO, MINUS_ONE, ONE, ZERO};

    const TOL: f64 = 1e-6;

    fn assert_complex_eq(a: Complex64, b: Complex64) {
        assert!(
            approx_eq!(f64, a.re, b.re, epsilon = TOL) &&
                approx_eq!(f64, a.im, b.im, epsilon = TOL),
            "Expected {} got {}", a, b
        );
    }

    #[test]
    fn test_basic_single_qubit() {
        // |0⟩ -> H -> |+⟩
        let mut sim = QuantumCircuit::new(1);
        sim.apply_h(0);
        let state = sim.get_state_vector();

        let expected = vec![INV_ROOT_TWO, INV_ROOT_TWO];

        assert_eq!(state.len(), 2);
        assert_complex_eq(state[0], expected[0]);
        assert_complex_eq(state[1], expected[1]);
    }

    #[test]
    fn test_sink_node_distribution() {
        let terminal = Rc::new(RefCell::new(DdNode::Terminal(ZERO)));
        let sink = Rc::new(RefCell::new(DdNode::NonTerminal {
            qubit: usize::MAX,
            zero: (ZERO, Rc::new(RefCell::new(DdNode::Terminal(ZERO)))),
            one: (ZERO, Rc::new(RefCell::new(DdNode::Terminal(ZERO)))),
        }));

        let q0 = Rc::new(RefCell::new(DdNode::NonTerminal {
            qubit: 1,
            zero: (ONE, sink.clone()),
            one: (ONE, sink.clone()),
        }));
        let q1 = Rc::new(RefCell::new(DdNode::NonTerminal {
            qubit: 1,
            zero: (ONE, q0.clone()),
            one: (ONE, terminal.clone()),
        }));

        let sim = QuantumCircuit {
            root: q1,
            num_qubits: 2,
        };

        let state = sim.get_state_vector();
        let expected = Complex64::new(2.0, 0.0); // 1.0 * 1.0 * 2 combinations

        assert_complex_eq(state[0], expected);
        assert_complex_eq(state[1], expected);
        assert_complex_eq(state[2], expected);
        assert_complex_eq(state[3], expected);
    }

    #[test]
    fn test_early_terminal() {
        // 3-qubit system terminating at q1
        let terminal = Rc::new(RefCell::new(DdNode::Terminal(ZERO)));

        let q1 = Rc::new(RefCell::new(DdNode::NonTerminal {
            qubit: 1,
            zero: (ONE, terminal.clone()),
            one: (ONE, terminal.clone()),
        }));

        let q2 = Rc::new(RefCell::new(DdNode::NonTerminal {
            qubit: 2,
            zero: (ONE, q1.clone()),
            one: (ONE, q1.clone()),
        }));

        let sim = QuantumCircuit {
            root: q2,
            num_qubits: 3,
        };

        let state = sim.get_state_vector();
        let expected = ZERO;

        // Should affect all states where q2 and q1 are set
        for i in 0..8 {
            let bits = i >> 1; // q0 is unprocessed
            if (i & 0b110) == 0b000 { // q2=0, q1=0
                assert_complex_eq(state[i], expected);
            }
        }
    }

    #[test]
    fn test_full_processing_no_unused() {
        let mut sim = QuantumCircuit::new(2);
        sim.apply_h(0);
        sim.apply_h(1);

        let state = sim.get_state_vector();
        assert_complex_eq(state[0], HALF); // 00
        assert_complex_eq(state[1], HALF); // 01
        assert_complex_eq(state[2], HALF); // 10
        assert_complex_eq(state[3], HALF); // 11
    }

    #[test]
    fn test_negative_and_complex_weights() {
        // Test phase accumulation
        let mut sim = QuantumCircuit::new(1);
        sim.apply_h(0);
        sim.apply_z(0); // Z-gate adds π phase to |1⟩
        sim.apply_y(0); // Y-gate adds π phase around Y-axis
        let state = sim.get_state_vector();

        assert_complex_eq(state[0], ONE);
        assert_complex_eq(state[1], MINUS_ONE);
    }
}