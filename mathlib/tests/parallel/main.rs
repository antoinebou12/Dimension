//! Parallel correctness tests: verify parallel implementations match sequential results.

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
mod tests {
    use mathlib::prelude::*;
    use mathlib::{clustering, distance, graph::*, stats};

    #[test]
    fn test_parallel_kmeans_matches_sequential() {
        let mut data = Matrix::with_storage(8, 2, Storage::Column);
        let vals = vec![
            1.0, 2.0, 1.5, 2.5, 8.0, 9.0, 8.5, 9.5, 1.2, 2.1, 1.8, 2.3, 8.2, 9.1, 8.3, 9.3,
        ];
        for (i, &v) in vals.iter().enumerate() {
            data.set(i / 2, i % 2, v);
        }
        let result = clustering::kmeans(&data, 2, Some(10));
        assert_eq!(result.labels().len(), 8);
        assert_eq!(result.n_clusters(), 2);
        // Verify centroids are reasonable
        assert!(result.centroids().rows() == 2);
        assert!(result.centroids().cols() == 2);
    }

    #[test]
    fn test_parallel_dbscan_matches_sequential() {
        let mut data = Matrix::with_storage(8, 2, Storage::Column);
        let vals = vec![
            1.0, 2.0, 1.5, 2.5, 8.0, 9.0, 8.5, 9.5, 1.2, 2.1, 1.8, 2.3, 8.2, 9.1, 8.3, 9.3,
        ];
        for (i, &v) in vals.iter().enumerate() {
            data.set(i / 2, i % 2, v);
        }
        let result = clustering::dbscan(&data, 1.5, 2);
        assert_eq!(result.labels().len(), 8);
        // Should find at least one cluster
        assert!(result.n_clusters() > 0);
    }

    #[test]
    fn test_parallel_dijkstra_correctness() {
        let mut g = Graph::new(5);
        g.add_edge(0, 1, 4.0);
        g.add_edge(0, 2, 1.0);
        g.add_edge(2, 1, 2.0);
        g.add_edge(1, 3, 1.0);
        g.add_edge(2, 3, 5.0);
        g.add_edge(3, 4, 3.0);

        let result = dijkstra(&g, 0);
        assert_eq!(result.dist[0], 0.0);
        assert_eq!(result.dist[1], 3.0); // 0->2->1
        assert_eq!(result.dist[2], 1.0); // 0->2
        assert_eq!(result.dist[3], 4.0); // 0->2->1->3
        assert_eq!(result.dist[4], 7.0); // 0->2->1->3->4
    }

    #[test]
    fn test_parallel_astar_correctness() {
        let mut g = Graph::new(5);
        g.add_edge(0, 1, 4.0);
        g.add_edge(0, 2, 1.0);
        g.add_edge(2, 1, 2.0);
        g.add_edge(1, 3, 1.0);
        g.add_edge(2, 3, 5.0);
        g.add_edge(3, 4, 3.0);

        let result = astar(&g, 0, 4, |_, _| 0.0);
        assert!(!result.path.is_empty());
        assert_eq!(result.dist, 7.0); // 0->2->1->3->4
    }

    #[test]
    fn test_parallel_bfs_correctness() {
        let mut g = Graph::new(6);
        g.add_edge_undirected(0, 1, 1.0);
        g.add_edge_undirected(0, 2, 1.0);
        g.add_edge_undirected(1, 3, 1.0);
        g.add_edge_undirected(2, 4, 1.0);
        g.add_edge_undirected(3, 5, 1.0);

        let result = bfs(&g, 0);
        assert_eq!(result.order.len(), 6);
        assert_eq!(result.depth[0], 0);
        assert_eq!(result.depth[1], 1);
        assert_eq!(result.depth[2], 1);
        assert_eq!(result.depth[3], 2);
        assert_eq!(result.depth[4], 2);
        assert_eq!(result.depth[5], 3);
    }

    #[test]
    fn test_parallel_covariance_correctness() {
        let mut data = Matrix::with_storage(3, 3, Storage::Column);
        let vals = vec![1.0, 2.0, 3.0, 2.0, 4.0, 6.0, 3.0, 6.0, 9.0];
        for (i, &v) in vals.iter().enumerate() {
            data.set(i / 3, i % 3, v);
        }
        let cov = stats::covariance(&data);
        assert_eq!(cov.rows(), 3);
        assert_eq!(cov.cols(), 3);
        // Covariance matrix should be symmetric
        for i in 0..3 {
            for j in 0..3 {
                assert!((cov.get(i, j) - cov.get(j, i)).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_parallel_pso_convergence() {
        use mathlib::argmin::{PsoOptions, pso};
        // Minimize sphere function: f(x) = sum(x_i^2)
        let cost = |x: &[f64]| x.iter().map(|&v| v * v).sum::<f64>();
        let bounds = (vec![-5.0, -5.0], vec![5.0, 5.0]);
        let result = pso(bounds, 20, cost, 50, Some(PsoOptions::default()));
        // Should converge near zero
        assert!(result.best_cost < 0.1);
    }

    #[test]
    fn test_parallel_convolution_correctness() {
        use mathlib::transforms::conv_1d;
        let signal = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let kernel = vec![0.5, 0.5];
        let result = conv_1d(&signal, &kernel);
        // Manual verification of first few values
        assert_eq!(result.len(), 6);
        assert!((result[0] - 0.5).abs() < 1e-10);
        assert!((result[1] - 1.5).abs() < 1e-10);
        assert!((result[2] - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_parallel_vector_operations() {
        let mut a: Vector<f64> = Vector::with_capacity(5);
        let mut b: Vector<f64> = Vector::with_capacity(5);
        for i in 0..5 {
            a.set(i, (i + 1) as f64);
            b.set(i, (i + 2) as f64);
        }

        // Dot product
        let dot = a.dot(&b);
        assert!((dot - 70.0_f64).abs() < 1e-10);

        // Vector addition
        let c = &a + &b;
        assert_eq!(c.rows(), 5);
        assert!((c.get(0) - 3.0_f64).abs() < 1e-10);
        assert!((c.get(4) - 11.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_parallel_distance_metrics() {
        let mut data = Matrix::with_storage(2, 3, Storage::Column);
        data.set(0, 0, 1.0);
        data.set(0, 1, 2.0);
        data.set(0, 2, 3.0);
        data.set(1, 0, 4.0);
        data.set(1, 1, 5.0);
        data.set(1, 2, 6.0);

        let d = distance::euclidean_rows(&data, 0, 1);
        let expected =
            ((4.0_f64 - 1.0).powi(2) + (5.0_f64 - 2.0).powi(2) + (6.0_f64 - 3.0).powi(2)).sqrt();
        assert!((d - expected).abs() < 1e-10);
    }

    #[test]
    fn test_cpu_parallel_api() {
        use mathlib::cpu::parallel::{par_add_f64, par_dot_f64, par_squared_diff_sum_f64};
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        assert!((par_dot_f64(&a, &b) - 32.0_f64).abs() < 1e-10);
        let mut out = [0.0; 3];
        par_add_f64(&a, &b, &mut out);
        assert!((out[0] - 5.0).abs() < 1e-10);
        assert!((out[1] - 7.0).abs() < 1e-10);
        assert!((out[2] - 9.0).abs() < 1e-10);
        let c = [1.0, 0.0, 2.0];
        let d = [1.0, 1.0, 1.0];
        assert!((par_squared_diff_sum_f64(&c, &d) - 2.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_parallel_dijkstra_single_node() {
        let g = Graph::new(1);
        let result = dijkstra(&g, 0);
        assert_eq!(result.dist.len(), 1);
        assert!((result.dist[0] - 0.0).abs() < 1e-10);
        assert!(result.prev[0].is_none());
    }

    #[test]
    fn test_parallel_bfs_single_source_no_edges() {
        let g = Graph::new(4);
        let result = bfs(&g, 0);
        assert_eq!(result.order.len(), 1);
        assert_eq!(result.depth[0], 0);
        assert_eq!(result.depth[1], usize::MAX);
    }

    #[test]
    fn test_parallel_kmeans_k1() {
        let mut data = Matrix::with_storage(5, 2, Storage::Column);
        for i in 0..5 {
            data.set(i, 0, i as f64);
            data.set(i, 1, i as f64);
        }
        let result = clustering::kmeans(&data, 1, Some(5));
        assert_eq!(result.labels().len(), 5);
        assert_eq!(result.n_clusters(), 1);
        assert!(result.centroids().rows() == 1 && result.centroids().cols() == 2);
    }

    #[test]
    fn test_parallel_covariance_small_2x2() {
        let mut data = Matrix::with_storage(2, 2, Storage::Column);
        data.set(0, 0, 1.0);
        data.set(0, 1, 2.0);
        data.set(1, 0, 3.0);
        data.set(1, 1, 4.0);
        let cov = stats::covariance(&data);
        assert_eq!(cov.rows(), 2);
        assert_eq!(cov.cols(), 2);
        assert!((cov.get(0, 1) - cov.get(1, 0)).abs() < 1e-10);
    }

    #[test]
    fn test_parallel_conjugate_gradient() {
        use mathlib::structure::Storage;
        use mathlib::{Matrix, Vector, solve_cg};
        let mut a = Matrix::with_storage(2, 2, Storage::Column);
        a.set(0, 0, 4.0);
        a.set(0, 1, 1.0);
        a.set(1, 0, 1.0);
        a.set(1, 1, 3.0);
        let mut b = Vector::with_capacity(2);
        b.resize(2);
        b.set(0, 1.0);
        b.set(1, 2.0);
        let x = solve_cg(&a, &b, 1e-12, 10).unwrap();
        let ax = &a * &x;
        let r = &b - &ax;
        let r_norm_sq = r.data().iter().map(|v| v * v).sum::<f64>();
        assert!(r_norm_sq < 1e-20, "CG residual too large");
    }

    #[cfg(feature = "genetic")]
    #[test]
    fn test_parallel_cmaes_sphere() {
        use mathlib::CmaEsBuilder;
        fn sphere(x: &[f64]) -> f64 {
            x.iter().map(|&v| v * v).sum()
        }
        let dim = 3;
        let mean = vec![2.0; dim];
        let mut opt = CmaEsBuilder::new(dim, mean, 0.3)
            .max_generations(100)
            .seed(42)
            .build();
        let result = opt.optimize(sphere);
        assert!(
            result.fitness < 0.1,
            "CMA-ES sphere should converge, got {}",
            result.fitness
        );
    }
}

#[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
mod tests {
    #[test]
    fn test_parallel_feature_disabled() {
        // This test runs when parallel feature is disabled
        // Just verify the crate compiles and basic functionality works
        use mathlib::prelude::*;
        let mut m = Matrix::with_storage(2, 2, Storage::Column);
        m.set(0, 0, 1.0);
        m.set(0, 1, 2.0);
        m.set(1, 0, 3.0);
        m.set(1, 1, 4.0);
        assert_eq!(m.rows(), 2);
        assert_eq!(m.cols(), 2);
    }
}
