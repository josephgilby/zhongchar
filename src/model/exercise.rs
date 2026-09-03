use serde::{Deserialize, Serialize};
use indexed_db_futures::{query_source::QuerySource, Build};
use indexed_db_futures::transaction::TransactionMode;
use wasm_bindgen::JsValue;

use crate::model::{db, error::ZhongCharResult};

/// Defines a prerequisite for an exercise.
/// This allows for complex learning dependency graphs.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "id")]
pub enum Prerequisite {
    /// Prerequisite is another exercise.
    /// The ID would be an exercise_id (e.g., "1a_char_你").
    Exercise(String),
    /// Prerequisite is a core concept (vocab/radical).
    /// The ID would be a concept_id (e.g., "char_你" or "rad_9").
    Concept(String),
    /// Prerequisite is an entire module (e.g., a grammar rule).
    /// The ID would be a module_id (e.g., "grammar_measure_words").
    Module(String),
}

/// Data for Exercise Type 1a: Character/Word Recognition
/// Prompt: Show a character.
/// Task: User provides pinyin and meaning.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exercise1a {
    /// The Primary Key for the exercises store. e.g., "1a_char_你"
    pub exercise_id: String,
    
    /// The core concept this exercise tests.
    /// This links to a Hanzi's `character` or a Radical's `number`.
    /// e.g., "char_你" or "rad_9"
    pub target_concept_id: String,
    
    /// The character/word to show the user. e.g., "你"
    pub prompt: String,
    
    /// The expected pinyin answer(s). e.g., ["nǐ"]
    pub pinyin: Vec<String>,
    
    /// The expected meaning answer. e.g., "you"
    pub meaning: String,
    
    /// A list of prerequisites for this exercise.
    /// For a 1a, this might be empty or just the concept itself.
    pub prerequisites: Vec<Prerequisite>,
}

/// An enum that holds all possible exercise types.
/// This allows us to store different-shaped exercises in the same object store.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "exercise_type")]
pub enum Exercise {
    Type1a(Exercise1a),
    // When you're ready, you can add:
    // Type3a(Exercise3a),
    // Type4d(Exercise4d),
}

impl Exercise {
    /// Retrieves all exercises from the database.
    pub async fn get_all_from_db() -> ZhongCharResult<Vec<Exercise>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::EXERCISES_STORE).build()?;
        let store = tx.object_store(db::EXERCISES_STORE)?;

        let exercises_from_db = store
            .get_all()
            .await?
            .collect::<Result<Vec<JsValue>, indexed_db_futures::error::Error>>()?;
        
        let exercises = exercises_from_db
            .into_iter()
            .filter_map(|val| serde_wasm_bindgen::from_value(val).ok())
            .collect();
            
        Ok(exercises)
    }

    pub async fn add_to_db(exercise: &Exercise) -> ZhongCharResult<()> {
        let db = db::open_db().await?;
        let tx = db
            .transaction(db::EXERCISES_STORE)
            .with_mode(TransactionMode::Readwrite) // Correct: with_mode
            .build()?;
        let store = tx.object_store(db::EXERCISES_STORE)?;

        // let key = JsValue::from_str(&exercise.get_key()); // This is no longer needed
        let val = serde_wasm_bindgen::to_value(exercise)?;

        // FIX 1: `put` only takes 1 argument (the value) because the store uses
        // an in-line key (keyPath: "exercise_id"), which it finds thanks to #[serde(flatten)].
        store.put(&val).await?;
        
        // FIX 2: The method is .commit(), not .done()
        tx.commit().await?; // Commit the transaction
        Ok(())
    }
}