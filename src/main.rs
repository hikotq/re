mod nfa;
mod dfa;
extern crate regparser;
use nfa::Nfa;
use dfa::Dfa;

pub fn main() {
    let regex = "ab*";
    let nfa = Nfa::re2nfa(&regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    dfa.write("dot.dot");
}
