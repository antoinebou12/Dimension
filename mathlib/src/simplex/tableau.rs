//! Tableau-based two-phase simplex implementation.

use super::{SimplexError, SimplexResult, SimplexStatus};
use crate::Storage;
use crate::matrix::Matrix;
use crate::vector::Vector;
use std::f64;

const MAX_ITERATIONS: usize = 50_000;

/// Solve LP in standard form: minimize c'x subject to Ax = b, x >= 0.
///
/// - `c`: objective coefficients (length n).
/// - `A`: constraint matrix (m × n).
/// - `b`: right-hand side (length m).
///
/// Returns the solution vector x (length n), objective value, and status.
///
/// # Errors
///
/// Returns `SimplexError::InconsistentDimensions` if dimensions do not match.
/// Returns `SimplexError::Infeasible` if no feasible solution exists.
/// Returns `SimplexError::Unbounded` if the problem is unbounded.
/// Returns `SimplexError::MaxIterationsExceeded` if the iteration limit is hit.
#[allow(clippy::too_many_lines)] // Single cohesive tableau loop; splitting would obscure the algorithm.
pub fn simplex_solve(
    c: &Vector<f64>,
    a: &Matrix<f64>,
    b: &Vector<f64>,
) -> Result<SimplexResult, SimplexError> {
    let m = a.rows();
    let n = a.cols();
    if b.rows() != m || c.rows() != n {
        return Err(SimplexError::InconsistentDimensions);
    }
    if m == 0 || n == 0 {
        return Err(SimplexError::InconsistentDimensions);
    }

    // Build tableau: (m+1) rows × (n + m + 1) columns. Row-major for easy row ops.
    // Rows 0..m: constraints. Row m: objective.
    // Cols 0..n: structural. Cols n..n+m: artificial (Phase I) / slack. Col n+m: RHS.
    let cols = n + m + 1;
    let rows = m + 1;
    let mut tab = Matrix::with_storage(rows, cols, Storage::Row);
    tab.set_zero();

    // Copy A and b; ensure b >= 0 by flipping rows.
    for i in 0..m {
        for j in 0..n {
            let v = a.get(i, j);
            tab.set(i, j, v);
        }
        let bi = b.get(i);
        if bi < 0.0 {
            for j in 0..n {
                tab.set(i, j, -tab.get(i, j));
            }
            tab.set(i, n + m, -bi);
        } else {
            tab.set(i, n + m, bi);
        }
        tab.set(i, n + i, 1.0); // artificial
    }

    // Phase I objective row: minimize sum of artificials.
    // Reduced costs for structural cols: -sum_i A[i,j]. For artificials 0 (basic). RHS: sum(b).
    for j in 0..n {
        let mut s = 0.0;
        for i in 0..m {
            s += tab.get(i, j);
        }
        tab.set(m, j, -s);
    }
    let mut rhs = 0.0;
    for i in 0..m {
        rhs += tab.get(i, n + m);
    }
    // Store -z in objective row RHS so that pivot row operations update it correctly.
    tab.set(m, n + m, -rhs);

    // Basic variables: initially artificials (cols n..n+m).
    let mut basic_col = vec![0_usize; m];
    for (i, bc) in basic_col.iter_mut().enumerate().take(m) {
        *bc = n + i;
    }

    // Phase I: pivot until obj value is 0 and no artificials in basis.
    for _iter in 0..MAX_ITERATIONS {
        let neg_z = tab.get(m, n + m);
        let has_artificial = basic_col.iter().any(|&b| b >= n);
        if neg_z >= -1e-10 && !has_artificial {
            break;
        }
        if neg_z >= -1e-10 && has_artificial {
            // Drive out remaining artificials (degenerate case).
            let mut pivoted = false;
            for (i, bc) in basic_col.iter_mut().enumerate().take(m) {
                if *bc >= n {
                    for j in 0..n {
                        if tab.get(i, j).abs() > 1e-12 {
                            pivot(&mut tab, m, cols, i, j);
                            *bc = j;
                            pivoted = true;
                            break;
                        }
                    }
                    if pivoted {
                        break;
                    }
                }
            }
            if pivoted {
                continue;
            }
        }
        // Entering: most negative reduced cost; smallest index on tie (Bland's rule).
        let mut best = 0.0_f64;
        for j in 0..n {
            let r = tab.get(m, j);
            if r < best {
                best = r;
            }
        }
        let mut enter_col = None;
        if best < -1e-12 {
            for j in 0..n {
                if tab.get(m, j) <= best + 1e-15 {
                    enter_col = Some(j);
                    break;
                }
            }
        }
        if let Some(j_enter) = enter_col {
            // Leaving: min ratio b_i / A_{i,j} for A_{i,j} > 0.
            let mut leave_row = None;
            let mut min_ratio = f64::INFINITY;
            for i in 0..m {
                let a_ij = tab.get(i, j_enter);
                if a_ij > 1e-12 {
                    let rhs_i = tab.get(i, n + m);
                    let ratio = rhs_i / a_ij;
                    if ratio < min_ratio {
                        min_ratio = ratio;
                        leave_row = Some(i);
                    }
                }
            }
            let Some(i_leave) = leave_row else {
                return Err(SimplexError::Unbounded);
            };
            pivot(&mut tab, m, cols, i_leave, j_enter);
            basic_col[i_leave] = j_enter;
            continue;
        }

        // No negative reduced cost and still have artificials: try to drive one out.
        let mut pivoted = false;
        for (i, bc) in basic_col.iter_mut().enumerate().take(m) {
            if *bc >= n {
                for j in 0..n {
                    if tab.get(i, j).abs() > 1e-12 {
                        pivot(&mut tab, m, cols, i, j);
                        *bc = j;
                        pivoted = true;
                        break;
                    }
                }
                if pivoted {
                    break;
                }
            }
        }
        if !pivoted {
            if tab.get(m, n + m) < -1e-10 {
                return Err(SimplexError::Infeasible);
            }
            break;
        }
    }

    if tab.get(m, n + m) < -1e-10 {
        return Err(SimplexError::Infeasible);
    }

    // Phase II: replace objective row with -c (minimize c'x => reduced costs -c for non-basic).
    for j in 0..n {
        tab.set(m, j, -c.get(j));
    }
    tab.set(m, n + m, 0.0);
    // Zero out basic columns in objective row.
    for (i, &jb) in basic_col.iter().enumerate().take(m) {
        if jb < n {
            let coef = tab.get(m, jb);
            if coef.abs() > 1e-15 {
                for k in 0..cols {
                    tab.set(m, k, tab.get(m, k) - coef * tab.get(i, k));
                }
            }
        }
    }

    // Phase II pivots.
    for _iter in 0..MAX_ITERATIONS {
        let mut enter_col = None;
        let mut best = 0.0;
        for j in 0..n {
            let r = tab.get(m, j);
            if r < -1e-12 && r < best {
                best = r;
                enter_col = Some(j);
            }
        }
        let Some(j_enter) = enter_col else {
            // Optimal (Phase II RHS stores z = c'x after zeroing basic columns)
            let obj_val = tab.get(m, n + m);
            let x = extract_solution(n, m, &tab, &basic_col);
            return Ok(SimplexResult {
                x,
                objective: obj_val,
                status: SimplexStatus::Optimal,
            });
        };
        let mut leave_row = None;
        let mut min_ratio = f64::INFINITY;
        for i in 0..m {
            let a_ij = tab.get(i, j_enter);
            if a_ij > 1e-12 {
                let rhs_i = tab.get(i, n + m);
                let ratio = rhs_i / a_ij;
                if ratio < min_ratio {
                    min_ratio = ratio;
                    leave_row = Some(i);
                }
            }
        }
        let Some(i_leave) = leave_row else {
            return Err(SimplexError::Unbounded);
        };
        pivot(&mut tab, m, cols, i_leave, j_enter);
        basic_col[i_leave] = j_enter;
    }

    Err(SimplexError::MaxIterationsExceeded)
}

fn pivot(
    tab: &mut Matrix<f64>,
    num_constraint_rows: usize,
    cols: usize,
    leave_row: usize,
    enter_col: usize,
) {
    let pivot_val = tab.get(leave_row, enter_col);
    assert!(pivot_val.abs() > 1e-15);

    #[cfg(feature = "simd")]
    {
        use crate::cpu::simd::{scalar_mul_f64, sub_f64};
        let data = tab.data_mut();
        let stride = cols;
        let pivot_start = leave_row * stride;
        let mut pivot_copy = vec![0.0_f64; cols];
        pivot_copy.copy_from_slice(&data[pivot_start..pivot_start + cols]);
        let mut temp = vec![0.0_f64; cols];
        scalar_mul_f64(1.0 / pivot_val, &pivot_copy, &mut temp);
        data[pivot_start..pivot_start + cols].copy_from_slice(&temp);
        let mut row_buf = vec![0.0_f64; cols];
        for i in 0..=num_constraint_rows {
            if i == leave_row {
                continue;
            }
            let fac = data[i * stride + enter_col];
            if fac.abs() > 1e-15 {
                let row_start = i * stride;
                scalar_mul_f64(fac, &pivot_copy, &mut temp);
                sub_f64(&data[row_start..row_start + cols], &temp, &mut row_buf[..]);
                data[row_start..row_start + cols].copy_from_slice(&row_buf);
            }
        }
    }

    #[cfg(not(feature = "simd"))]
    {
        for j in 0..cols {
            tab.set(leave_row, j, tab.get(leave_row, j) / pivot_val);
        }
        for i in 0..=num_constraint_rows {
            if i == leave_row {
                continue;
            }
            let fac = tab.get(i, enter_col);
            if fac.abs() > 1e-15 {
                for j in 0..cols {
                    tab.set(i, j, tab.get(i, j) - fac * tab.get(leave_row, j));
                }
            }
        }
    }
}

fn extract_solution(n: usize, m: usize, tab: &Matrix<f64>, basic_col: &[usize]) -> Vector<f64> {
    let mut x = Vector::with_capacity(n);
    x.set_zero();
    let rhs_col = n + m;
    for (i, &jb) in basic_col.iter().enumerate().take(m) {
        if jb < n {
            x.set(jb, tab.get(i, rhs_col));
        }
    }
    x
}
