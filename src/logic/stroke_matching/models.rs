use serde::{Deserialize, Serialize};

use crate::logic::stroke_matching::geometry;

/// Represents a single 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Represents a stroke drawn by the user, composed of a series of points.
#[derive(Debug)]
pub struct UserStroke {
    pub points: Vec<Point>,
}

/// Represents a single correct stroke from the character data.
/// It contains the median points used for matching.
#[derive(Debug)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub vectors: Vec<Point>,
    pub length: f64,
}

impl Stroke {
    pub fn new(points: Vec<Point>) -> Self {
        let vectors = if points.len() > 1 {
            points.windows(2).map(|p| geometry::subtract(p[1], p[0])).collect()
        } else {
            Vec::new()
        };
        let length = geometry::length(&points);
        Self { points, vectors, length }
    }

    pub fn get_starting_point(&self) -> Option<Point> {
        self.points.first().copied()
    }

    pub fn get_ending_point(&self) -> Option<Point> {
        self.points.last().copied()
    }
}
