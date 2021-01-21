# ccs-parser

### Background
A process in CCS may be described by a Labeled Transition System (LTS), and is used to model processes that are potentially non-deterministic, non-terminating, or consist of multiple parallel components. Here each node is a Process, defined by the grammar in section 11.2 of Models of Computations (Roberto Bruni, Ugo Montanari). This process may be considered a state, from which an Action may be performed. An Action is performed using something called a Channel, modelling a component that can be queried for in- or output. Each transition is labelled with an Action and goes to a Process.

## Parser (src/parse.rs)
We implemented a parser to build a syntax tree from the grammar on 227 of the book. The parser was build using the Peg library[@rust:peg]. It produces a syntax tree encoded in a Rust Enum.

## Next (src/semantics.rs)
The "next" function takes a syntax tree/subtree and recursively derives a single transition that is possible from this state, if any. Depending on the type of expression it encounters in the syntax tree, it tries to perform a single rule from the operational semantics. If we reach a Prefix node, we know from the semantics that we can take some action and pass this information up into the recursion. If a transitions exists it returns a derivation trace, listing the rules used, and the resulting syntax tree.

## CCS (src/semantics.rs)
The "ccs" function takes a syntax tree, representing an initial state, and continuously calls the "next" function as long as it produces transitions. After the first iteration, it call next using the result from the previous call. This results in a single trace of transitions that are possible from the initial state until no transitions are available.

## Concessions
* CCS allows for multiple relabelling statements to be nested. By its semantics, this could allow for a transitive mapping between the nested statements. Our implementation still allows for multiple relabeling, but maintains a single universal mapping for the entire subtree. Whenever an additional relabelling is encountered, it is added to the existing one via a union for the following subtree. 
* Synchronization (src/semantics/compose.rs) is currently limited in regards to relabelling. Considering the mappings of channels provides currently complicates the search for opposing channels. Thus, synchronizing over relabelled statements is not supported to avoid faulty output.

## Program
### Installation
Install the rust version management tool (https://www.rust-lang.org/learn/get-started). Unzip our project cd into it. Set the rust version for the current directory to nightly: `rustup override set nightly-2021-01-20`.

### Running
Then build from source and run using `cargo r -- --help` in the root dir of our repository (the same dir as Cargo.toml). Our program will now print its help. You can pass arguments to it after `cargo r -- `, the -- tells the rust package management tool cargo to pass the arguments to our program. Alternatively after running cargo r once you can the binary directly, it is located in target/debug/ccs_tracer.

### Examples
We have demo script that goes through the examples in `example_input`. You can call it after `cargo r`.
