use bevy::prelude::Vec2;

/// O(N)
///
/// sample will return interpolated point on spline defined in points
/// points data requirements:
/// - all points sorted by X axis
/// - all values (x,y) clamped between 0.0 and 1.0
/// - t clamped between 0.0 and 1.0
/// - spline len should be not so big ~15-20 points is ok (alg not implement binary search and working with O(N))
/// - result depending on array size:
///     len(0) = 0.0
///     len(1) = value of this single point
///     len(2) = linear lerp between two points
///     len(3) = linear lerp between first and last points
///     len(4+) = catmull-rom interpolation
pub fn sample(points: &[Vec2], t: f32) -> f32 {
    return match points.len() {
        0 => 0.0,
        1 => points[0].y,
        2 => lerp(points[0].y, points[1].y, t),
        3 => lerp(points[0].y, points[2].y, t),
        _ => {
            let last_ind = points.len() - 1;

            // micro opts
            if t <= 0.0 {
                return points[0].y;
            }
            if t >= 1.0 {
                return points[last_ind].y;
            }

            // alg
            let ind = lowest(points, t);

            let cp0 = points[ind];
            let cp1 = match ind < last_ind {
                true => points[ind + 1],
                false => points[last_ind] + Vec2::new(0.01, 0.0),
            };

            let cpm0 = match ind == 0 {
                true => points[ind] - Vec2::new(0.01, 0.0),
                false => points[ind - 1],
            };

            let cpm1 = match ind < last_ind - 1 {
                true => points[ind + 2],
                false => points[last_ind] + Vec2::new(0.01, 0.0),
            };

            let norm_x = normalize(t, cp0.x, cp1.x);

            cubic_hermite(
                norm_x,
                (cpm0.x, cpm0.y),
                (cp0.x, cp0.y),
                (cp1.x, cp1.y),
                (cpm1.x, cpm1.y),
            )
        }
    };
}

#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + ((b - a) * t)
}

#[inline(always)]
fn lowest(points: &[Vec2], t: f32) -> usize {
    let mut ind = points.len() - 1;
    for p in points.iter().rev() {
        if t >= p.x {
            return ind;
        }

        if ind == 0 {
            break;
        }

        ind -= 1;
    }

    return 0;
}

#[inline(always)]
fn normalize(t: f32, start: f32, end: f32) -> f32 {
    (t - start) / (end - start)
}

#[inline(always)]
fn cubic_hermite(t: f32, x: (f32, f32), a: (f32, f32), b: (f32, f32), y: (f32, f32)) -> f32 {
    // sampler stuff
    let two_t = t * 2.;
    let three_t = t * 3.;
    let t2 = t * t;
    let t3 = t2 * t;
    let two_t3 = t2 * two_t;
    let two_t2 = t * two_t;
    let three_t2 = t * three_t;

    // tangents
    let m0 = (b.1 - x.1) / (b.0 - x.0) * (b.0 - a.0);
    let m1 = (y.1 - a.1) / (y.0 - a.0) * (b.0 - a.0);

    a.1 * (two_t3 - three_t2 + 1.)
        + m0 * (t3 - two_t2 + t)
        + b.1 * (three_t2 - two_t3)
        + m1 * (t3 - t2)
}
