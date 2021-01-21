#![feature(exclusive_range_pattern)]

use std::fs;
use std::path::PathBuf;
extern crate peg;

mod parse;
mod semantics;
pub mod tree;
use parse::parse;
use simple_logger::SimpleLogger;

#[derive(structopt::StructOpt)]
#[structopt(verbatim_doc_comment)]
/// A parser and tracer for single line CCS programs
///
/// This parses and generates a trace of the operational semantics for a
/// a one line CCS program. You can use C-style comments (//) within the file.
///
/// This can parse CCS as presented in "models of computation"
/// by: Roberto Bruni and Ugo Montanari, with some modifications:
///
/// - For an output action prepend a character with a '!'
/// - For recursion use '_rec' instead of 'rec' and any small
///   latin character for the variable
/// - Use only small greek letters and latin capital letters for actions
///
/// example input:
/// (α.nil + β.nil) | (!α.nil + γ.nil)
///
struct Args {
    /// path to CCS source, if not provided the default example is ran
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// how verbose the output should be: Trace, Debug, Info, Warn or Error.
    /// the default is to print warnings and above.
    #[structopt(short, long)]
    verbosity: Option<log::LevelFilter>,
    /// print the syntax tree of the input.
    #[structopt(short, long)]
    print_tree: bool,
    /// do not generate the trace.
    #[structopt(short, long)]
    hide_trace: bool,
}

#[paw::main]
fn main(args: Args) {
    SimpleLogger::new()
        .with_level(args.verbosity.unwrap_or(log::LevelFilter::Warn))
        .init()
        .unwrap();

    let input = fs::read_to_string(args.input)
        .expect("could not open file");

    let tree = parse(&input);
    if args.print_tree {
        println!("syntax tree:\n{}", tree);
    }
    if !args.hide_trace {
        semantics::ccs(tree);
    }
}
