// This crate contains pure graph functionality.
// Graphs are represented as Vec<Vec<usize>>. 

/// Constructs subgraph of g induced by subset. In the resulting graph vertex i corresponds to
/// vertex subset[i] in the original graph.
pub fn get_subgraph(g: &Vec<Vec<usize>>, subset: &Vec<usize>) -> Vec<Vec<usize>> {
    let mut imp: Vec<i32> = vec![-1; g.len()]; 
    for i in 0..subset.len() {
        imp[subset[i]] = i as i32;
    }
    let mut h: Vec<Vec<usize>> = vec![Vec::new(); subset.len()];
    for i in 0..subset.len() {
        for v in g[subset[i]].iter().cloned() {
            let newv = imp[v];
            if newv != -1 {
                h[i].push(newv as usize);
            }
        }
    }
    h
}

fn top_ordering_dfs(g: &Vec<Vec<usize>>, vis: &mut Vec<bool>, ord: &mut Vec<usize>, u: usize) {
    if vis[u] {
        return;
    } 
    vis[u] = true;
    for v in g[u].iter().cloned() {
        top_ordering_dfs(g, vis, ord, v);
    }
    ord.push(u);
}

/// Returns topological ordering of directed acyclic graph g. 
pub fn top_ordering(g: &Vec<Vec<usize>>) -> Vec<usize> {
    let mut vis = vec![false; g.len()];
    let mut ord: Vec<usize> = Vec::new();
    for u in 0..g.len() {
        if !vis[u] {
            top_ordering_dfs(g, &mut vis, &mut ord, u);
        }
    }
    ord.reverse();
    ord
}

fn propagate_label(g: &Vec<Vec<usize>>, vertex_labels: &mut Vec<usize>, u: usize, label: usize) {
    if vertex_labels[u] != 0 {
        return;
    }
    vertex_labels[u] = label;
    for v in &g[u] {
        propagate_label(g, vertex_labels, *v, label);
    }
}

/// Returns list of strongly connected components. 
pub fn compute_sccs(h: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    // compute top ordering
    let ord = top_ordering(h);
    // get reverse graph
    let mut hrev: Vec<Vec<usize>> = vec![Vec::new(); h.len()];
    for u in 0..h.len() {
        for v in &h[u] {
            hrev[*v].push(u);
        }
    }
    let mut label: usize = 1;
    let mut vertex_labels = vec![0; h.len()];
    for u in ord.into_iter() {
        if vertex_labels[u] == 0 {
            propagate_label(&hrev, &mut vertex_labels, u, label);
            label += 1;
        }
    }
    let mut scc: Vec<Vec<usize>> = vec![Vec::new(); label-1];
    for u in 0..h.len() {
        scc[vertex_labels[u]-1].push(u);    
    }
    scc
}
