use crate::heuristic;
use crate::BipartiteGraph;
use crate::global_abort::GLOBAL_ABORT;
use std::sync::atomic::Ordering;
use rand::seq::SliceRandom;
use rand::Rng;

#[inline(always)]
fn get_crossings(g: &BipartiteGraph, cm: &mut Vec<Vec<u8>>, u: usize, v: usize) -> u64 {
    if cm[u][v] == 255 {
        let x = g.pair_crossing_number(u, v);
        if x < 255 {
            cm[u][v] = x as u8;
        }
        return x;
    }
    cm[u][v] as u64
}

#[inline(always)]
fn update_perm_pos(perm: &mut Vec<usize>, pos: &mut Vec<usize>, swap: usize) {
    perm.swap(swap, swap+1);
    pos[perm[swap]] = swap;
    pos[perm[swap+1]] = swap+1;
}

fn best_reinsert(g: &BipartiteGraph, perm: &Vec<usize>, cm: &mut Vec<Vec<u8>>, v: usize, up: usize, range: usize) -> (i64, usize) {
    let mut minval: i64 = i64::MAX; 
    let mut minidx: usize = 0;
    let mut acc: i64 = 0;
           
    let mut steps_since_min = 0;
    let num_steps = if up == 1 { perm.len() - v - 1 } else { v };
    for step in 0..num_steps {
        if GLOBAL_ABORT.load(Ordering::Relaxed) {
            return (minval, minidx);
        }
        let i = if up == 1 { v + step + 1 } else { v - step - 1 };
        steps_since_min += 1;
        if range < 2 && (steps_since_min > 500 || acc > 1000) {
            break;
        }
        if range == 2 && (steps_since_min > 5000 || acc > 10000) {
            break;
        }
        if up == 1 {
            acc += get_crossings(g, cm, perm[i], perm[v]) as i64 - get_crossings(g, cm, perm[v], perm[i]) as i64;
        } else {
            acc += get_crossings(g, cm, perm[v], perm[i]) as i64 - get_crossings(g, cm, perm[i], perm[v]) as i64;
        }
        if acc <= minval {
            minval = acc;
            minidx = i;
            steps_since_min = 0;
        }
    }
    (minval, minidx)
}

pub fn sifting_large(g: &BipartiteGraph) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut perm: Vec<usize> = heuristic::mean_heuristic(g); 
    let mut iter = 0;

    let mut pos = vec![0; perm.len()];
    for i in 0..perm.len() {
        pos[perm[i]] = i;
    }
    
    let mut cm: Vec<Vec<u8>> = vec![vec![255; perm.len()]; perm.len()];

    loop {
        let mut vertices: Vec<usize> = (0..perm.len()).collect();
        vertices.shuffle(&mut rng);
        for vert in vertices.iter().cloned() {
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                return perm;
            }
            let v = pos[vert];
            let range = iter % 3;
            let (minval_up, minidx_up) = best_reinsert(g, &perm, &mut cm, v, 1, range);
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                return perm;
            }
            let (minval_down, minidx_down) = best_reinsert(g, &perm, &mut cm, v, 0, range);
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                return perm;
            }

            let minval;
            let minidx;
            if minval_up < minval_down {
                minval = minval_up;
                minidx = minidx_up;
            } else if minval_up > minval_down {
                minval = minval_down;
                minidx = minidx_down;
            } else {
                minval = minval_up;
                if rng.gen_range(0..=1) == 0 {
                    minidx = minidx_up;
                } else {
                    minidx = minidx_down;
                }
            }

            if minval <= 0 {
                if minidx > v {
                    for i in v..minidx {
                        update_perm_pos(&mut perm, &mut pos, i);
                    }
                } else {
                    for i in (minidx..v).rev() {
                        update_perm_pos(&mut perm, &mut pos, i);
                    } 
                }
            }
        }
        iter += 1;
    }
}

