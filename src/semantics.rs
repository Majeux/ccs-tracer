use crate::tree::{Action, Node};
use derive_new::new;
use std::collections::{HashMap, HashSet};

mod compose;
use compose::{composition, parallel};
mod recurse;
pub use recurse::recurse_sub;

/// Information regarding the operation, for example for Choice this
/// indicates weather the left or right child was taken
pub enum Operand {
    Action(Action),
    Left,
    Right,
    /// Synchronizing on channel name
    Sync(String),
    Bound(String),
    None,
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use Action::*;
        match self {
            Operand::Action(action) => match action {
                In(s) => write!(f, "In({})", s),
                Out(s) => write!(f, "Out({})", s),
            },
            Operand::Left => write!(f, "Left"),
            Operand::Right => write!(f, "Right"),
            Operand::Sync(name) => write!(f, "Sync on {}", name),
            Operand::Bound(bound) => write!(f, "{}", bound),
            Operand::None => Ok(()),
        }
    }
}

/// A single step in a transition
#[derive(new)]
pub struct Step {
    /// The operation such as: Compose, Choice, Prefix etc
    operation: &'static str,
    /// Information regarding the operation, for example for Choice this
    /// indicates weather the left or right child was taken
    operand: Operand,
    /// Result after operation with operand on previous step
    result: String,
}

/// The status of the current transition as a list
/// of ['Step']s
type Status = Vec<Step>;

/// Perform transitions on the CCS tree until no more can be made.
/// Prints a trace of transitions, and a trace of derivations for each transition
pub fn ccs(root: Node) {
    let mut prev_state = root;
    let mut transitions = 0;

    let mut visited = HashSet::new();
    visited.insert(prev_state.clone());

    while let Some((status, next_state)) = next(&prev_state, Context::default()) {
        println!("##################################################################");
        println!("Trans: {} -> {}", prev_state.infix(), next_state.infix());

        let mut tab = "";
        let mut i = 0;
        transitions += 1;

        for step in status.into_iter().rev() {
            if step.operation.is_empty() {
                println!("---\n--- {}: {} ---\n", step.operand, step.result);
                tab = "\t";
                i = 0;
                continue;
            }
            println!(
                "{}{} | {}: {} \t-> {}",
                tab, i, step.operation, step.operand, step.result
            );

            i += 1;
        }

        println!("#");
        // If we find a duplicate in visited, we entered a cycle due to recursion
        let duplicate = !visited.insert(*next_state.clone());

        if duplicate {
            println!("Cycle found: terminating");
            break;
        }

        prev_state = *next_state;
    }

    println!("CCS-process terminated in {} transition(s)", transitions);
}

/// The context keeps track of modifications that are applied to the tree. In the prefix nodes
/// we take the context into account to pick the right action.
#[derive(Clone, Default)]
pub struct Context {
    restrict: HashSet<String>,
    relabel: HashMap<String, String>,
}

/// Uses CCS's operational semantics to derive a single transition from some state.
/// Returns Some trace in Status and the resulting state in Box<Node> if possible, or else None
pub fn next(state: &Node, mut context: Context) -> Option<(Status, Box<Node>)> {
    use Node::*;

    match state {
        Recurse(bound, node) => {
            // assume all recursions are guarded: every occurence of "bound" is in a prefix
            // current statement is bound to the "bound" variable
            // context.subs.insert(bound.clone(), state.clone());

            next(node, context).map(|(mut status, mut node)| {
                recurse_sub(&mut *node, &bound, state);
                let step = Step::new("Recurse on", Operand::Bound(bound.to_owned()), node.infix());
                status.push(step);
                (status, node)
            })
        }
        Restrict(node, action) => {
            // add action to a set of restricted channels, used in Prefix case
            context.restrict.insert(action.channel());
            next(node, context)
                .map(|(status, node)| (status, Box::new(Restrict(node, action.to_owned()))))
        }
        Relabel(node, map) => {
            // extend current relabelling with the one found in map
            let map_items = map.iter().map(|(key, value)| (key.into(), value.into()));
            context.relabel.extend(map_items);

            next(node, context).map(|(status, node)| (status, Box::new(Relabel(node, map.clone()))))
        }
        Compose(node_a, node_b) => {
            // Choose to move in first side with available action (returning Some)
            // and return the composition updated with the result
            let comp = composition(node_a, node_b);
            let par = || parallel(node_a, node_b, context);

            comp.or_else(par)
        }
        Choice(node_a, node_b) => {
            // Choose to move in first side with available action (returning Some)
            // and return the result

            let a = next(node_a, context.clone()).map(|(mut s, n)| {
                let step = Step::new("Choice", Operand::Left, node_a.infix());
                s.push(step);
                (s, n)
            });

            let b = || {
                next(node_b, context).map(|(mut s, n)| {
                    let step = Step::new("Choice", Operand::Right, node_b.infix());
                    s.push(step);
                    (s, n)
                })
            };

            a.or_else(b)
        }
        Prefix(action, node) => {
            // Perform the found action and return the resulting node
            // Change action if a relabelling exists
            let mut channel = &action.channel();
            let mut action = action.clone();

            if let Some(new_channel) = context.relabel.get(channel) {
                channel = new_channel;
                action = action.with_new_channel(channel);
            }

            if context.restrict.contains(channel) {
                return None;
            }

            let step = Step::new("Action", Operand::Action(action), node.infix());
            let status = vec![step];

            Some((status, node.clone()))
        }
        Name(_string) => None,
        Nil => None,
    }
}
