//! Tree traversal benchmarks: BFS, DFS.

mod bfs;
mod dfs;
mod helpers;

use criterion::criterion_group;

criterion_group! {
    name = benches;
    config = criterion::Criterion::default().warm_up_time(std::time::Duration::from_secs(1));
    targets = bfs::bench_bfs, dfs::bench_dfs
}
