use nfa::{Nfa, StateSet};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::io::{BufWriter, Write};

pub struct State {
    pub t: [Option<usize>; 256],
    pub id: usize,
    pub accept: bool,
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = "State { t: [".to_string();
        for i in 0..256 {
            let s = &match self.t[i] {
                Some(n) => format!("{}: {}, ", i, n),
                _ => "".to_string(),
            };
            output += s;
        }
        output += "], ";
        output += &format!("id: {}, accept: {}", self.id, self.accept);
        write!(f, "({})", output)
    }
}

impl State {
    fn new(id: usize, accept: bool) -> Self {
        State {
            t: [None; 256],
            id: id,
            accept: accept,
        }
    }
}

#[derive(Debug)]
pub struct Dfa {
    pub states: Vec<State>,
    state_num: usize,
}

impl Dfa {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            state_num: 0,
        }
    }

    pub fn nfa2dfa(nfa: &Nfa) -> Self {
        Dfa::construct(nfa)
    }

    fn construct(nfa: &Nfa) -> Self {
        let mut dfa = Dfa::new();
        let mut state_num = 0;
        let mut queue: VecDeque<StateSet> = VecDeque::new();
        let mut subset_to_state: HashMap<StateSet, usize> = HashMap::new();
        queue.push_back(nfa.start_states());
        subset_to_state.insert(nfa.start_states(), state_num);
        state_num += 1;

        while !queue.is_empty() {
            let mut accept = false;
            let subset: StateSet = queue.pop_front().unwrap();
            let mut transitions: Vec<Option<StateSet>> = Vec::with_capacity(256);
            for _ in 0..256 {
                transitions.push(None);
            }

            for iter in subset.iter() {
                accept |= nfa.states[*iter as usize].accept;
                for c in 0..256 {
                    if let Some(nfa_t) = nfa.t(*iter as usize, c as u8) {
                        transitions[c] = Some(nfa_t);
                    }
                }
            }
            let state = dfa.new_state(accept);

            for c in 0..256 {
                if let Some(ref next) = transitions[c] {
                    if !subset_to_state.contains_key(next) {
                        subset_to_state.insert(next.clone(), state_num);
                        state_num += 1;
                        queue.push_back(next.clone());
                    }
                    state.t[c] = Some(*subset_to_state.get(next).unwrap());
                }
            }
        }
        dfa.state_num = state_num;
        dfa
    }

    fn new_state(&mut self, accept: bool) -> &mut State {
        let id = self.state_num;
        self.states.push(State::new(id, accept));
        self.state_num += 1;
        &mut self.states[id]
    }

    pub fn accept(&self, s: &str) -> bool {
        let mut state = &self.states[0];
        for c in s.to_string().chars() {
            if let Some(next) = state.t[c as usize] {
                state = &self.states[next];
            } else {
                return false;
            }
        }
        true
    }

    pub fn dot(&self) -> String {
        let mut dot = r###"
            digraph G {
            rankdir=LR;
            empty [label = "" shape = plaintext];
        "###
            .to_owned();
        let mut ac_state_dot = "\nnode [shape = doublecircle]".to_owned();

        for ac_state in self.ac_state_set.iter() {
            ac_state_dot.push_str(&("s".to_owned() + &ac_state.to_string() + " "));
        }
        ac_state_dot.push_str(";\n");
        dot.push_str(&ac_state_dot);
        dot.push_str("node [shape = circle];\nempty -> s0 [label = \"start\"]\n");
        for (state, transitions) in self.transitions.iter() {
            for (label, t_state) in transitions.iter() {
                dot.push_str(&format!(
                    "s{} -> s{} [label = \"{}\"]\n",
                    state, t_state, label
                ));
            }
        }
        dot.push_str("}");
        dot
    }

    pub fn write(&self, file_name: &str) {
        let dot = self.dot();
        let mut f = BufWriter::new(fs::File::create(file_name).unwrap());
        f.write(dot.as_bytes()).unwrap();
    }
}

#[test]
fn regex_accept_char() {
    let regex = "a";
    let s = "a";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    assert!(dfa.accept(s);)
}

#[test]
fn regex_accept_union() {
    let regex = "a|b";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "a";
    assert!(dfa.accept(s));
    let s = "b";
    assert!(dfa.accept(s));
}

#[test]
fn regex_accept_concat() {
    let regex = "ab";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "ab";
    assert!(dfa.accept(s));
}

#[test]
fn regex_accept_star() {
    let regex = "a*";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "";
    assert!(dfa.accept(s));
    let s = "a";
    assert!(dfa.accept(s));
    let s = "aaaaaaaaaaaaaaaa";
    assert!(dfa.accept(s));
}

#[test]
fn regex_accept_01() {
    let regex = "(a|b)c";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "ac";
    assert!(dfa.accept(s));
    let s = "bc";
    assert!(dfa.accept(s));
}

#[test]
fn regex_accept_02() {
    let regex = "a*c";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "c";
    assert!(dfa.accept(s));
    let s = "ac";
    assert!(dfa.accept(s));
    let s = "aaaaac";
    assert!(dfa.accept(s));
}

#[test]
fn regex_accept_03() {
    let regex = "(a|c)*";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "";
    assert!(dfa.accept(s));
    let s = "accccc";
    assert!(dfa.accept(s));
    let s = "aaaaac";
    assert!(dfa.accept(s));
    let s = "aaaaaaaaaaaaaaaa";
    assert!(dfa.accept(s));
}
