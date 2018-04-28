mod nfa;
mod dfa;
extern crate regparser;
use std::mem;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::sync::Mutex;
use std::io::Write;
use std::str;
use regparser::parser::{Lexer, Parser};
use nfa::Nfa;
use dfa::Dfa;
#[macro_use]
extern crate lazy_static;

pub fn main() {
    let regex = "ab*";
    let nfa = nfa::Nfa::re2nfa(&regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    dfa.write("dot.dot");
}
