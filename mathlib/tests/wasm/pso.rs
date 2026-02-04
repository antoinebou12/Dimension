//! Integration tests for PSO (demo: minimize sphere x²+y² on [-5,5]²).
//! The wasm binding pso_minimize requires a JS callback; here we test the same
//! scenario using the native mathlib::pso so the demo behavior is covered.

#![cfg(feature = "wasm")]

use mathlib::pso;

#[test]
fn wasm_demo_pso_sphere() {
    // Demo: minimize x²+y² on [-5,5]²; expect best position ~[0,0], best cost ~0
    let lower = vec![-5.0, -5.0];
    let upper = vec![5.0, 5.0];
    let cost = |x: &[f64]| x.iter().map(|&v| v * v).sum::<f64>();
    let result = pso((lower, upper), 20, cost, 100, None);
    let pos = &result.best_position;
    let c = result.best_cost;
    assert_eq!(pos.len(), 2);
    assert!(
        pos[0].abs() < 0.5 && pos[1].abs() < 0.5,
        "best position near origin, got {:?}",
        pos
    );
    assert!(c < 0.5, "best cost near 0, got {}", c);
}
