use std::f64::consts::PI;

use leptos::logging::log;

use crate::logic::stroke_matching::{geometry, models::Point};

use super::models::{Stroke, UserStroke};

const AVERAGE_DISTANCE_THRESHOLD: f64 = 350.0; // bigger = more lenient
const START_AND_END_DIST_THRESHOLD: f64 = 250.0; // bigger = more lenient
const COSINE_SIMILARITY_THRESHOLD: f64 = 0.0; // -1 to 1, smaller = more lenient
const MIN_LEN_THRESHOLD: f64 = 0.35; // smaller = more lenient
const FRECHET_THRESHOLD: f64 = 0.4; // bigger = more lenient

const SHAPE_FIT_ROTATIONS: [f64; 5] = [
    PI / 16.0,
    PI / 32.0,
    0.0,
    -PI / 32.0,
    -PI / 16.0,
];
pub struct StrokeMatchResult {
    pub is_match: bool,
}

pub fn stroke_matches(user_stroke: &UserStroke, correct_stroke: &Stroke) -> StrokeMatchResult {
    // 1. Pre-computation and Filtering
    if user_stroke.points.len() < 2 {
        return StrokeMatchResult { is_match: false };
    }

    // 2. Average Distance Check (fast fail)
    let avg_dist = geometry::get_average_distance(&user_stroke.points, &correct_stroke.points);
    if avg_dist > AVERAGE_DISTANCE_THRESHOLD {
        log!("❌ Match failed: Avg Distance {:.2}", avg_dist);
        return StrokeMatchResult { is_match: false };
    }
    
    // 2. Start and End Points Check
    if !start_and_end_matches(user_stroke, correct_stroke) {
        log!("❌ Match failed: Start/End Points");
        return StrokeMatchResult { is_match: false };
    }

    // 3. Direction Check
    if !direction_matches(user_stroke, correct_stroke) {
        log!("❌ Match failed: Direction");
        return StrokeMatchResult { is_match: false };
    }

    // 4. Length Check
    if !length_matches(user_stroke, correct_stroke) {
        log!("❌ Match failed: Length");
        return StrokeMatchResult { is_match: false };
    }

    // 5. Shape Check (the final, most precise check)
    if !shape_matches(user_stroke, correct_stroke) {
        log!("❌ Match failed: Shape");
        return StrokeMatchResult { is_match: false };
    }


    log!("✅ Match success!");
    StrokeMatchResult { is_match: true }
}

fn start_and_end_matches(user_stroke: &UserStroke, correct_stroke: &Stroke) -> bool {
    if let (Some(user_start), Some(user_end), Some(correct_start), Some(correct_end)) = (
        user_stroke.points.first(),
        user_stroke.points.last(),
        correct_stroke.get_starting_point(),
        correct_stroke.get_ending_point(),
    ) {
        let start_dist = geometry::distance(*user_start, correct_start);
        let end_dist = geometry::distance(*user_end, correct_end);
        start_dist <= START_AND_END_DIST_THRESHOLD && end_dist <= START_AND_END_DIST_THRESHOLD
    } else {
        false
    }
}

fn direction_matches(user_stroke: &UserStroke, correct_stroke: &Stroke) -> bool {
    let user_vectors: Vec<Point> = user_stroke.points.windows(2).map(|p| geometry::subtract(p[1], p[0])).collect();
    
    let similarities: Vec<f64> = user_vectors.iter().map(|uv| {
        correct_stroke.vectors.iter()
            .map(|cv| geometry::cosine_similarity(*uv, *cv))
            .fold(f64::NEG_INFINITY, f64::max)
    }).collect();

    if similarities.is_empty() { return true; } // Or false, depending on strictness

    let avg_similarity: f64 = similarities.iter().sum::<f64>() / similarities.len() as f64;
    avg_similarity > COSINE_SIMILARITY_THRESHOLD
}

fn length_matches(user_stroke: &UserStroke, correct_stroke: &Stroke) -> bool {
    let user_len = geometry::length(&user_stroke.points);
    log!("user_len + 25: {} correct_len + 25: {} ratio: {}", user_len + 25.0, correct_stroke.length + 25.0, (user_len + 25.0) / (correct_stroke.length + 25.0));
    // Add 25 to each to avoid division by zero and handle short strokes better.
    (user_len + 25.0) / (correct_stroke.length + 25.0) >= MIN_LEN_THRESHOLD
}

fn shape_matches(user_stroke: &UserStroke, correct_stroke: &Stroke) -> bool {
    let norm_user_curve = geometry::normalize_curve(&user_stroke.points);
    let norm_correct_curve = geometry::normalize_curve(&correct_stroke.points);

    let min_dist = SHAPE_FIT_ROTATIONS.iter().fold(f64::INFINITY, |min_so_far, &theta| {
        let rotated_curve = geometry::rotate_curve(&norm_correct_curve, theta);
        let dist = geometry::frechet_dist(&norm_user_curve, &rotated_curve);
        min_so_far.min(dist)
    });

    min_dist <= FRECHET_THRESHOLD
}