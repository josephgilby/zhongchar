use crate::logic::stroke_matching::models::Point;

/// Subtracts two points to create a vector.
pub fn subtract(p1: Point, p2: Point) -> Point {
    Point { x: p1.x - p2.x, y: p1.y - p2.y }
}

/// Calculates the magnitude (length) of a vector.
pub fn magnitude(vector: Point) -> f64 {
    (vector.x.powi(2) + vector.y.powi(2)).sqrt()
}

/// Calculates the Euclidean distance between two points.
pub fn distance(p1: Point, p2: Point) -> f64 {
    magnitude(subtract(p1, p2))
}

/// Calculates the total length of a curve defined by a series of points.
pub fn length(points: &[Point]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    points.windows(2).map(|p| distance(p[0], p[1])).sum()
}

/// Calculates the cosine similarity between two vectors.
/// Returns a value between -1 (opposite directions) and 1 (same direction).
pub fn cosine_similarity(p1: Point, p2: Point) -> f64 {
    let dot_product = p1.x * p2.x + p1.y * p2.y;
    let magnitude1 = magnitude(p1);
    let magnitude2 = magnitude(p2);
    if magnitude1 == 0.0 || magnitude2 == 0.0 {
        return 0.0;
    }
    dot_product / (magnitude1 * magnitude2)
}

/// For each point in `points_a`, finds the minimum distance to any point in `points_b`,
/// and returns the average of these minimum distances.
pub fn get_average_distance(points_a: &[Point], points_b: &[Point]) -> f64 {
    if points_a.is_empty() || points_b.is_empty() {
        return 0.0;
    }

    let total_dist: f64 = points_a
        .iter()
        .map(|p1| {
            // For each point in A, find the closest point in B.
            points_b
                .iter()
                .map(|p2| distance(*p1, *p2))
                .fold(f64::INFINITY, f64::min)
        })
        .sum();

    total_dist / (points_a.len() as f64)
}

fn extend_point_on_line(p1: Point, p2: Point, dist: f64) -> Point {
    let vect = subtract(p2, p1);
    let norm = dist / magnitude(vect);
    Point {
        x: p2.x + norm * vect.x,
        y: p2.y + norm * vect.y,
    }
}

/// Break up long segments in a curve into smaller segments of a max length.
pub fn subdivide_curve(curve: &[Point], max_len: f64) -> Vec<Point> {
    if curve.is_empty() {
        return Vec::new();
    }
    let mut new_curve = vec![curve[0]];
    for point in curve.iter().skip(1) {
        let prev_point = *new_curve.last().unwrap();
        let seg_len = distance(*point, prev_point);
        if seg_len > max_len {
            let num_new_points = (seg_len / max_len).ceil() as i32;
            let new_seg_len = seg_len / num_new_points as f64;
            for i in 0..num_new_points {
                new_curve.push(extend_point_on_line(*point, prev_point, -1.0 * new_seg_len * (i + 1) as f64));
            }
        } else {
            new_curve.push(*point);
        }
    }
    new_curve
}

/// Redraw a curve using a specific number of equally spaced points.
pub fn outline_curve(curve: &[Point], num_points: usize) -> Vec<Point> {
    if curve.is_empty() {
        return Vec::new();
    }
    let curve_len = length(curve);
    let segment_len = curve_len / (num_points - 1) as f64;
    let mut outline_points = vec![curve[0]];
    let mut remaining_curve_points = curve[1..].to_vec();

    for _ in 0..num_points - 2 {
        let mut last_point = *outline_points.last().unwrap();
        let mut remaining_dist = segment_len;
        let mut outline_point_found = false;
        while !outline_point_found {
            if remaining_curve_points.is_empty() {
                break;
            }
            let next_point_dist = distance(last_point, remaining_curve_points[0]);
            if next_point_dist < remaining_dist {
                remaining_dist -= next_point_dist;
                last_point = remaining_curve_points.remove(0);
            } else {
                let next_point = extend_point_on_line(last_point, remaining_curve_points[0], remaining_dist - next_point_dist);
                outline_points.push(next_point);
                outline_point_found = true;
            }
        }
    }
    outline_points.push(*curve.last().unwrap());
    outline_points
}

/// Translate and scale a curve to a standard size and position.
pub fn normalize_curve(curve: &[Point]) -> Vec<Point> {
    let outlined_curve = outline_curve(curve, 30);
    let mean_x = outlined_curve.iter().map(|p| p.x).sum::<f64>() / outlined_curve.len() as f64;
    let mean_y = outlined_curve.iter().map(|p| p.y).sum::<f64>() / outlined_curve.len() as f64;
    let mean = Point { x: mean_x, y: mean_y };
    let translated_curve: Vec<Point> = outlined_curve.iter().map(|p| subtract(*p, mean)).collect();
    let first = translated_curve[0];
    let last = *translated_curve.last().unwrap();
    let scale = ((first.x.powi(2) + first.y.powi(2) + last.x.powi(2) + last.y.powi(2)) / 2.0).sqrt();
    let scaled_curve: Vec<Point> = translated_curve.iter().map(|p| Point { x: p.x / scale, y: p.y / scale }).collect();
    subdivide_curve(&scaled_curve, 0.05)
}

/// Rotate a curve around the origin by theta radians.
pub fn rotate_curve(curve: &[Point], theta: f64) -> Vec<Point> {
    curve.iter().map(|p| Point {
        x: theta.cos() * p.x - theta.sin() * p.y,
        y: theta.sin() * p.x + theta.cos() * p.y,
    }).collect()
}

/// Calculate the Fréchet distance between two curves.
pub fn frechet_dist(curve1: &[Point], curve2: &[Point]) -> f64 {
    let (long_curve, short_curve) = if curve1.len() >= curve2.len() { (curve1, curve2) } else { (curve2, curve1) };
    
    let mut prev_results_col: Vec<f64> = Vec::new();
    for i in 0..long_curve.len() {
        let mut cur_results_col: Vec<f64> = Vec::new();
        for j in 0..short_curve.len() {
            let val = if i == 0 && j == 0 {
                distance(long_curve[0], short_curve[0])
            } else if i > 0 && j == 0 {
                prev_results_col[0].max(distance(long_curve[i], short_curve[0]))
            } else if i == 0 && j > 0 {
                let last_res = *cur_results_col.last().unwrap();
                last_res.max(distance(long_curve[0], short_curve[j]))
            } else {
                let last_res = *cur_results_col.last().unwrap();
                let min_prev = prev_results_col[j].min(prev_results_col[j-1]).min(last_res);
                min_prev.max(distance(long_curve[i], short_curve[j]))
            };
            cur_results_col.push(val);
        }
        prev_results_col = cur_results_col;
    }
    *prev_results_col.last().unwrap_or(&0.0)
}