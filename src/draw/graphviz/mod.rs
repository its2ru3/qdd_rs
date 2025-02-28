use std::collections::HashMap;
use std::rc::Rc;
use num::complex::Complex64;
use crate::{DdNode, NodePtr, QuantumCircuit};

impl QuantumCircuit {
    /// Returns a Graphviz DOT string representing the decision diagram.
    pub fn to_graphviz_dot(&self) -> String {
        let mut dot = String::new();
        // Global graph settings.
        dot.push_str("digraph DD {\n");
        dot.push_str("  graph [rankdir=TB, splines=true, nodesep=0.6];\n");
        dot.push_str("  node [shape=none];\n");
        dot.push_str("  edge [arrowhead=normal];\n");
        dot.push_str("  root [shape=point, style=invis];\n");

        let mut visited: HashMap<usize, String> = HashMap::new();
        // Traverse starting from the root. This function returns Some(unique_id)
        // if the node is non-terminal (Terminal nodes are omitted).
        let root_id = traverse_dd(&self.root, &mut visited, &mut dot);
        if let Some(rid) = root_id {
            dot.push_str(&format!("  root -> {};\n", rid));
        }
        dot.push_str("}\n");
        dot
    }
}

/// Recursively traverses the decision diagram and emits DOT node and edge definitions.
/// - `visited`: maps pointer addresses to unique DOT ids.
/// - `dot`: accumulates DOT output.
/// Returns Some(unique_id) for nonterminal nodes, or None if the node is Terminal (which we omit).
fn traverse_dd(
    node: &NodePtr,
    visited: &mut HashMap<usize, String>,
    dot: &mut String,
) -> Option<String> {
    let key = Rc::as_ptr(node) as usize;
    if let Some(id) = visited.get(&key) {
        return Some(id.clone());
    }
    let node_ref = node.borrow();
    match &*node_ref {
        DdNode::Terminal(_) => {
            // Omit Terminal nodes entirely.
            None
        }
        DdNode::NonTerminal { qubit, zero, one } => {
            // Generate a unique id; here we use the pointer address in hex.
            let id = format!("node_{:x}", key);
            visited.insert(key, id.clone());
            if *qubit == usize::MAX {
                // This is a Sink node; rename it to "1" and represent it as a rectangle.
                dot.push_str(&format!(
                    "  {} [label=\"1\", shape=rectangle, style=\"rounded\", width=0.3, height=0.3];\n",
                    id
                ));
            } else {
                // Normal NonTerminal node: label by its qubit value using an HTML-like table.
                let label = format!(r#"<<TABLE BORDER="0" CELLBORDER="1" CELLSPACING="0" CELLPADDING="2">
  <TR><TD COLSPAN="2" ALIGN="CENTER">q<sub>{}</sub></TD></TR>
  <TR>
    <TD PORT="zero" ALIGN="CENTER"><FONT POINT-SIZE="12">0</FONT></TD>
    <TD PORT="one" ALIGN="CENTER"><FONT POINT-SIZE="12">1</FONT></TD>
  </TR>
</TABLE>>"#, qubit);
                dot.push_str(&format!("  {} [label={}];\n", id, label));

            }
            if let Some(child_id) = traverse_dd(&zero.1, visited, dot) {
                let (edge_color, edge_width) = edge_style(zero.0);
                let label_str = format_weight(zero.0).unwrap_or_default();
                dot.push_str(&format!(
                    "  {}:zero -> {} [label=<{}>, color=\"{}\", penwidth=\"{}\"];\n",
                    id, child_id, label_str, edge_color, edge_width
                ));
            }
            if let Some(child_id) = traverse_dd(&one.1, visited, dot) {
                let (edge_color, edge_width) = edge_style(one.0);
                let label_str = format_weight(one.0).unwrap_or_default();
                dot.push_str(&format!(
                    "  {}:one -> {} [label=<{}>, color=\"{}\", penwidth=\"{}\"];\n",
                    id, child_id, label_str, edge_color, edge_width
                ));
            }
            Some(id)
        }
    }
}
// Add a helper to set edge color by phase and width by amplitude:
fn edge_style(weight: Complex64) -> (String, f64) {
    let (r, theta) = weight.to_polar();
    let hue = ((theta % (2.0 * std::f64::consts::PI) + 2.0 * std::f64::consts::PI)
        % (2.0 * std::f64::consts::PI)) / (2.0 * std::f64::consts::PI);
    let color = format!("{:.3} 0.6 0.9", hue); // HSV-like
    let width = 0.8 + 1.2 * r;
    (color, width)
}


/// Formats a Complex64 weight for display.
/// Returns None if the weight is (approximately) 1.
fn format_weight(weight: Complex64) -> Option<String> {
    let tol = 1e-12;
    // Omit label if weight is 1+0i.
    // if (weight.re - 1.0).abs() < tol && weight.im.abs() < tol {
    //     return None;
    // }
    // Get polar representation: r and theta (in radians)
    let (r, theta) = weight.to_polar();
    // If magnitude ~ 1
    if (r - 1.0).abs() < tol {
        // If angle ~ 0, omit label
        if theta.abs() < tol {
            return None;
        }
        // Check special angles ±π/2 => ±i, ±π => -1, etc. Otherwise, e^(iθ)
        // (Use your existing special-angle logic or fallback.)
        // Example fallback:
        let angle_str = format_angle(theta, tol).unwrap_or(format!("{:.3}", theta));
        return Some(format!("<font point-size=\"8\">e(i{})</font>", angle_str));
    }
    // Format the magnitude: try a surd, then a rational, then decimals.
    let r_str = if let Some(s) = format_surds(r, tol) {
        s
    } else if let Some(rat) = approx_rational_str(r, tol) {
        rat
    } else {
        format!("{}", r)
    };

    // Check for special angles.
    let pi = std::f64::consts::PI;
    let angle_special = if theta.abs() < tol {
        Some("0".to_string())
    } else if (theta - (pi/2.0)).abs() < tol {
        Some("π/2".to_string())
    } else if (theta + (pi/2.0)).abs() < tol {
        Some("-π/2".to_string())
    } else if (theta - pi).abs() < tol {
        Some("π".to_string())
    } else if (theta + pi).abs() < tol {
        Some("-π".to_string())
    } else {
        None
    };

    // If a special angle is detected, format without the exponential.
    if let Some(special) = angle_special {
        if special == "0" {
            return Some(format!("<font point-size=\"8\">{}</font>", r_str));
        } else if special == "π/2" || special == "-π/2" {
            // Purely imaginary: if magnitude is 1, just output "i" or "-i".
            let sign = if special == "π/2" { "" } else { "-" };
            if r_str == "1" {
                Some(format!("<font point-size=\"8\">{}i</font>", sign));
            } else {
                return Some(format!("<font point-size=\"8\">{}{}i</font>", r_str, sign));
            }
        } else if special == "π" || special == "-π" {
            // Purely real and negative.
            let sign = if special == "π" { "-" } else { "-" }; // both cases negative.
            if r_str == "1" {
                Some(format!("<font point-size=\"8\">{}1</font>", sign));
            } else {
                return Some(format!("<font point-size=\"8\">{}{}</font>", sign, r_str));
            }
        }
    }

    // Otherwise, fallback to the exponential representation.
    let theta_str = if let Some(s) = format_angle(theta, tol) {
        s
    } else {
        format!("{:.3}", theta)
    };
    Some(format!("<font point-size=\"8\">{} e(i{})</font>", r_str, theta_str))
}


fn format_surds(x: f64, tol: f64) -> Option<String> {
    // Hard-code some common surd forms.
    if (x - (1.0 / 2.0_f64.sqrt())).abs() < tol {
        return Some("1/√2".to_string());
    }
    if (x - (2.0_f64.sqrt())).abs() < tol {
        return Some("√2".to_string());
    }
    // Add more common surds as needed.
    None
}

// Todo: instead of fraction, use the f64 value directly, maybe in scientific notation
fn approx_rational_str(x: f64, tol: f64) -> Option<String> {
    use num_rational::Ratio;
    use num_traits::ToPrimitive;
    let rat: Ratio<i64> = Ratio::approximate_float(x)?;
    if let Some(r_val) = rat.to_f64() {
        if (r_val - x).abs() < tol {
            return Some(format!("{:.3}/{:.3}", rat.numer(), rat.denom()));
        }
    }
    None
}

fn format_angle(theta: f64, tol: f64) -> Option<String> {
    let pi = std::f64::consts::PI;
    if theta.abs() < tol {
        return Some("0".to_string());
    }
    if (theta - (pi/4.0)).abs() < tol {
        return Some("π/4".to_string());
    }
    if (theta - (pi/2.0)).abs() < tol {
        return Some("π/2".to_string());
    }
    if (theta - pi).abs() < tol {
        return Some("π".to_string());
    }
    if (theta + (pi/4.0)).abs() < tol {
        return Some("-π/4".to_string());
    }
    if (theta + (pi/2.0)).abs() < tol {
        return Some("-π/2".to_string());
    }
    if (theta + pi).abs() < tol {
        return Some("-π".to_string());
    }
    None
}



