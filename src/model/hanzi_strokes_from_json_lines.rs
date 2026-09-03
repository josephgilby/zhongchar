use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct HanziStrokes {
    pub character: String,
    pub strokes: Vec<String>,
    pub medians: Vec<Vec<Vec<f64>>>,
}