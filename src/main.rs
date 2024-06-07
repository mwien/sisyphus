use sisyphus::bipartite_graph::BipartiteGraph;
use sisyphus::heuristic;

fn main() {
    // Solve the problem using the given strategy.
    let g  = BipartiteGraph::new_from_stdin().expect("c Failed to read the graph!");    
    let res = heuristic::start(&g);
    for u in &res { println!("{}", u); }    
}
