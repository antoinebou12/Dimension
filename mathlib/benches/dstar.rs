//! Benchmarks for D* Lite (replan).

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{Graph, dstar_lite};

fn small_grid() -> Graph {
    let mut g = Graph::new(16);
    for i in 0..4 {
        for j in 0..4 {
            let u = i * 4 + j;
            if j + 1 < 4 {
                g.add_edge(u, i * 4 + (j + 1), 1.0);
            }
            if j > 0 {
                g.add_edge(u, i * 4 + (j - 1), 1.0);
            }
            if i + 1 < 4 {
                g.add_edge(u, (i + 1) * 4 + j, 1.0);
            }
            if i > 0 {
                g.add_edge(u, (i - 1) * 4 + j, 1.0);
            }
        }
    }
    g
}

pub fn bench_dstar(c: &mut Criterion) {
    let mut group = c.benchmark_group("dstar");
    group.bench_function("initial_plan", |b| {
        let mut g = small_grid();
        b.iter(|| black_box(dstar_lite(&mut g, 0, 15)));
    });
    group.bench_function("replan_after_update", |b| {
        let mut g = small_grid();
        if let Some(e) = g.out_edges[1].iter_mut().find(|(v, _)| *v == 2) {
            e.1 = 10.0;
        }
        b.iter(|| black_box(dstar_lite(&mut g, 0, 15)));
    });
    group.finish();
}

criterion_group!(benches, bench_dstar);
