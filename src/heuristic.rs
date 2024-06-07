use crate::BipartiteGraph;
use crate::sifting;
use crate::sifting_large;
use crate::sifting_very_large;
use crate::global_abort::GLOBAL_ABORT;
use signal_hook::{iterator::Signals, consts::signal::*};
use std::{thread,sync::atomic::Ordering};

pub fn start(g: &BipartiteGraph) -> Vec<usize> {
    let mut ordering: Vec<usize> = Vec::new(); 
    for u in g.isolated.iter().cloned() {
        ordering.push(u);
    }
    let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap(); // SIGINT might be removed here, but helps to test via CTRL-C.
    thread::spawn(move || {
        for _sig in signals.forever() {
            GLOBAL_ABORT.store(true, Ordering::Relaxed);
        }
    });
    // encode matrix/graph more efficiently (u8) and allow for larger graphs
    if g.n1 < 10_000 {
        let sccs = g.reduce();
        let res = sifting::sifting_heuristic(g, &sccs);
        for v in res.iter().cloned() {
            for twin in g.ids[v].iter().cloned() {
                ordering.push(twin);
            }
        }
    } else if g.n1 < 75_000 {
        let res = sifting_large::sifting_large(g);
        for v in res.iter().cloned() {
            for twin in g.ids[v].iter().cloned() {
                ordering.push(twin);
            }
        }
    } else {
        let res = sifting_very_large::sifting_very_large(g);
        for v in res.iter().cloned() {
            for twin in g.ids[v].iter().cloned() {
                ordering.push(twin);
            }
        }
    }
    for el in &mut ordering {
        *el += g.n0 + 1;
    }
    ordering
}

// heuristics below are used in as subroutine, not as a standalone heuristic solver
pub fn mean_heuristic(g: &BipartiteGraph) -> Vec<usize> {
    let mut ordering: Vec<usize> = (0..g.n1).collect(); 
    let mut means: Vec<f64> = Vec::new();
    for u in 0..g.n1 {
        if g.adjs[u].is_empty() {
            means.push(0.0); 
        } else {
            let mean: f64 = (g.adjs[u].iter().sum::<usize>() as f64) / (g.adjs[u].len() as f64);
            means.push(mean); 
        }
    }
    ordering.sort_by(|a, b| means[*a].partial_cmp(&means[*b]).unwrap());
    ordering
}

