use serde::{Deserialize, Serialize};
use indexed_db_futures::{query_source::QuerySource, Build};
use indexed_db_futures::transaction::TransactionMode;
use wasm_bindgen::JsValue;

use crate::model::{
    db, 
    error::ZhongCharResult,
    exercise::Prerequisite // Import the existing Prerequisite type
};

/// Represents a single vocabulary item (character or word) in the database.
/// This is the "source of truth" from which exercises are generated.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VocabEntry {
    /// The Primary Key. e.g., "char_〇" or "word_你好"
    pub concept_id: String,
    
    /// The character(s) to be learned. e.g., "〇" or "你好"
    pub character: String, 
    
    /// All valid pinyin readings. e.g., ["líng"] or ["yī", "yí", "yì"]
    pub pinyin: Vec<String>,
    
    /// The English meaning(s).
    pub meaning: String,
    
    /// Optional path to the offline audio file, e.g., "/audio/ling2.mp3"
    pub audio_url: Option<String>,
    
    /// List of concepts required before learning this one.
    pub prerequisites: Vec<Prerequisite>,
}

// You will also move your database logic here, targeting the VOCAB_STORE
impl VocabEntry {
    /// Retrieves all vocab entries from the database.
    pub async fn get_all_from_db() -> ZhongCharResult<Vec<VocabEntry>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::VOCAB_STORE).build()?; // <-- Use VOCAB_STORE
        let store = tx.object_store(db::VOCAB_STORE)?;

        let entries_from_db = store
            .get_all()
            .await?
            .collect::<Result<Vec<JsValue>, indexed_db_futures::error::Error>>()?;
        
        let entries = entries_from_db
            .into_iter()
            .filter_map(|val| serde_wasm_bindgen::from_value(val).ok())
            .collect();
            
        Ok(entries)
    }

    /// Adds or updates a vocab entry in the database.
    pub async fn add_to_db(entry: &VocabEntry) -> ZhongCharResult<()> {
        let db = db::open_db().await?;
        let tx = db
            .transaction(db::VOCAB_STORE) // <-- Use VOCAB_STORE
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        let store = tx.object_store(db::VOCAB_STORE)?;
        let val = serde_wasm_bindgen::to_value(entry)?;

        // This assumes your keyPath is "concept_id", per the db.rs update below
        store.put(&val).await?;
        
        tx.commit().await?;
        Ok(())
    }
}
