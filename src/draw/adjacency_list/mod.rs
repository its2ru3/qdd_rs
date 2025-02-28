use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use crate::{DdNode, NodePtr, QuantumCircuit};

impl QuantumCircuit {
    pub fn print_adjacency_list(&self) {
        let adj = self.get_adjacency_list();
        for (node, edges) in adj {
            println!("Node: {}", node);
            for (label, child) in edges {
                println!("  Edge: {} -> {}", label, child);
            }
        }
    }
    /// Returns an adjacency list for the decision diagram.
    /// Each non-TERMINAL node is given a unique name like "N0_Q1"
    /// (with `level` and its qubit index), and every TERMINAL node is merged and named "T".
    /// The returned BTreeMap maps node names to a Vec of (edge label, children node name) tuples.
    pub fn get_adjacency_list(&self) -> BTreeMap<String, Vec<(String, String)>> {
        let mut adj: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        let mut visited: HashMap<usize, String> = HashMap::new();
        let mut level = 0;
        // Start recursion from the root. For non-terminals, level is defined by its stored qubit.
        self.collect_adj(&self.root, &mut adj, &mut visited, &mut level);
        adj
    }

    /// Recursive helper to traverse the DD and populate the adjacency list.
    /// It returns the unique name of the node.
    pub(crate) fn collect_adj(
        &self,
        node: &NodePtr,
        adj: &mut BTreeMap<String, Vec<(String, String)>>,
        visited: &mut HashMap<usize, String>,
        level: &mut usize,
    ) -> String {
        let key = Rc::as_ptr(node) as usize;
        // If already visited, return its name.
        if let Some(name) = visited.get(&key) {
            return name.clone();
        }
        let node_ref = node.borrow();
        match &*node_ref {
            DdNode::Terminal(_) => {
                // Terminal nodes are merged into one with special name "T".
                println!("=========== From inside the T node ==========");
                // All T nodes are not same!!!!!!!!!
                let name = "T".to_string();
                visited.insert(key, name.clone());
                // Ensure TERMINAL node appears in the adj list.
                adj.entry(name.clone()).or_default();
                name
            }
            DdNode::NonTerminal { qubit, zero, one } => {
                // Create a unique name for the non-TERMINAL node.
                let name ;
                if *qubit == usize::MAX {
                    name = "Sink".to_string();
                } else {
                    // Unique name based on structure (qubit + children pointers)
                    // let zero_ptr = Rc::as_ptr(&zero.1) as usize;
                    // let one_ptr = Rc::as_ptr(&one.1) as usize;
                    // format!("Q{}_{:x}_{:x}", qubit, zero_ptr, one_ptr)
                    name = format!("L{:02}_Q{}", level, qubit)
                }

                *level += 1;
                visited.insert(key, name.clone());
                // Recursively process both children.
                let child_zero = self.collect_adj(&zero.1, adj, visited, level);
                let child_one  = self.collect_adj(&one.1,  adj, visited, level);
                // Prepare edge labels with branch and weight.
                let weight_zero = format!("{:.3}+{:.3}i", zero.0.re, zero.0.im);
                let weight_one  = format!("{:.3}+{:.3}i", one.0.re, one.0.im);
                let edges = vec![
                    (format!("0: {}", weight_zero), child_zero),
                    (format!("1: {}", weight_one),  child_one),
                ];
                adj.insert(name.clone(), edges);
                name
            }
        }
    }
}
