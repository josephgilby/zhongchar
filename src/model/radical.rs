use std::time::Instant;

use indexed_db_futures::BuildPrimitive;
use indexed_db_futures::{query_source::QuerySource, Build};
use indexed_db_futures::transaction::TransactionMode;
use leptos::logging::log;
use serde::{Deserialize, Serialize};
use leptos::prelude::*;
use wasm_bindgen::JsValue;
use crate::helpers::prepend_relative_url;

use crate::model::error::ZhongCharError;
use crate::model::{db, radical_from_csv};

use super::error::ZhongCharResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Radical {
    pub number: i32,
    pub radical_forms: Vec<char>,
    pub stroke_count: i32,
    pub meaning: String,
    pub colloquial_term: Option<String>,
    pub pinyin: String,
    pub han_viet: String,
    pub hiragana_romaji: String,
    pub hangul_romaja: String,
    pub frequency: i32,
    pub simplified: Option<String>,
    pub examples: String,
}

impl Radical {

    pub async fn seed_if_needed() -> ZhongCharResult<()> {
        let db = db::open_db().await?;

        let tx = db.transaction(db::RADICALS_STORE).build()?;
        let store = tx.object_store(db::RADICALS_STORE)?;

        let count = store.count().await?;
        if count > 0 {
            return Ok(()); // Already seeded
        }

        leptos::logging::log!("Starting Radical Store seed...");
        let start_time = window().performance().expect("performance should be available").now();

        // --- Database is empty, so we seed it ---
        let radicals_to_seed = Self::fetch_radicals().await?;
        let tx = db.transaction(db::RADICALS_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        let store = tx.object_store(db::RADICALS_STORE)?;

        for radical in &radicals_to_seed {
            let value = serde_wasm_bindgen::to_value(radical)
                .map_err(|e| ZhongCharError::Wasm(e.to_string()))?;
            let _ = store.add(&value).primitive()?;
        }

        tx.commit().await?;

        let end_time = window().performance().expect("performance should be available").now();
        let elapsed_ms = end_time - start_time;
        leptos::logging::log!("Radical Store seeding finished in {:.2}ms", elapsed_ms);

        Ok(())
    }

    pub async fn get_all_from_db() -> ZhongCharResult<Vec<Radical>> {
        let db = db::open_db().await?;
        let tx = db.transaction(db::RADICALS_STORE).build()?;
        let store = tx.object_store(db::RADICALS_STORE)?;

        let radicals_from_db= store
                .get_all()
                .await?
                .collect::<Result<Vec<JsValue>, indexed_db_futures::error::Error>>()?;
        
        let radicals = radicals_from_db
            .into_iter()
            .filter_map(|val| serde_wasm_bindgen::from_value(val).ok())
            .collect();
            
        Ok(radicals)
    }


    pub async fn get_all() -> ZhongCharResult<Vec<Radical>> {
        let db = db::open_db().await?;

        let tx = db.transaction(db::RADICALS_STORE).build()?;
        let store = tx.object_store(db::RADICALS_STORE)?;

        // Await the `Count` builder first, then use `?` on the Result.
        let count = store.count().await?;
        if count > 0 {
            // Await the `GetAllRecords` builder first, then use `?` on the Result.
            let radicals_from_db= store
                .get_all()
                .await?
                .collect::<Result<Vec<JsValue>, indexed_db_futures::error::Error>>()?;
            let radicals: Vec<Radical> = radicals_from_db
                .into_iter()
                .filter_map(|val| serde_wasm_bindgen::from_value(val).ok())
                .collect();
            return Ok(radicals);
        }

        // --- If we reach here, the DB is empty and needs to be seeded ---
        
        let radicals_to_seed = Self::fetch_radicals().await?;
        let tx = db.transaction(db::RADICALS_STORE)
            .with_mode(TransactionMode::Readwrite)
            .build()?;
        let store = tx.object_store(db::RADICALS_STORE)?;

        for radical in &radicals_to_seed {
            let value = serde_wasm_bindgen::to_value(radical)
                .map_err(|e| ZhongCharError::Wasm(e.to_string()))?;
            // .add() returns a Result, so `?` is correct here.
            store.add(&value).await?;
        }

        tx.commit().await?;

        Ok(radicals_to_seed)
    }
    
    pub async fn fetch_radicals() -> ZhongCharResult<Vec<Radical>> {
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
            "radicals.csv",
        );
        let text = 
            reqwasm::http::Request::get(&url)
                .send()
                .await?
                .text()
                .await?;
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(text.as_bytes());
        let mut radicals: Vec<Radical> = Vec::new();
        let des = reader.deserialize();
        for result in des {
            let radical: radical_from_csv::Radical = result?;
            radicals.push(Radical::from(radical));
        }
        
        Ok(radicals)
    }

}

impl From<radical_from_csv::Radical> for Radical {
    fn from(value: radical_from_csv::Radical) -> Self {
        let radical_forms_vec: Vec<char> = value.radical_forms.chars()
            .filter(|c| c.is_alphabetic() && !c.is_ascii())
            .collect();
        Self {
            radical_forms: radical_forms_vec, 
            number: value.number,
            stroke_count: value.stroke_count,
            meaning: value.meaning,
            colloquial_term: value.colloquial_term,
            pinyin: value.pinyin,
            han_viet: value.han_viet,
            hiragana_romaji: value.hiragana_romaji,
            hangul_romaja: value.hangul_romaja,
            frequency: value.frequency,
            simplified: value.simplified,
            examples: value.examples,
        }
    }
}