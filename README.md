# sisyphus

This is a solver for the one-sided crossing minimization problem (OCM) as a submission to the PACE 2024 Heuristic Track. The input format and I/O behavior is as specified by the [PACE](https://pacechallenge.org/2024/).

The tool was developed by *Max Bannach, Florian Chudigiewitsch, Kim-Manuel Klein, Till Tantau* and *Marcel Wienöbst*.

# Algorithm

The solver first reduces the problem to a directed weighted feedback arc set (FAS) instance as, e.g., described in [1]. Each strongly connected component is linearly ordered by repeatedly starting a hill-climber based on the *sifting* strategy described in [2]. The best obtained solution is output. Because of the repeated hill-climber runs, the solver is named *sisyphus*. 

1. Alexander Dobler: *[A Note on the Complexity of One-Sided Crossing Minimization of Trees](https://arxiv.org/abs/2306.15339).* (Technical Report, 2023)
2. Christian Matuszewski, Robby Schönfeld, and Paul Molitor: Using Sifting for k-Layer Straightline Crossing Minimization. Graph Drawing: 7th International Symposium (1999). 

# Dependencies
The following open source [crates](https://crates.io) are used. They are automatically downloaded and compiled when the solver is build using *Cargo*. 
- [signal-hook](https://crates.io/crates/signal-hook)
- [rand](https://crates.io/crates/rand)

# Build
sisyphus is implemented in [Rust](https://www.rust-lang.org) and can simply be build using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```
cargo build --release
```

# Run
After the build is completed, the tool can either be executed directly via

```
./target/release/sisyphus < <instance.gr>
```

or by using Cargo

```
cargo run --release < <instance.gr>
```
