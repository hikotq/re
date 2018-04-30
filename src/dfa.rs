use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use nfa::{Nfa, StateSet};

#[derive(Debug)]
pub struct Dfa {
    pub transitions: HashMap<usize, HashMap<char, usize>>,
    init_state: usize,
    pub ac_state_set: StateSet,
    states_num: usize,
}

impl Dfa {
    pub fn nfa2dfa(nfa: &Nfa) -> Self {
        Dfa::construct(nfa)
    }

    fn construct(nfa: &Nfa) -> Self {
        let mut dfa = Dfa {
            transitions: HashMap::new(),
            init_state: 0,
            ac_state_set: StateSet(HashSet::new()),
            states_num: 0,
        };
        let mut nfa_init_state_set = StateSet(HashSet::new());
        nfa_init_state_set.insert(0);
        nfa_init_state_set = StateSet(nfa.epsilon_expand(nfa_init_state_set));
        if nfa_init_state_set.contains(&0) {
            dfa.ac_state_set.insert(0);
        }
        let mut done = HashSet::new();
        let mut state_set_to_id = HashMap::new();
        let nfa_ac_states = {
            let mut ac_state_set = HashSet::new();
            for ac_state in nfa.states.iter().filter(|&state| state.accept) {
                ac_state_set.insert(ac_state.id);
            }
            StateSet(ac_state_set)
        };
        dfa.construct_recursive(
            nfa,
            nfa_init_state_set,
            &mut done,
            &mut state_set_to_id,
            &nfa_ac_states,
        );
        dfa
    }

    fn construct_recursive(
        &mut self,
        nfa: &Nfa,
        state_set: StateSet,
        done: &mut HashSet<StateSet>,
        state_set_to_id: &mut HashMap<StateSet, usize>,
        nfa_ac_states: &StateSet,
    ) {
        if !state_set_to_id.contains_key(&state_set) {
            state_set_to_id.insert(state_set.clone(), self.states_num);
            self.states_num += 1;
        }
        let mut subset_transitions = HashMap::new();
        let mut transitions = HashMap::new();
        for byte in (0 as u8)..=255 {
            let c = byte as char;
            let mut t = HashSet::new();
            for id in state_set.clone().0.iter() {
                if let Some(state_set) = nfa.states[*id].transitions.get(&c) {
                    t = t.union(state_set).cloned().collect();
                }
            }
            let t: HashSet<usize> = t.union(&nfa.epsilon_expand(StateSet(t.clone())))
                .cloned()
                .collect();
            if !t.is_empty() {
                state_set_to_id
                    .entry(StateSet(t.clone()))
                    .or_insert(self.states_num);
                let id = *state_set_to_id.get(&StateSet(t.clone())).unwrap();
                if id == self.states_num {
                    if t.intersection(&nfa_ac_states.0).count() != 0 {
                        self.ac_state_set.insert(self.states_num);
                    }
                    self.states_num += 1;
                }
                transitions.insert(c, id);
                subset_transitions.insert(c, StateSet(t));
            }
        }
        {
            let subset_tval = subset_transitions.values().clone();
            for next in subset_tval {
                if !done.contains(next) {
                    done.insert(next.clone());
                    self.construct_recursive(
                        nfa,
                        next.clone(),
                        done,
                        state_set_to_id,
                        nfa_ac_states,
                    );
                }
            }
        }
        self.transitions
            .insert(*state_set_to_id.get(&state_set).unwrap(), transitions);
    }

    pub fn accept(&self, s: &str) -> bool {
        let mut state = 0;
        for c in s.to_string().chars() {
            if let Some(&next_state) = self.transitions.get(&state).unwrap().get(&c) {
                state = next_state;
            } else {
                return false;
            }
        }
        if self.ac_state_set.contains(&state) {
            true
        } else {
            false
        }
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
    let s = "";
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
