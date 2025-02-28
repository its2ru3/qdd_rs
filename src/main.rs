use std::fs::File;
use std::process::Command;
use qdd_rs::QuantumCircuit;
use std::io::Write;

fn main() -> std::io::Result<()> {

    let mut sim = QuantumCircuit::new(6);
    // sim.apply_h(4);
    // sim.apply_cnot(4,2);
    // sim.apply_h(6);
    // sim.apply_cnot(6,0);
    // sim.apply_t(6);
    // sim.apply_cnot(4,0);
    // sim.apply_cnot(6,3);

    for q in 0..6 {
        sim.apply_h(q);
    }
    sim.apply_cz(5,4);
    sim.apply_cz(5,2);
    sim.apply_cz(5,0);
    //
    sim.apply_cz(4,3);
    sim.apply_cz(4,1);
    // //
    sim.apply_cz(3,2);
    println!("========== last gate =======>");
    sim.apply_cz(3,0);
    //
    sim.apply_cz(2,1);
    // //
    sim.apply_cz(1,0);


    // println!("\nState vector:");
    // let state_vector = sim.get_state_vector();
    // // println!("{:?}", state_vector);
    // let threshold = 1e-6; // Set your threshold value
    // for (i, amp) in state_vector.iter().enumerate() {
    //     if amp.norm() > threshold {
    //         println!("|{:0width$b}⟩: {:.3}", i, amp, width = sim.num_qubits);
    //     }
    // }

    // sim.print_adjacency_list();
    // let dot = sim.to_graphviz_dot();
    // let output_image = "graph.png";
    // draw_dot(&dot, output_image)?;
    // println!("Graph image saved as {}", output_image);
    // println!("Number of Terminal and NonTerminal Nodes in DD: {:?}", sim.count_nodes());
    Ok(())
}

fn draw_dot(dot: &str, output_path: &str) -> std::io::Result<()> {
    // Write the DOT string to a temporary file.
    let temp_dot = "temp.dot";
    let mut file = File::create(temp_dot)?;
    file.write_all(dot.as_bytes())?;

    // Use Graphviz to generate a PNG file.
    Command::new("dot")
        .args(&["-Tpng","-Gdpi=600", temp_dot, "-o", output_path])
        .output()?;

    // Optionally, remove the temporary DOT file.
    // std::fs::remove_file(temp_dot)?;
    Ok(())
}