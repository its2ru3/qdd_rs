use num::complex::Complex64;
use rand::{rng, Rng};
use crate::{DdNode, NodePtr, QuantumCircuit};

/// Measure a single qubit, returning the measurement result
impl QuantumCircuit {
    pub fn measure_qubit(&mut self, qubit: usize) -> u8 {
        let (prob0, _) = self.calculate_probabilities(qubit);
        let mut rng = rng();
        if rng.random::<f64>() < prob0 { 0 } else { 1 }
    }

    /// Calculate probabilities for |0⟩ and |1⟩ states of a qubit
    pub fn calculate_probabilities(&self, target: usize) -> (f64, f64) {
        let mut prob0 = 0.0;
        let mut prob1 = 0.0;
        self.traverse(&self.root, Complex64::new(1.0, 0.0), target, &mut prob0, &mut prob1);
        let total = prob0 + prob1;
        (prob0 / total, prob1 / total)
    }

    fn traverse(&self, node: &NodePtr, mut amplitude: Complex64, target: usize, prob0: &mut f64, prob1: &mut f64) {
        let node_ref = node.borrow();
        match &*node_ref {
            DdNode::Terminal(_) => {
                let prob = amplitude.norm_sqr();
                if target == 0 {
                    *prob0 += prob;
                }
            }
            DdNode::NonTerminal { qubit, zero, one } => {
                amplitude *= zero.0;
                self.traverse(&zero.1, amplitude, target, prob0, prob1);

                amplitude = one.0;
                self.traverse(&one.1, amplitude, target, prob0, prob1);

                if *qubit == target {
                    *prob1 += amplitude.norm_sqr();
                }
            }
        }
    }
}
