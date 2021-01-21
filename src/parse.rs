use crate::tree::{Action, Map, Node};

#[doc(hidden)]
peg::parser!( grammar ccs() for str {
    rule _() = [' ']*

    rule nil() -> Node
        = "nil" { Node::Nil }

    rule name() -> Node
        = name:$(['a'..='z']+) { Node::Name(name.to_owned()) }

    rule var() -> String
        = c:$(['a'..='z']) { c.to_owned() }

    rule action() -> Action
        = in_action() / out_action()

    rule in_action() -> Action
        = a:$(['A'..='Z' | 'α'..='ω']) { Action::In(a.to_owned()) }

    rule out_action() -> Action
        = "!" a:$(['A'..='Z' | 'α'..='ω']) { Action::Out(a.to_owned()) }

    rule substitute() -> (String, String)
        = value:action() "/" key:action() { (key.channel(), value.channel()) }

    rule mapping() -> Map
        = head:substitute() tail:("," _ a:substitute() {a})* {
            let mut tail = tail; //needed because macro does not allow mut declaration
            let mut map = Map::new();
            map.insert(head.0, head.1);
            map.extend(tail.drain(..));
            map
    }

    // Precedence climbing parser, top operators have precedence over lower
    pub(crate) rule process() -> Node
        = precedence!{
            // Atomics
            p:nil() { p }
            p:name() { p }

            // Braces
            "(" p:process() ")" { p }
            --
            // Composition / Choice
            p1:(@) "|" p2:@ { Node::Compose(Box::new(p1), Box::new(p2)) }
            p1:(@) "+" p2:@ { Node::Choice(Box::new(p1), Box::new(p2)) }
            --
            // Restriction
            p:(@) "\\" a:action() { Node::Restrict(Box::new(p), a) }
            --
            // Relabeling
            p:(@) "[" m:mapping() "]" { Node::Relabel(Box::new(p), m) }
            --
            // Recursion
            "_rec " x:var() "." p:(@) { Node::Recurse(x, Box::new(p)) }
            --
            // Prefix
            a:action() "." p:(@) { Node::Prefix(a, Box::new(p)) }
        }

});

fn fix_whitespace(input: &str) -> String {
    let input = input.replace(" ", "");
    input.replace("rec", "rec ")
}

/// given a string of valid ccs returns a syntax tree. This will
/// panic if the sting is not valid ccs.
pub fn parse(input: &str) -> Node {
    let mut input = input
        .lines()
        .filter(|s| !s.is_empty())
        .filter(|s| !s.trim_start().starts_with("//"));
    let clean_input = input
        .next()
        .map(fix_whitespace)
        .expect("input has got to have at least one non empty non commented line");
    if input.count() > 0 {
        log::warn!("only reading first non empty non comment line");
    }

    log::info!("parsing input: {}", clean_input);
    ccs::process(&clean_input).unwrap()
}
