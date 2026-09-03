use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use indexed_db_futures::prelude::*;
use indexed_db_futures::transaction::TransactionMode;
use std::time::Instant;

use crate::model::{
    db,
    error::{ZhongCharError, ZhongCharResult},
    hanzi_strokes_from_json_lines,
};

/// This is the struct that will be stored in the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HanziStrokes {
    pub character: char,
    pub strokes: Vec<String>,
    pub medians: Vec<Vec<Vec<f64>>>,
}

impl HanziStrokes {
    /// Checks if the database is seeded with stroke data and, if not, populates it.
    pub async fn seed_if_needed() -> ZhongCharResult<()> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZI_STROKES_STORE).build()?;
        let store = tx.object_store(db::HANZI_STROKES_STORE)?;

        // 1. Check if the store is already populated.
        if store.count().await? > 0 {
            return Ok(());
        }

        leptos::logging::log!("Starting Hanzi Strokes Store seed...");
        let start_time = window().performance().expect("performance should be available").now();

        // 2. Fetch the data from graphics.txt.
        let base_url = option_env!("BASE_URL").unwrap_or("/");
        let port = window().location().port().unwrap_or_default();
        let port_part = if !port.is_empty() { format!(":{}", port) } else { "".to_string() };
        
        let url = format!(
            "{}//{}{}{}{}",
            window().location().protocol().unwrap(),
            window().location().hostname().unwrap(),
            port_part,
            base_url,
            "graphics.txt", // The new file name
        );
        let text = reqwasm::http::Request::get(&url).send().await?.text().await?;

        // 3. Add the data to the database.
        let tx = db
            .transaction(db::HANZI_STROKES_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        let store = tx.object_store(db::HANZI_STROKES_STORE)?;

        for (i, line) in text.lines().enumerate() {
            // Replace the `if let` with a `match` statement.
            match serde_json::from_str::<hanzi_strokes_from_json_lines::HanziStrokes>(line) {
                Ok(json_strokes) => {
                    // This is the success case, which works as before.
                    let hanzi_strokes = HanziStrokes::from(json_strokes);
                    let value = serde_wasm_bindgen::to_value(&hanzi_strokes)?;
                    // start the work without awaiting, transaction waits for all work to complete
                    let _ = store.add(&value).primitive()?;
                }
                Err(e) => {
                    // This is the new error handling case.
                    // It logs the error to the browser's developer console.
                    leptos::logging::log!(
                        "Skipping line {}: {:?}. Content: '{}'",
                        i + 1, // Add 1 because enumerate is 0-indexed
                        e,
                        line
                    );
                }
            }
        }

        tx.commit().await?;

        let end_time = window().performance().expect("performance should be available").now();
        let elapsed_ms = end_time - start_time;
        leptos::logging::log!("Hanzi Strokes Store seeding finished in {:.2}ms", elapsed_ms);

        Ok(())
    }

    /// Retrieves a single character's stroke data from the DB.
    pub async fn get_one_from_db(character: char) -> ZhongCharResult<Option<HanziStrokes>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZI_STROKES_STORE).build()?;
        let store = tx.object_store(db::HANZI_STROKES_STORE)?;
        let key = JsValue::from_str(&character.to_string());

        match store.get(&key).await? {
            Some(val) => Ok(serde_wasm_bindgen::from_value(val)?),
            None => Ok(None),
        }
    }
}

/// Converts from the raw JSON-line struct to the database-ready struct.
impl From<hanzi_strokes_from_json_lines::HanziStrokes> for HanziStrokes {
    fn from(value: hanzi_strokes_from_json_lines::HanziStrokes) -> Self {
        Self {
            character: value.character.chars().next().unwrap_or('?'),
            strokes: value.strokes,
            medians: value.medians,
        }
    }
}