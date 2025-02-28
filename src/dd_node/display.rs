use std::fmt;
use std::fmt::{Display, Formatter};
use crate::constants::ZERO;
use crate::DdNode;

impl Default for DdNode {
    fn default() -> Self {
        DdNode::Terminal(ZERO)
    }
}
impl Display for DdNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn fmt_node(node: &DdNode, f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
            let pad = "  ".repeat(indent);
            match node {
                DdNode::Terminal(c) => write!(f, "{}Terminal({})", pad, c),
                DdNode::NonTerminal { qubit, zero, one } => {
                    writeln!(f, "{}NonTerminal {{", pad)?;
                    writeln!(f, "{}  qubit: {},", pad, qubit)?;
                    writeln!(f, "{}  zero: (", pad)?;
                    writeln!(f, "{}      {},", pad, zero.0)?;
                    fmt_node(&*zero.1.borrow(), f, indent + 2)?;
                    writeln!(f, "\n{}  ),", pad)?;
                    writeln!(f, "{}  one: (", pad)?;
                    writeln!(f, "{}      {},", pad, one.0)?;
                    fmt_node(&*one.1.borrow(), f, indent + 2)?;
                    writeln!(f, "\n{}  )", pad)?;
                    write!(f, "{}}}", pad)
                }
            }
        }
        fmt_node(self, f, 0)
    }
}
