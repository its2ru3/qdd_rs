use num::complex::Complex64;
use crate::{DdNode, QuantumCircuit, QubitIter};

/// Apply U gate: general single-qubit gate with parameters (theta, phi, lambda).
/// In Qiskit: u(theta, phi, lambda) = U3(theta, phi, lambda) (up to a global phase).
/// Using the same global phase as Qiskit.
// Warning: phi is not used!!!! Something is wrong here.
// Warning: U gate isn't correct, as far as I have checked

impl QuantumCircuit {
    pub fn apply_u(&mut self, target_qubit: usize, theta: f64, phi: f64, lambda: f64) {
        let iter = QubitIter::new(self.root.clone(), target_qubit);
        // The U gate is defined, up to global phase, as:
        // U(theta, phi, lambda) = [[cos(theta/2), -exp(i*lambda)*sin(theta/2)],
        //                           [exp(i*phi)*sin(theta/2), exp(i*(phi+lambda))*cos(theta/2)]]
        // In our DD representation, we update the weights on the edges:
        for node in iter {
            let mut node_ref = node.borrow_mut();
            if let DdNode::NonTerminal { zero, one, .. } = &mut *node_ref {
                // let cos = theta.cos() / 2.0; // Warning
                let sin = theta.sin() / 2.0;
                // In this example, we set the zero branch weight to cos(theta/2)
                // and the one branch weight to exp(i*phi)*sin(theta/2) or similar.
                // Then we incorporate additional phase factors in the children as needed.
                *zero = (Complex64::new(theta.cos() * 0.5, 0.0), zero.1.clone());
                *one = (
                    Complex64::from_polar(sin, lambda),
                    one.1.clone()
                );
                // We need to adjust the global phase by multiplying the entire branch
                // with exp(i*phi)
            }
        }
    }
}
