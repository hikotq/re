extern crate regparser;
pub mod converter;
pub mod dfa;
pub mod monoid;
pub mod nfa;

use dfa::Dfa;
use monoid::{Monoid, TransitionPat};
use nfa::Nfa;

fn main() {
    let regex = "(a|ba)*";
    let nfa = Nfa::re2nfa(regex);
    let mut dfa = Dfa::nfa2dfa(&nfa);
    dfa.minimize();
    dfa.write("dfa.dot");
    Monoid::construct(&dfa);
}
