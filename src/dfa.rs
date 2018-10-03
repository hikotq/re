use nfa::{Nfa, StateSet};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs;
use std::io::{BufWriter, Write};
use std::mem;

#[derive(Clone)]
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
            let mut transitions: Vec<Option<StateSet>> = vec![None; 256];

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
                    if !subset_to_state.contains_key(&next) {
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

        for ac_state in self.states.iter().filter(|&s| s.accept) {
            ac_state_dot.push_str(&("s".to_owned() + &ac_state.id.to_string() + " "));
        }
        ac_state_dot.push_str(";\n");
        dot.push_str(&ac_state_dot);
        dot.push_str("node [shape = circle];\nempty -> s0 [label = \"start\"]\n");
        for s in self.states.iter() {
            for (label, t_state) in s.t.iter().enumerate() {
                if let Some(t_state) = t_state {
                    dot.push_str(&format!(
                        "s{} -> s{} [label = \"{}\"]\n",
                        s.id, t_state, label
                    ));
                }
            }
        }
        dot.push_str("}");
        dot
    }

    pub fn write(&self, file_name: &str) {
        let dot = self.dot();
        let mut f = BufWriter::new(fs::File::create(file_name).unwrap());
        f.write(dot.as_bytes()).ok();
    }
}

impl Dfa {
    pub fn minimize(&mut self) {
        let mut distinction_table = vec![Vec::new(); self.states.len()];
        for i in 0..self.states.len() - 1 {
            for j in ((i + 1)..self.states.len()).rev() {
                distinction_table[i].push(self.states[i].accept != self.states[j].accept);
            }
        }

        let mut distinction_flag = true;
        while distinction_flag {
            distinction_flag = false;
            for i in 0..(self.states.len() - 1) {
                for j in (i + 1)..self.states.len() {
                    if !distinction_table[i][self.states.len() - j - 1] {
                        for c in 0..=255 {
                            let mut n1 = self.states[i].t[c];
                            let mut n2 = self.states[j].t[c];
                            if n1 != n2 {
                                if n1 > n2 {
                                    mem::swap(&mut n1, &mut n2);
                                };
                                if n1.is_none() || n2.is_none() || distinction_table[n1.unwrap()]
                                    [self.states.len() - n2.unwrap() - 1]
                                {
                                    distinction_flag = true;
                                    distinction_table[i][self.states.len() - j - 1] = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut swap_map: HashMap<usize, usize> = HashMap::new();
        for i in 0..self.states.len() {
            for j in (i + 1)..self.states.len() {
                if !swap_map.contains_key(&j) {
                    if !distinction_table[i][self.states.len() - j - 1] {
                        swap_map.insert(j, i);
                    }
                }
            }
        }

        if swap_map.is_empty() {
            return;
        }

        let minimum_size = self.states.len() - swap_map.len();
        let mut replace_map = vec![0; self.states.len()];
        let mut d = 0;
        for s in 0..self.states.len() {
            if !swap_map.contains_key(&s) {
                replace_map[s] = d;
                d += 1;
                if s != replace_map[s] {
                    self.states[replace_map[s]] = self.states[s].clone();
                    self.states[replace_map[s]].id = replace_map[s];
                }
            } else {
                replace_map[s] = replace_map[swap_map[&s]];
            }
        }

        {
            let mut i = 0;

            while self.states[i].id < minimum_size {
                for c in 0..=255 {
                    if let Some(n) = self.states[i].t[c] {
                        self.states[i].t[c] = Some(replace_map[n]);
                    }
                }
                i += 1;
            }
        }
        drop(self.states.drain(minimum_size..));
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
fn regex_accept_dot() {
    let regex = "(a.*bc|bd)";
    let nfa = Nfa::re2nfa(regex);
    let dfa = Dfa::nfa2dfa(&nfa);
    let s = "bd";
    assert!(dfa.accept(s));
    let s = "abc";
    assert!(dfa.accept(s));
    let s = "adddbc";
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
