use std::cell::RefCell;
use std::rc::Rc;
use crate::{DdNode, QuantumCircuit};
use crate::constants::{ONE, ZERO};

impl QuantumCircuit {
    /// Initialize a new quantum state with all qubits in all |0⟩ state
    pub fn new(num_qubits: usize) -> Self {
        let terminal = Rc::new(RefCell::new(DdNode::Terminal(ZERO)));
        let sink = Rc::new(RefCell::new(DdNode::NonTerminal {
            qubit: usize::MAX,
            zero: (ZERO, terminal.clone()),
            one: (ZERO, terminal.clone()),
        }));
        let mut current = sink;

        // Build initial decision diagram structure
        // q_(n-1) is the root node
        for qubit in 0..num_qubits {
            let new_node = Rc::new(RefCell::new(DdNode::NonTerminal {
                qubit,
                zero: (ONE, current.clone()),
                one: (ZERO, terminal.clone()),
            }));
            current = new_node;
        }

        QuantumCircuit {
            root: current,
            num_qubits,
        }
    }
}