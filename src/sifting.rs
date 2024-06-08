use crate::scc::SCC;
use crate::BipartiteGraph;
use crate::graph;
use crate::global_abort::GLOBAL_ABORT;
use std::time::Instant;
use std::sync::atomic::Ordering;
use rand::thread_rng;
use rand::seq::SliceRandom;
use rand::Rng;

fn get_inv_w(sccs: &Vec<SCC>) -> Vec<Vec<Vec<u64>>> {
    let mut inv_w: Vec<Vec<Vec<u64>>> = Vec::new();
    for scc in sccs {
        let mut scc_inv_w = vec![vec![0; scc.n]; scc.n];
        for j in 0..scc.n {
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                return Vec::new();
            }
            for k in 0..scc.n {
                scc_inv_w[j][k] = scc.w[k][j];
            }
        }
        inv_w.push(scc_inv_w);
    }
    inv_w 
}

pub fn eval_ordering_scc(perm: &Vec<usize>, scc: &SCC) -> u64 {
    let mut res = 0;
    for i in 0..scc.n {
        if GLOBAL_ABORT.load(Ordering::Relaxed) {
            return u64::MAX-1;
        }
        for j in (i+1)..scc.n {
            res += scc.w[perm[j]][perm[i]];
        }
    }
    res
}

fn map_to_original_labels(perm: &Vec<Vec<usize>>, sccs: &Vec<SCC>) -> Vec<usize> {
    let mut ordering = Vec::new();
    for i in 0..sccs.len() {
        for &p in perm[i].iter() {
            ordering.push(sccs[i].labels[p]);
        }
    }
    ordering
}

fn insert_cost_per_pos(scc: &SCC, diffs: &Vec<Vec<u64>>, perm: &Vec<usize>, v: usize) -> Vec<u64> {
    let n = perm.len();
    let mut pre: Vec<u64> = vec![0; n+1];
    let mut suf: Vec<u64> = vec![0; n+1];
    for i in 0..n {
        pre[i+1] = pre[i] + scc.w[v][perm[i]];
    }
    let mut perm_rev = perm.clone();
    perm_rev.reverse();
    for i in 0..n {
        suf[i+1] = suf[i] + diffs[v][perm_rev[i]];
    }
    suf.reverse();
    pre.iter().zip(&suf).map(|(p, s)| p + s).collect()
}

fn get_min(val: &Vec<u64>) -> (u64, Vec<usize>) {
    let minval = *val.iter().min().unwrap();
    let minima = val.iter()
        .enumerate()
        .filter(|(_, &x)| x == minval)
        .map(|(idx, _)| idx)
        .collect();
    (minval, minima)
}

pub fn insertion_sifting(sccs: &Vec<SCC>) -> Vec<Vec<usize>> {
    let inv_w = get_inv_w(sccs); // could precompute this
    let mut ordering: Vec<Vec<usize>> = Vec::new();
    let mut rng = thread_rng();
    for i in 0..sccs.len() {
        let scc = &sccs[i];
        let mut scc_ordering: Vec<usize> = Vec::new();
        let mut vertices: Vec<usize> = (0..scc.n).collect();
        vertices.shuffle(&mut rng);
        for v in vertices.iter().cloned() {
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                scc_ordering.push(v);
                continue;
            }
            let cost = insert_cost_per_pos(scc, &inv_w[i], &scc_ordering, v);
            let (_, minima) = get_min(&cost);
            scc_ordering.insert(*minima.choose(&mut rng).unwrap(), v);
        }
        ordering.push(scc_ordering);
    }
    ordering
}

pub fn insertionplus_sifting(sccs: &Vec<SCC>) -> Vec<Vec<usize>> {
    let inv_w = get_inv_w(sccs); // could precompute this
    let mut ordering: Vec<Vec<usize>> = Vec::new();
    let mut rng = thread_rng();
    for i in 0..sccs.len() {
        let scc = &sccs[i];
        let mut scc_ordering: Vec<usize> = Vec::new();
        let mut vertices: Vec<usize> = (0..scc.n).collect();
        vertices.shuffle(&mut rng);
        for v in vertices.iter().cloned() {
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                scc_ordering.push(v);
                continue;
            }
            let cost = insert_cost_per_pos(scc, &inv_w[i], &mut scc_ordering, v);
            let (_, minima) = get_min(&cost);
            scc_ordering.insert(*minima.choose(&mut rng).unwrap(), v);
            if scc_ordering.len() % 50 == 0 {
                // put into function
                let mut iter = 0;
                let mut last_improvement = 0;
                while iter - last_improvement < 2*scc_ordering.len() { 
                    if GLOBAL_ABORT.load(Ordering::Relaxed) {
                        break;
                    }
                    let vpos = rng.gen_range(0..scc_ordering.len());
                    let v = scc_ordering[vpos];
                    scc_ordering.remove(vpos); 
                    let cost = insert_cost_per_pos(scc, &inv_w[i], &scc_ordering, v);
                    let (mincost, minima) = get_min(&cost);
                    let previous_cost = cost[vpos]; 
                    let delta = previous_cost - mincost;
                    if delta > 0 {
                        last_improvement = iter;
                    }
                    let mut inspos;
                    loop {
                        inspos = *minima.choose(&mut rng).unwrap();
                        if inspos !=  vpos || minima.len() == 1 {
                            break;
                        }
                    }
                    scc_ordering.insert(inspos, v);
                    iter += 1;
                }
                
            }
        }
        ordering.push(scc_ordering);
    }
    ordering
    
}

pub fn hillclimber_sifting(sccs: &Vec<SCC>, initial_ordering: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let inv_w = get_inv_w(sccs); // could precompute this
    let mut ordering = initial_ordering;
    let mut rng = thread_rng();
    let mut iter = 0;
    let mut last_improvement = 0;
    while iter - last_improvement < 4 { 
        for i in 0..sccs.len() {
            let scc = &sccs[i];
            if scc.n == 1 { continue; }
            let scc_ordering = &mut ordering[i];            
            // do shuffles or just take random elements?
            let mut vertices: Vec<usize> = (0..scc.n).collect();
            vertices.shuffle(&mut rng);
            for v in vertices.iter().cloned() {
                if GLOBAL_ABORT.load(Ordering::Relaxed) {
                    return ordering;
                }
                // for now always remove element first, later optimize this
                let vpos = scc_ordering.iter().position(|&x| x == v).unwrap();
                scc_ordering.remove(vpos); 
                let cost = insert_cost_per_pos(scc, &inv_w[i], scc_ordering, v);
                let (mincost, minima) = get_min(&cost);
                let previous_cost = cost[vpos]; 
                let delta = previous_cost - mincost;
                if delta > 0 {
                    last_improvement = iter;
                }
                let mut inspos;
                loop {
                    inspos = *minima.choose(&mut rng).unwrap();
                    if inspos !=  vpos || minima.len() == 1 {
                        break;
                    }
                }
                scc_ordering.insert(inspos, v);
            }
        }
        iter += 1;
    }
    ordering
}

pub fn sifting_heuristic(_g: &BipartiteGraph, sccs: &Vec<SCC>) -> Vec<usize> {
    let start = Instant::now();
    // PART 1: run heuristic a few times on original instance
    // initialize frequency count
    let mut freqs_per_scc = Vec::new();
    for scc in sccs.iter() {
        freqs_per_scc.push(vec![vec![0; scc.n]; scc.n]);
    }

    let mut first_bestval_perscc = vec![u64::MAX; sccs.len()];
    let mut first_bestperm_perscc = vec![Vec::new(); sccs.len()]; 
    let mut cntruns = 0;
    while start.elapsed().as_secs_f64() <= 60.0 { 
        let perm = hillclimber_sifting(sccs, insertion_sifting(sccs));
        if GLOBAL_ABORT.load(Ordering::Relaxed) {
            if first_bestval_perscc[0] == u64::MAX {
                return map_to_original_labels(&perm, sccs);
                
            } else {
                return map_to_original_labels(&first_bestperm_perscc, sccs);
            }
        }
        for i in 0..sccs.len() {
            let scc_eval = eval_ordering_scc(&perm[i], &sccs[i]);
            if scc_eval < first_bestval_perscc[i] {
                first_bestval_perscc[i] = scc_eval;
                first_bestperm_perscc[i] = perm[i].clone();
            }
        }
        for i in 0..sccs.len() {
            for j in 0..perm[i].len() {
                if GLOBAL_ABORT.load(Ordering::Relaxed) {
                    return map_to_original_labels(&first_bestperm_perscc, sccs);
                }
                for k in (j+1)..perm[i].len() {
                    freqs_per_scc[i][perm[i][j]][perm[i][k]] += 1;
                }
            }
        }
        cntruns += 1;
    }

    if cntruns < 10 { // should rarely happen
        while !GLOBAL_ABORT.load(Ordering::Relaxed) { 
            let perm = hillclimber_sifting(sccs, insertion_sifting(sccs));
            if GLOBAL_ABORT.load(Ordering::Relaxed) {
                if first_bestval_perscc[0] == u64::MAX {
                    return map_to_original_labels(&perm, sccs);
                    
                } else {
                    return map_to_original_labels(&first_bestperm_perscc, sccs);
                }
            }
            for i in 0..sccs.len() {
                let scc_eval = eval_ordering_scc(&perm[i], &sccs[i]);
                if scc_eval < first_bestval_perscc[i] {
                    first_bestval_perscc[i] = scc_eval;
                    first_bestperm_perscc[i] = perm[i].clone();
                }
            }
        }
    } 
    let mut bestval_perscc = Vec::new(); 
    let mut bestperm_perscc = Vec::new(); 
    let mut newsccs: Vec<SCC> = Vec::new();
    // PART 2: reduce edges which always incur costs and recompute sccs
    // remove edges -> maybe have lower bound on number of iterations
    // this can never run at timeout 5 min -> no need to insert breaks/returns
    for i in 0..sccs.len() {
        let scc = &sccs[i];
        let mut h = vec![Vec::new(); scc.n];
        for j in 0..scc.n {
            for k in scc.g[j].iter().cloned() {
                if freqs_per_scc[i][j][k] > cntruns / 30 {
                    h[j].push(k);
                }
            }
        }

        let mut invbestperm = vec![0; scc.n];
        for j in 0..scc.n {
            invbestperm[first_bestperm_perscc[i][j]] = j;
        }

        let hsccs = graph::compute_sccs(&h);
        for hscc in hsccs.iter() {
            let mut w: Vec<Vec<u64>> = vec![vec![0; hscc.len()]; hscc.len()];
            let mut g: Vec<Vec<usize>> = vec![Vec::new(); hscc.len()];
            for j in 0..hscc.len() {
                for k in 0..hscc.len() {
                    w[j][k] = scc.w[hscc[j]][hscc[k]];
                    if w[j][k] != 0 {
                        g[j].push(k);
                    }
                }
            }
            let mut labels: Vec<usize> = Vec::new();
            for hl in hscc.iter().cloned() {
                labels.push(scc.labels[hl]);
            }
            newsccs.push(SCC::new(labels, w, g)); 

            let mut bestnewperm: Vec<usize> = (0..hscc.len()).collect();
            bestnewperm.sort_by_key(|&i| invbestperm[hscc[i]]);
            bestval_perscc.push(eval_ordering_scc(&bestnewperm, newsccs.last().unwrap()));
            bestperm_perscc.push(bestnewperm);
        }
    }

    while !GLOBAL_ABORT.load(Ordering::Relaxed) {
        let perm = hillclimber_sifting(&newsccs, insertionplus_sifting(&newsccs));
        if GLOBAL_ABORT.load(Ordering::Relaxed) {
            break;
        }
        for i in 0..newsccs.len() {
            let scc_eval = eval_ordering_scc(&perm[i], &newsccs[i]);
            if scc_eval < bestval_perscc[i] {
                bestval_perscc[i] = scc_eval;
                bestperm_perscc[i] = perm[i].clone();
            }
        }
    }
    map_to_original_labels(&bestperm_perscc, &newsccs)
}
