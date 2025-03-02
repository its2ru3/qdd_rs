# qdd_rs
A Quantum Circuit Simulator using Decision Diagrams and Error Correction in Rust

_This project is developed under the QOSF Mentorship Program._ (in progress)

### Overview

In this project I tried to implement a decision diagram (DD)–based quantum circuit simulator inspired by MQT's DDSIM ([mqt-ddsim on GitHub](https://github.com/cda-tum/mqt-ddsim)). Unlike DDSIM—which simulates circuits by performing multiplication and addition on decision diagrams—this simulator takes a different approach by implementing each quantum gate individually with dedicated functions for handling each case. In short, the simulation is built around explicit, gate-by-gate routines for operations such as H, CNOT, U, and various single and two qubit gates.

While DDSIM is not extended to multi-core architectures, this implementation is structured so that it can be readily extended to run on multiple cores. This can be achieved by parallelizing the gate execution on all the matching nodes (with qubit value equal to target) of the decision diagram and by executing a series of quantum gates on a single qubit concurrently using the effective Unitary operation.

There are a lot of unchecked bugs or so to say, not added cases. I need to identify all possible cases, especially the H(or U) and CX(or any controlled) gates. I can see that the addition and multiplications are not the hard parts, the most critical part is removing the redundant nodes which needs to traverse the whole graph more often. So, when and to which part of dd we have to traverse for redundancies is more important.

### Features

- **Gate-by-Gate Implementation:**  
    Each gate (H, CX, U, and controlled gates) is implemented as a distinct function/method, allowing for fine-grained control and easy extension.
    
- **Custom Decision Diagram Structure:**  
    The simulator represents quantum circuits using a decision diagram where nodes are defined as either Terminal or NonTerminal (with special handling for sink node). The design leverages Rust’s `Rc<RefCell<...>>` to manage shared ownership.
    
- **State Vector Extraction & Visualization:**  
    Functions are provided to traverse the DD and extract the full state vector, as well as to generate Graphviz DOT strings and image for visualizing the circuit’s structure.


### Examples

#### Simulating a Simple Circuit

Clone or fork the repo and modify the main.rs file as following.

```rust
// ---- Inside your main function ----
// Create a quantum simulator with 3 qubits.
let mut circ = QuantumSimulator::new(3);
// Apply a Hadamard gate to qubit 2.
circ.apply_h(2);
// Apply a CNOT gate with control qubit 2 and target qubit 1.
circ.apply_cnot(2, 1);
// Extract the state vector.
let state_vector = circ.get_full_state_vector();
println!("{:?}", state_vector);
// Generating a Graphviz Diagram
// Generate a DOT string for visualizing the decision diagram.
let dot_str = sim.to_graphviz_dot();
println!("{}", dot_str);
```

### Benchmarking
Because of the limited time and academic commitments, benchmarking has not been performed yet. I plan to do detailed performance evaluations in the future commits.

### Comment on Project Name
The name `qdd_rs` stands for **Quantum Decision Diagram in Rust, Single Core**. I thought of `ddsim_rs` but refrained because [Munich Quantum Software Company](https://munichquantum.software/) might argue.

### Future Work

- **Expand Gate Implementations:**  
    Complete the missing cases for H, CX, U, and controlled gates.
    
- **Integration & Testing:**  
    Develop a comprehensive suite of test cases and integrate with other quantum circuit libraries for broader applicability.
    
- **Optimize Performance:**  
    Leverage parallelism by extending the simulator to multi-core architectures and further optimizing memory usage.

and many more optimisation techniques. 

### Acknowledgements

This project is developed under the QOSF Mentorship Program. Special thanks to the creators of DDSIM by MQT and other researchers for their pioneering work on decision diagram–based quantum circuit simulation.

## License

This project is licensed under the [MIT License](LICENSE).
