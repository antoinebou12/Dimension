//! Integration tests that run the same logic as each example (excluding ply/png file-writing).
//! Run: `cargo test -p mathlib` (15 tests). With features: `-F genetic` or `-F wasm` for the rest.

use mathlib::cg::vector3;
use mathlib::easing::{bspline, ease_in_out_cubic, ease_out_bounce, hermite, lerp, linear};
use mathlib::{
    AStarResult, DijkstraResult, GradientDescentOptions, Graph, LineSearchOptions, Matrix, NOISE,
    Storage, Vector, articulation_points, astar, bridges, connected_components, dijkstra,
    gradient_descent, greedy_vertex_coloring, reverse_graph,
};
use mathlib::{
    Perspective3, SvmOptions, armijo, backtracking, dbscan, from_euler_angles, kmeans, look_at_rh,
    matrix4f_inverse, model_view_projection, new_perspective, new_translation, pca, pso, solve,
    svd_econ, svm, svm_rbf,
};
use mathlib::{PsoOptions, Quat4f};

fn path_from_prev(prev: &[Option<usize>], start: usize, end: usize) -> Vec<usize> {
    let mut path = vec![end];
    let mut u = end;
    while let Some(p) = prev[u] {
        path.push(p);
        if p == start {
            break;
        }
        u = p;
    }
    path.reverse();
    path
}

#[test]
fn example_argmin() {
    let x0 = vec![5.0_f64, 5.0];
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let gradient = |x: &[f64], g: &mut [f64]| {
        g[0] = 2.0 * x[0];
        g[1] = 2.0 * x[1];
    };
    let opts = GradientDescentOptions {
        max_iters: 500,
        tol: 1e-9,
        ..Default::default()
    };
    let result = gradient_descent(&x0, cost, gradient, &opts);
    assert!(
        result.cost < 1e-6,
        "cost should be near 0, got {}",
        result.cost
    );
    assert!(result.iterations > 0, "should perform iterations");
}

#[test]
#[cfg(feature = "genetic")]
fn example_cmaes() {
    use mathlib::{CmaEsBuilder, CmaEsResult};

    fn sphere(x: &[f64]) -> f64 {
        x.iter().map(|&v| v * v).sum()
    }
    let dim = 6;
    let mean = vec![1.0; dim];
    let mut opt = CmaEsBuilder::new(dim, mean, 0.3)
        .max_generations(150)
        .seed(42)
        .build();
    let result: CmaEsResult = opt.optimize(sphere);
    assert!(
        result.fitness < 1.0,
        "sphere should converge, got fitness {}",
        result.fitness
    );
    assert_eq!(result.solution.len(), dim);
    assert!(result.generations > 0);
}

#[test]
fn example_dbscan_3d() {
    let mut data = Matrix::with_storage(8, 3, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(0, 2, 0.0);
    data.set(1, 0, 0.1);
    data.set(1, 1, 0.0);
    data.set(1, 2, 0.0);
    data.set(2, 0, 0.0);
    data.set(2, 1, 0.1);
    data.set(2, 2, 0.0);
    data.set(3, 0, 5.0);
    data.set(3, 1, 5.0);
    data.set(3, 2, 5.0);
    data.set(4, 0, 5.1);
    data.set(4, 1, 5.0);
    data.set(4, 2, 5.0);
    data.set(5, 0, 5.0);
    data.set(5, 1, 5.1);
    data.set(5, 2, 5.0);
    data.set(6, 0, 100.0);
    data.set(6, 1, 100.0);
    data.set(6, 2, 100.0);
    data.set(7, 0, 200.0);
    data.set(7, 1, 200.0);
    data.set(7, 2, 200.0);

    let eps = 1.0;
    let min_pts = 2;
    let result = dbscan(&data, eps, min_pts);

    let noise_count = result.labels().iter().filter(|&&l| l == NOISE).count();
    assert_eq!(result.labels().len(), 8);
    assert!(result.n_clusters() >= 1);
    assert_eq!(
        noise_count, 2,
        "two noise points at (100,100,100) and (200,200,200)"
    );
}

#[test]
fn example_easing() {
    let ts = [0.0_f64, 0.25, 0.5, 0.75, 1.0];
    for &t in &ts {
        let l = linear(t);
        let c = ease_in_out_cubic(t);
        let b = ease_out_bounce(t);
        assert!(l >= 0.0 && l <= 1.0);
        assert!(c >= 0.0 && c <= 1.0);
        assert!(b >= 0.0 && b <= 1.0);
    }
    for &t in &ts {
        let v = lerp(0.0, 10.0, t);
        assert!(v >= 0.0 && v <= 10.0);
    }
    for &t in &ts {
        let h = hermite(0.0, 1.0, 0.0, 0.0, t);
        assert!(h >= 0.0 && h <= 1.0);
    }
    let pts = [0.0_f64, 1.0, 2.0, 3.0];
    for &t in &ts {
        let _ = bspline(&pts, t);
    }
    let axis = vector3(0.0_f32, 1.0, 0.0);
    let q0 = Quat4f::from_axis_angle(&axis, 0.0);
    let q1 = Quat4f::from_axis_angle(&axis, std::f32::consts::FRAC_PI_2);
    let q_mid = q0.slerp(&q1, 0.5);
    assert!(
        q_mid.w.is_finite() && q_mid.x.is_finite() && q_mid.y.is_finite() && q_mid.z.is_finite()
    );
}

#[test]
fn example_graph() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 4.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(1, 3, 6.0);
    g.add_edge(2, 3, 1.0);
    g.add_edge(2, 4, 3.0);
    g.add_edge(3, 4, 1.0);

    assert_eq!(g.num_nodes(), 5);
    assert_eq!(g.num_edges(), 7);
    assert!(g.is_adjacent(0, 1));

    let rev = reverse_graph(&g);
    assert!(rev.is_adjacent(2, 1));

    let dres: DijkstraResult = dijkstra(&g, 0);
    let path_to_4 = path_from_prev(&dres.prev, 0, 4);
    assert!(!path_to_4.is_empty());
    assert_eq!(path_to_4[0], 0);
    assert_eq!(path_to_4[path_to_4.len() - 1], 4);

    let ares: AStarResult = astar(&g, 0, 4, |_, _| 0.0);
    assert!(!ares.path.is_empty());
    assert_eq!(ares.path[0], 0);
    assert_eq!(ares.path[ares.path.len() - 1], 4);

    let mut undir = Graph::new(4);
    undir.add_edge_undirected(0, 1, 1.0);
    undir.add_edge_undirected(1, 2, 1.0);
    undir.add_edge_undirected(2, 3, 1.0);

    let components = connected_components(&undir);
    assert_eq!(components.len(), 1, "path 0-1-2-3 is one component");
    assert_eq!(components[0].len(), 4);
    let ap = articulation_points(&undir);
    assert!(!ap.is_empty());
    let br = bridges(&undir);
    assert_eq!(br.len(), 3);
    let colors = greedy_vertex_coloring(&undir);
    assert_eq!(colors.len(), 4);
}

#[test]
fn example_kmeans() {
    let mut data = Matrix::with_storage(6, 2, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(1, 0, 1.0);
    data.set(1, 1, 0.0);
    data.set(2, 0, 2.0);
    data.set(2, 1, 0.0);
    data.set(3, 0, 10.0);
    data.set(3, 1, 10.0);
    data.set(4, 0, 11.0);
    data.set(4, 1, 10.0);
    data.set(5, 0, 12.0);
    data.set(5, 1, 10.0);

    let k = 2;
    let result = kmeans(&data, k, Some(50));

    assert_eq!(result.n_clusters(), k);
    assert_eq!(result.labels().len(), 6);
    assert_eq!(result.centroids().rows(), k);
    assert_eq!(result.centroids().cols(), 2);
}

#[test]
fn example_linesearch() {
    let x = [1.0_f64, 0.0];
    let d = [-1.0_f64, 0.0];
    let f = 1.0;
    let g_dot_d = -2.0;
    let cost = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
    let opts = LineSearchOptions::default();
    let mut scratch = [0.0_f64; 2];

    let alpha = backtracking(&x, &d, f, g_dot_d, cost, &opts, &mut scratch);
    assert!(alpha > 0.0, "backtracking alpha should be positive");
    assert!(cost(&scratch) <= f + 1e-6);

    let mut scratch2 = [0.0_f64; 2];
    let alpha_armijo = armijo(&x, &d, f, g_dot_d, cost, &opts, &mut scratch2);
    assert!(alpha_armijo > 0.0);
}

#[test]
fn example_matrix() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(0, 2, 3.0);
    a.set(1, 0, 4.0);
    a.set(1, 1, 5.0);
    a.set(1, 2, 6.0);
    a.set(2, 0, 7.0);
    a.set(2, 1, 8.0);
    a.set(2, 2, 9.0);

    let at = a.transpose();
    assert_eq!(at.rows(), 3);
    assert_eq!(at.cols(), 3);
    assert!((at.get(0, 0) - 1.0_f64).abs() < 1e-9);
    assert!((at.get(1, 0) - 2.0_f64).abs() < 1e-9);

    let c = &a * &at;
    assert_eq!(c.rows(), 3);
    assert_eq!(c.cols(), 3);
    assert!((c.get(0, 0) - (1.0_f64 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0)).abs() < 1e-6);
}

#[test]
fn example_mvp_rotation_inverse_transpose() {
    let t = vector3(1.0, 0.0, 0.0);
    let model_t = new_translation(&t);
    let model_r = from_euler_angles(0.0, 0.1, 0.2);
    let model = &model_t * &model_r;

    let eye = vector3(0.0, 0.0, 5.0);
    let target = vector3(0.0, 0.0, 0.0);
    let up = vector3(0.0, 1.0, 0.0);
    let view = look_at_rh(&eye, &target, &up);

    let aspect = 16.0 / 9.0;
    let fov_y = std::f32::consts::FRAC_PI_4;
    let near = 0.1;
    let far = 100.0;
    let projection = new_perspective(aspect, fov_y, near, far);

    let mvp = model_view_projection(&model, &view, &projection);
    assert_eq!(mvp.rows(), 4);
    assert_eq!(mvp.cols(), 4);

    let rot = from_euler_angles(0.1, 0.2, 0.3);
    let rot_transpose = rot.transpose();
    let rot_inv = matrix4f_inverse(&rot);
    let rt_times_r = &rot_transpose * &rot;
    assert!((rt_times_r.get(0, 0) - 1.0).abs() < 1e-4);
    assert!((rt_times_r.get(1, 1) - 1.0).abs() < 1e-4);
    assert!((rot_inv.get(0, 0) - rot_transpose.get(0, 0)).abs() < 1e-5);

    let view_inv = matrix4f_inverse(&view);
    let view_times_inv = &view * &view_inv;
    assert!((view_times_inv.get(0, 0) - 1.0).abs() < 1e-4);
    assert!((view_times_inv.get(3, 3) - 1.0).abs() < 1e-4);

    let proj = Perspective3::new(aspect, fov_y, near, far);
    let proj_inv = proj.inverse_matrix();
    let proj_mat = proj.as_matrix();
    let _proj_times_inv = &proj_mat * &proj_inv;
    assert_eq!(proj_mat.rows(), 4);
    assert_eq!(proj_inv.rows(), 4);
}

#[test]
fn example_pca() {
    let mut data = Matrix::with_storage(10, 4, Storage::Column);
    for i in 0..10 {
        for j in 0..4 {
            data.set(i, j, (i as f64) * 0.5 + (j as f64));
        }
    }

    let result = pca(&data, Some(2));
    let mean = result.mean();
    let ev = result.explained_variance();

    assert_eq!(result.n_components(), 2);
    assert_eq!(mean.rows(), 4);
    assert_eq!(ev.rows(), 2);
    assert!(ev.get(0) >= 0.0 && ev.get(1) >= 0.0);
}

#[test]
fn example_pso() {
    fn sphere_cost(x: &[f64]) -> f64 {
        x.iter().map(|v| v * v).sum()
    }
    let dim = 4usize;
    let low = vec![-5.0; dim];
    let high = vec![5.0; dim];
    let result = pso(
        (low, high),
        20,
        sphere_cost,
        100,
        Some(PsoOptions::default()),
    );

    assert!(
        result.best_cost < 1.0,
        "PSO sphere should improve, got {}",
        result.best_cost
    );
    assert_eq!(result.best_position.len(), dim);
    assert!(result.iterations > 0);
}

#[test]
fn example_solve() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 2.0);

    let mut b = Vector::with_capacity(2);
    b.set(0, 3.0);
    b.set(1, 3.0);

    let x = solve(&a, &b).expect("solve should succeed");
    let ax0 = a.get(0, 0) * x.get(0) + a.get(0, 1) * x.get(1);
    let ax1 = a.get(1, 0) * x.get(0) + a.get(1, 1) * x.get(1);
    assert!((ax0 - 3.0).abs() < 1e-9);
    assert!((ax1 - 3.0).abs() < 1e-9);
}

#[test]
fn example_svd() {
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(1, 0, 0.0);
    a.set(2, 0, 0.0);
    a.set(0, 1, 0.0);
    a.set(1, 1, 2.0);
    a.set(2, 1, 0.0);

    let econ = svd_econ(&a);
    let u = econ.u();
    let sigma = econ.sigma();
    let v = econ.v();

    assert_eq!(u.rows(), 3);
    assert_eq!(u.cols(), 2);
    assert_eq!(sigma.rows(), 2);
    assert_eq!(v.rows(), 2);
    assert_eq!(v.cols(), 2);
    assert!(sigma.get(0) >= sigma.get(1));
}

#[test]
fn example_svm() {
    let mut x = Matrix::with_storage(6, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 2.0);
    x.set(1, 0, 2.0);
    x.set(1, 1, 3.0);
    x.set(2, 0, 2.0);
    x.set(2, 1, 2.0);
    x.set(3, 0, 0.0);
    x.set(3, 1, 0.0);
    x.set(4, 0, 1.0);
    x.set(4, 1, 0.0);
    x.set(5, 0, 0.0);
    x.set(5, 1, 1.0);

    let y = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0];

    let opts = SvmOptions {
        c: 10.0,
        max_iters: 10_000,
        tol: 1e-3,
    };
    let result = svm(&x, &y, Some(opts)).expect("svm fit");
    let pred = result.predict(&x);
    assert_eq!(pred.len(), 6);
}

#[test]
fn example_svm_rbf() {
    let mut x = Matrix::with_storage(8, 2, Storage::Column);
    x.set(0, 0, 0.0);
    x.set(0, 1, 0.0);
    x.set(1, 0, 1.0);
    x.set(1, 1, 1.0);
    x.set(2, 0, 0.1);
    x.set(2, 1, 0.1);
    x.set(3, 0, 0.9);
    x.set(3, 1, 0.9);
    x.set(4, 0, 1.0);
    x.set(4, 1, 0.0);
    x.set(5, 0, 0.0);
    x.set(5, 1, 1.0);
    x.set(6, 0, 0.9);
    x.set(6, 1, 0.1);
    x.set(7, 0, 0.1);
    x.set(7, 1, 0.9);

    let y = [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0];

    let gamma = 2.0;
    let opts = SvmOptions {
        c: 10.0,
        max_iters: 10_000,
        tol: 1e-3,
    };
    let result = svm_rbf(&x, &y, gamma, Some(opts)).expect("svm_rbf fit");
    assert!(result.n_support_vectors() > 0);
    let pred = result.predict(&x);
    assert_eq!(pred.len(), 8);
}

#[test]
fn example_vector() {
    let mut u = Vector::with_capacity(3);
    u.set(0, 1.0);
    u.set(1, 2.0);
    u.set(2, 3.0);

    let mut v = Vector::with_capacity(3);
    v.set(0, 4.0);
    v.set(1, 5.0);
    v.set(2, 6.0);

    let dot = u.dot(&v);
    assert!((dot - 32.0_f64).abs() < 1e-9);
    let norm_u = u.norm();
    let norm_v = v.norm();
    assert!((norm_u - 14.0_f64.sqrt()).abs() < 1e-9);
    assert!((norm_v - 77.0_f64.sqrt()).abs() < 1e-9);
}

#[test]
#[cfg(feature = "wasm")]
fn example_wasm() {
    use mathlib::wasm::{WasmCg, WasmMatrix, WasmMatrix32, WasmVector};

    let mut m = WasmMatrix::new(2, 2);
    m.set(0, 0, 1.0);
    m.set(0, 1, 2.0);
    m.set(1, 0, 3.0);
    m.set(1, 1, 4.0);
    let arr = m.to_array();
    assert_eq!(arr.len(), 4);

    let v = WasmVector::from_array(&[1.0, 2.0]);
    let mv = m.mul_vector(&v).expect("mul_vector");
    assert_eq!(mv.to_array().len(), 2);

    let b = WasmVector::from_array(&[5.0, 11.0]);
    let x = m.solve(&b).expect("solve");
    assert_eq!(x.to_array().len(), 2);

    let svd = m.svd_econ();
    assert_eq!(svd.get_sigma().to_array().len(), 2);

    let a = WasmVector::from_array(&[0.0, 0.0]);
    let b_vec = WasmVector::from_array(&[3.0, 4.0]);
    let mid = a.lerp(&b_vec, 0.5).expect("lerp");
    assert_eq!(mid.to_array().len(), 2);
    let _dist = a.euclidean_distance(&b_vec).expect("euclidean_distance");

    let rot = WasmMatrix32::rotation(0.0, 0.0, std::f32::consts::FRAC_PI_2);
    let pt = rot.transform_point(1.0, 0.0, 0.0).expect("transform_point");
    assert_eq!(pt.len(), 3);

    let view = WasmCg::look_at_rh(0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    let _proj = WasmCg::new_perspective(16.0 / 9.0, std::f32::consts::FRAC_PI_4, 0.1, 100.0);
    let _view_inv = view.inverse().expect("view inverse");
}
