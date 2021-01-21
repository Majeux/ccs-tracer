use super::{next, Context, Operand, Status, Step, recurse_sub};
use crate::tree::{Action, Node};
use std::collections::HashMap;

/// Try to find a transition either in node_a or node_b, if none was found
/// this returns None
pub fn parallel(
    node_a: &Node,
    node_b: &Node,
    context: Context,
) -> Option<(Status, Box<Node>)> {
    use Node::*;

    let a = next(node_a, context.clone()).map(|(mut s, n)| {
        let step = Step::new("Parallel", Operand::Left, node_a.infix());
        s.push(step);
        let comp = Compose(n, Box::new(node_b.clone()));
        (s, Box::new(comp))
    });

    let b = || {
        next(node_b, context).map(|(mut s, n)| {
            let step = Step::new("Parallel", Operand::Right, node_b.infix());
            s.push(step);
            let comp = Compose(Box::new(node_a.clone()), n);
            (s, Box::new(comp))
        })
    };

    a.or_else(b)
}

/// Try to synchronize on a channel somewhere in the subtrees of node_a and node_b. Returns None
/// if no synchronization could be done.
pub fn composition(node_a: &Node, node_b: &Node) -> Option<(Status, Box<Node>)> {
    let (mut l_status, mut left_search_res) = rec_find_actions(node_a);
    let (mut r_status, mut right_search_res) = rec_find_actions(node_b);

    l_status.push(Step::new("", Operand::Left, node_a.infix()));
    r_status.push(Step::new("", Operand::Right, node_b.infix()));

    let mut status = l_status;
    status.append(&mut r_status);

    for (k, left_hand) in left_search_res.drain() {
        if let Some(right_hand) = right_search_res.remove(&k.bar()) {
            let comp = Node::Compose(left_hand, right_hand);
            let step = Step {
                operation: "Compose",
                operand: Operand::Sync(k.channel()),
                result: comp.infix(),
            };
            status.push(step);

            return Some((status, Box::new(comp)));
        }
    }

    None
}

/// Find actions (Prefix) to synchronize on. Returns search trace and HashMap of all actions and their resulting
/// subtree if taken.
fn rec_find_actions(state: &Node) -> (Status, HashMap<Action, Box<Node>>) {
    use Node::*;

    match state {
        Recurse(string, node) => {
            // recurse is ignored during synchronization
            let step = Step {
                operation: "Recurse",
                operand: Operand::None,
                result: "Compose does not apply recursions".into(),
            };
            // actions may be taken to bound variables -> replace
            let mut sub_node = *node.clone();
            recurse_sub(&mut sub_node, string, state);

            let (mut status, map) = rec_find_actions(&sub_node);
            status.push(step);

            (status, map)
        }
        Restrict(node, _action) => {
            //restrictions are ignored during composition/sync
            let step = Step {
                operation: "Restrict",
                operand: Operand::None,
                result: "Compose ignores restrictions".into(),
            };
            
            let (mut status, map) = rec_find_actions(node);
            status.push(step);

            (status, map)
        }
        Relabel(_node, _map) => panic!("Relabelling inside Synchronization not supported"),
        Compose(node_a, node_b) => {
            let mut merged = HashMap::new();

            let (mut l_status, mut left_reductions) = rec_find_actions(node_a);
            for (k, v) in left_reductions.drain() {
                let node = Compose(v, node_b.clone());
                merged.insert(k, Box::new(node));
            }
            let step = Step::new("Parallel", Operand::Left, node_a.infix());
            l_status.push(step);

            let (mut r_status, mut right_reductions) = rec_find_actions(node_b);
            for (k, v) in right_reductions.drain() {
                let node = Compose(node_a.clone(), v);
                merged.insert(k, Box::new(node));
            }
            let step = Step::new("Parallel", Operand::Right, node_b.infix());
            r_status.push(step);

            l_status.append(&mut r_status);
            (l_status, merged)
        }
        Choice(node_a, node_b) => {
            let mut merged = HashMap::new();

            let (mut l_status, mut left_reductions) = rec_find_actions(node_a);
            for (k, v) in left_reductions.drain() {
                merged.insert(k, v);
            }
            let step = Step::new("Choice", Operand::Left, node_a.infix());
            l_status.push(step);

            let (mut r_status, mut right_reductions) = rec_find_actions(node_b);
            for (k, v) in right_reductions.drain() {
                merged.insert(k, v);
            }
            let step = Step::new("Choice", Operand::Right, node_b.infix());
            r_status.push(step);

            l_status.append(&mut r_status);
            (l_status, merged)
        }
        Prefix(action, node) => {
            let mut reduction = HashMap::new();
            reduction.insert(action.clone(), node.clone());

            let step = Step::new("Action", Operand::Action(action.clone()), node.infix());
            let status = vec![step];

            (status, reduction)
        }
        Name(_string) => (Vec::new(), HashMap::new()),
        Nil => (Vec::new(), HashMap::new()),
    }
}
