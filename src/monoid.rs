use dfa::Dfa;
use std::collections::{HashMap, VecDeque};

#[derive(Hash, Clone, Eq, PartialEq, Debug)]
pub struct TransitionPat {
    pat: Vec<usize>,
}

impl TransitionPat {
    fn new(dfa_size: usize) -> Self {
        TransitionPat {
            pat: vec![0; dfa_size + 1],
        }
    }

    fn identity(dfa_size: usize) -> Self {
        let mut post_transition_states = vec![0; dfa_size + 1];
        for i in 0..dfa_size + 1 {
            post_transition_states[i] = i;
        }
        TransitionPat {
            pat: post_transition_states,
        }
    }

    fn multiply(&self, other: &Self) -> Self {
        let out_pat = self.pat.iter().map(|s| other.pat[*s]).collect();
        TransitionPat { pat: out_pat }
    }
}

type Element = usize;

pub struct Monoid {
    multiply_table: Vec<Vec<usize>>,
    char_morphism: Vec<usize>,
}

impl Monoid {
    pub fn construct(dfa: &Dfa) -> Self {
        let mut queue = VecDeque::new();
        let ident = TransitionPat::identity(dfa.states.len());
        let mut transitions_map = HashMap::new();
        let mut char_morphism = Vec::new();
        let tmap_len = transitions_map.len();
        transitions_map.insert(ident.clone(), tmap_len);
        queue.push_front(ident.clone());

        while let Some(pat) = queue.pop_back() {
            //全文字探索エッグ
            for c in 0..=255 {
                let mut next = TransitionPat::new(dfa.states.len());
                for i in 0..dfa.states.len() + 1 {
                    next.pat[i] = if pat.pat[i] == dfa.states.len() {
                        dfa.states.len()
                    } else if let Some(next_state) = dfa.states[pat.pat[i]].t[c] {
                        next_state
                    } else {
                        dfa.states.len()
                    };
                }

                if transitions_map.contains_key(&next) {
                    let tmap_len = transitions_map.len();
                    transitions_map.insert(next.clone(), tmap_len);
                    if pat == ident {
                        char_morphism[c] = tmap_len;
                    }
                    queue.push_front(next);
                }
            }
        }

        let mut tmp_map: Vec<(TransitionPat, usize)> =
            transitions_map.clone().into_iter().collect();
        tmp_map.sort_by(|ref a, ref b| a.1.cmp(&b.1));
        let elements: Vec<TransitionPat> = tmp_map.into_iter().map(|(k, _)| k).collect();
        let mut multiply_table = vec![vec![0; transitions_map.len()]; transitions_map.len()];
        for i in 0..transitions_map.len() {
            multiply_table[0][i] = i;
            multiply_table[i][0] = i;
        }
        for i in 1..transitions_map.len() {
            for j in 1..transitions_map.len() {
                let calc_result = *transitions_map
                    .get(&elements[i].multiply(&elements[j]))
                    .unwrap();
                multiply_table[i][j] = calc_result;
            }
        }
        Monoid {
            multiply_table: multiply_table,
            char_morphism: char_morphism,
        }
    }

    fn multiply(&self, x: Element, y: Element) -> Element {
        self.multiply_table[x][y]
    }

    fn size(&self) -> usize {
        self.multiply_table.len()
    }

    pub fn is_aperiodic(&self) -> bool {
        for i in 0..self.size() {
            let mut e = i;
            for j in 0..self.size() {
                e = self.multiply(i, j)
            }
            if e != self.multiply(e, i) {
                return false;
            }
        }
        return true;
    }
}
