use leptos::prelude::window;
use serde::Deserialize;
use std::collections::HashMap;

use crate::model::error::ZhongCharResult; // Using HashMap for the flexible etymology

#[derive(Debug, Deserialize, Clone)]
pub struct Hanzi {
    pub character: String,
    pub definition: Option<String>,
    pub pinyin: Vec<String>,
    pub decomposition: String,
    // Using a HashMap is a flexible way to handle the optional phonetic/semantic fields
    pub etymology: Option<HashMap<String, String>>, 
    pub radical: String,
    // This field can contain a mix of arrays and nulls
    pub matches: Vec<Option<Vec<u32>>>, 
}