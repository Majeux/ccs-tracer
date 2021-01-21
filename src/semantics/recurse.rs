use crate::tree::Node;

/// recursively walk through the tree root swapping any name node
/// by the subtree in 'to'.
pub fn recurse_sub(root: &mut Node, from: &str, to: &Node) {
    use Node::*;

    match root {
        Recurse(_string, ref mut node) => {
            recurse_sub(&mut *node, from, to);
        }
        Restrict(ref mut node, _action) => {
            recurse_sub(&mut *node, from, to);
        }
        Relabel(ref mut node, _map) => {
            recurse_sub(&mut *node, from, to);
        }
        Compose(ref mut node_a, ref mut node_b) => {
            recurse_sub(&mut *node_a, from, to);
            recurse_sub(&mut *node_b, from, to);
        }
        Choice(ref mut node_a, ref mut node_b) => {
            recurse_sub(&mut *node_a, from, to);
            recurse_sub(&mut *node_b, from, to);
        }
        Prefix(_action, ref mut node) => {
            recurse_sub(&mut *node, from, to);
        }
        Name(string) => {
            if from == string.as_str() {
                *root = to.clone();
            }
        }
        Nil => {}
    }
}
