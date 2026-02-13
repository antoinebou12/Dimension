//! Tests for soft body simulation with Neo-Hookean elasticity.

use physics::{Integrator, Particle, PhysicsState, SoftBody, TetMesh, XpbdIntegrator};

/// Helper: build a single-tet soft body and return (state, soft_body_index).
fn single_tet_state(mu: f32, lambda: f32) -> PhysicsState {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let mesh = TetMesh::from_vertices_and_tets(&verts, vec![[0, 1, 2, 3]]);

    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, 0.0, 0.0];
    state.config.substeps = 4;
    state.config.solver_iterations = 8;

    let offset = state.particles.len();
    for v in &verts {
        state.particles.push(Particle::new(*v, 1.0));
    }
    let sb = SoftBody::new(offset, verts.len(), mesh, mu, lambda);
    state.add_soft_body(sb);
    state
}

#[test]
fn tet_mesh_from_vertices() {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let mesh = TetMesh::from_vertices_and_tets(&verts, vec![[0, 1, 2, 3]]);
    assert_eq!(mesh.num_tets(), 1);
    // Volume of a regular tet with edge 1 = 1/6
    assert!(
        (mesh.rest_volumes[0] - 1.0 / 6.0).abs() < 1e-5,
        "rest_volume = {}",
        mesh.rest_volumes[0]
    );
}

#[test]
fn tet_mesh_cube_generation() {
    let (verts, mesh) = TetMesh::cube(2, 2, 2, 1.0);
    assert_eq!(verts.len(), 27); // (2+1)^3
    assert_eq!(mesh.num_tets(), 2 * 2 * 2 * 6);
    // All rest volumes should be positive
    for v in &mesh.rest_volumes {
        assert!(*v > 0.0, "rest volume should be positive, got {v}");
    }
}

#[test]
fn soft_body_at_rest_stays_put() {
    let mut state = single_tet_state(1e4, 1e5);
    let initial: Vec<[f32; 3]> = state.particles.iter().map(|p| p.x).collect();

    let mut integrator = XpbdIntegrator::new();
    for _ in 0..60 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    // Without external forces and starting at rest configuration,
    // vertices should barely move
    for (i, p) in state.particles.iter().enumerate() {
        let dist_sq = (p.x[0] - initial[i][0]).powi(2)
            + (p.x[1] - initial[i][1]).powi(2)
            + (p.x[2] - initial[i][2]).powi(2);
        assert!(
            dist_sq < 0.01,
            "vertex {i} moved too much: {:?} -> {:?}",
            initial[i],
            p.x
        );
    }
}

#[test]
fn soft_body_deformation_recovery() {
    // Deform a soft body slightly, then let Neo-Hookean constraints restore it
    let mut state = single_tet_state(1e3, 1e4);

    // Stretch vertex 1 outward slightly (was [1, 0, 0])
    state.particles[1].x = [1.3, 0.0, 0.0];
    // Pin vertex 0 at origin so the body doesn't drift
    state.particles[0].inv_mass = 0.0;

    let mut integrator = XpbdIntegrator::new();
    for _ in 0..120 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    // After solver iterations, vertex 1 x-position should be closer to 1.0
    // than the initial 1.3
    let dx = (state.particles[1].x[0] - 1.0).abs();
    assert!(
        dx < 0.29,
        "vertex should recover toward rest x=1.0; got x[0] = {}",
        state.particles[1].x[0]
    );
}

#[test]
fn soft_body_volume_preservation() {
    // Create a cube soft body and verify volume is maintained
    let (verts, mesh) = TetMesh::cube(1, 1, 1, 1.0);
    let total_rest_vol: f32 = mesh.rest_volumes.iter().sum();
    assert!(total_rest_vol > 0.0, "rest volume should be positive");

    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, 0.0, 0.0];
    state.config.substeps = 8;
    state.config.solver_iterations = 10;

    let offset = state.particles.len();
    for v in &verts {
        state.particles.push(Particle::new(*v, 1.0));
    }
    // Soft jelly-like material
    let sb = SoftBody::new(offset, verts.len(), mesh.clone(), 50.0, 500.0);
    state.add_soft_body(sb);

    // Mild compression: move top vertices down slightly
    for p in &mut state.particles {
        if p.x[1] > 0.5 {
            p.x[1] *= 0.9; // very mild squish
        }
    }
    // Pin bottom vertices so the body doesn't fly away
    for p in &mut state.particles {
        if p.x[1] < 0.01 {
            p.inv_mass = 0.0;
        }
    }

    let mut integrator = XpbdIntegrator::new();
    for _ in 0..30 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    // Compute current volume as sum of |det(Dm)|/6 per tet
    let mut current_vol = 0.0_f32;
    let mut any_nan = false;
    for t in 0..mesh.num_tets() {
        let tet = mesh.tets[t];
        let p0 = state.particles[offset + tet[0]].x;
        let p1 = state.particles[offset + tet[1]].x;
        let p2 = state.particles[offset + tet[2]].x;
        let p3 = state.particles[offset + tet[3]].x;
        let dm = [
            p1[0] - p0[0],
            p2[0] - p0[0],
            p3[0] - p0[0],
            p1[1] - p0[1],
            p2[1] - p0[1],
            p3[1] - p0[1],
            p1[2] - p0[2],
            p2[2] - p0[2],
            p3[2] - p0[2],
        ];
        let det = mathlib::math3d_raw::mat3_det(&dm);
        if det.is_nan() {
            any_nan = true;
        }
        current_vol += det.abs() / 6.0;
    }

    assert!(!any_nan, "determinant should not be NaN");
    assert!(current_vol > 0.0, "current volume should be positive");

    let ratio = current_vol / total_rest_vol;
    assert!(
        ratio > 0.3 && ratio < 3.0,
        "volume ratio should be reasonable: {ratio} (rest={total_rest_vol}, current={current_vol})"
    );
}

#[test]
fn soft_body_lame_from_youngs() {
    let (mu, lambda) = physics::softbody::lame_from_youngs(1e5, 0.45);
    // μ = E / (2(1+ν)) = 1e5 / (2 * 1.45) ≈ 34482.8
    assert!((mu - 3.4483e4).abs() < 100.0, "mu = {mu}");
    assert!(lambda > 0.0, "lambda should be positive");
}

#[test]
fn soft_body_compliance_values() {
    let verts = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let mesh = TetMesh::from_vertices_and_tets(&verts, vec![[0, 1, 2, 3]]);
    let sb = SoftBody::new(0, 4, mesh, 1000.0, 10000.0);

    let dt_sq = (1.0 / 120.0_f32).powi(2);
    let cd = sb.compliance_deviatoric(0, dt_sq);
    let ch = sb.compliance_hydrostatic(0, dt_sq);

    assert!(cd > 0.0, "deviatoric compliance should be positive");
    assert!(ch > 0.0, "hydrostatic compliance should be positive");
    // Higher lambda → lower hydrostatic compliance (stiffer)
    assert!(ch < cd, "hydrostatic should be stiffer (lower compliance)");
}
