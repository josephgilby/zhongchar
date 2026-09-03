use std::ops::Range;
use std::time::Instant;

use indexed_db_futures::BuildPrimitive;
use indexed_db_futures::{prelude::QuerySource, Build};
use indexed_db_futures::transaction::TransactionMode;
use indexed_db_futures::cursor::CursorDirection;
use leptos::prelude::window;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::model::error::ZhongCharError;
use crate::model::{db, error::ZhongCharResult, hanzi_from_json_lines};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Etymology {
    pub r#type: String, // `r#` escapes the Rust keyword `type`
    pub hint: String,
    pub phonetic: Option<String>,
    pub semantic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hanzi {
    pub character: char,
    pub definition: Option<String>,
    pub pinyin: Vec<String>,
    pub decomposition: String,
    pub etymology: Option<Etymology>,
    pub radical: String,
    pub matches: Vec<Option<Vec<u32>>>,
}

impl Hanzi {
    pub async fn seed_if_needed() -> ZhongCharResult<()> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZIS_STORE).build()?;
        let store = tx.object_store(db::HANZIS_STORE)?;

        let count = store.count().await?;
        if count > 0 {
            return Ok(()); // Already seeded
        }

        leptos::logging::log!("Starting Hanzi Store seed...");
        let start_time = window().performance().expect("performance should be available").now();

        let base_url = option_env!("BASE_URL").unwrap_or("/");
        let port = window().location().port(); // Get port as Option<String>
        let port_part = match port {
            Ok(p) if !p.is_empty() => format!(":{}", p), 
            _ => "".to_string(),
        };
        let url = format!(
            "{}//{}{}{}{}",
            window().location().protocol().unwrap(),
            window().location().hostname().unwrap(),
            port_part,
            base_url, 
            "dictionary.txt",
        );
        let text = reqwasm::http::Request::get(&url).send().await?.text().await?;

        let tx = db.transaction(db::HANZIS_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        let store = tx.object_store(db::HANZIS_STORE)?;

        for line in text.lines() {
            if let Ok(json_hanzi) = serde_json::from_str::<hanzi_from_json_lines::Hanzi>(line) {
                let hanzi = Hanzi::from(json_hanzi);
                let value = serde_wasm_bindgen::to_value(&hanzi)
                    .map_err(|e| ZhongCharError::Wasm(e.to_string()))?;
                let _ = store.add(&value).primitive()?;
            }
        }

        tx.commit().await?;

        let end_time = window().performance().expect("performance should be available").now();
        let elapsed_ms = end_time - start_time;
        leptos::logging::log!("Hanzi Store seeding finished in {:.2}ms", elapsed_ms);

        Ok(())
    }

    pub async fn get_one_from_db(character: char) -> ZhongCharResult<Option<Hanzi>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZIS_STORE).build()?;
        let store = tx.object_store(db::HANZIS_STORE)?;

        // Convert the char to a JsValue to use as a key
        let key = JsValue::from_str(&character.to_string());

        // Use the `get` method to find the specific record
        let result: Option<JsValue> = store.get(&key).await?;

        // Deserialize the JsValue back into a Hanzi struct if it was found
        match result {
            Some(val) => {
                let hanzi: Hanzi = serde_wasm_bindgen::from_value(val)?;
                Ok(Some(hanzi))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_from_db() -> ZhongCharResult<Vec<Hanzi>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZIS_STORE).build()?;
        let store = tx.object_store(db::HANZIS_STORE)?;

        let hanzi_from_db= store
            .get_all()
            .await?
            .collect::<Result<Vec<JsValue>, indexed_db_futures::error::Error>>()?;
        
        // Deserialize each JsValue back into a Hanzi struct.
        let hanzi_list = hanzi_from_db
            .into_iter()
            .filter_map(|val| serde_wasm_bindgen::from_value(val).ok())
            .collect();
            
        Ok(hanzi_list)
    }

    pub async fn get_count() -> ZhongCharResult<usize> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZIS_STORE).build()?;
        let store = tx.object_store(db::HANZIS_STORE)?;
        Ok(store.count().await? as usize)
    }

    pub async fn get_range(range: Range<usize>) -> ZhongCharResult<Vec<Hanzi>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::HANZIS_STORE).build()?;
        let store = tx.object_store(db::HANZIS_STORE)?;

        let Some(mut cursor) = store
            .open_cursor()
            .with_direction(CursorDirection::Next)
            .await? else {
                return Ok(Vec::new());
            };
        if range.start > 0 {
            cursor.advance_by(range.start as u32).await?;
        }

        let mut hanzis = Vec::with_capacity(range.len());
        for _ in 0..range.len() {
            // `next_record` gets the current value and advances the cursor.
            // It returns an Option because we might reach the end of the store.
            match cursor.next_record::<JsValue>().await? {
                Some(value) => {
                    if let Ok(hanzi) = serde_wasm_bindgen::from_value(value) {
                        hanzis.push(hanzi);
                    }
                }
                // If it returns None, there are no more records, so we stop.
                None => break,
            }
        }
        
        Ok(hanzis)

    }
}


impl From<hanzi_from_json_lines::Hanzi> for Hanzi {
    fn from(value: hanzi_from_json_lines::Hanzi) -> Self {
        let etymology = value.etymology.map(|mut e| Etymology {
            r#type: e.remove("type").unwrap_or_default(),
            hint: e.remove("hint").unwrap_or_default(),
            phonetic: e.remove("phonetic"),
            semantic: e.remove("semantic"),
        });

        Self {
            character: value.character.chars().next().unwrap_or('?'),
            definition: value.definition,
            pinyin: value.pinyin,
            decomposition: value.decomposition,
            etymology, // The new struct
            radical: value.radical,
            matches: value.matches,
        }
    }
}