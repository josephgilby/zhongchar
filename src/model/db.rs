use indexed_db_futures::{
    database::{Database, VersionChangeEvent}, error::{Error, OpenDbError}, Build, KeyPath
};

use leptos::logging::log; // Import the log macro

pub const DB_NAME: &str = "ZhongCharDB";
pub const RADICALS_STORE: &str = "radicals";
pub const HANZIS_STORE: &str = "hanzis";
pub const HANZI_STROKES_STORE: &str = "hanzi_strokes";
pub const EXERCISES_STORE: &str = "exercises";
pub const VOCAB_STORE: &str = "vocab";

pub async fn open_db() -> Result<Database, OpenDbError> {
    let db_req = Database::open(DB_NAME);

    let db_req = db_req
        .with_on_blocked(|_evt| {
            log!("Database upgrade is blocked, likely by another open tab.");
            Ok(())
        })
        .with_on_upgrade_needed(|_evt: VersionChangeEvent, db| -> Result<(), Error> {
            if !db.object_store_names().any(|name| name == RADICALS_STORE) {
                db.create_object_store(RADICALS_STORE)
                    .with_key_path(KeyPath::One(String::from("number")))
                    .build()?;
            }

            if !db.object_store_names().any(|name| name == HANZIS_STORE) {
                db.create_object_store(HANZIS_STORE)
                    .with_key_path(KeyPath::One(String::from("character")))
                    .build()?;
            }

            if !db.object_store_names().any(|name| name == HANZI_STROKES_STORE) {
                db.create_object_store(HANZI_STROKES_STORE)
                    .with_key_path(KeyPath::One(String::from("character")))
                    .build()?;
            }

            if !db.object_store_names().any(|name| name == EXERCISES_STORE) {
                db.create_object_store(EXERCISES_STORE)
                    .with_key_path(KeyPath::One(String::from("exercise_id")))
                    .build()?;
            }
            
            if !db.object_store_names().any(|name| name == VOCAB_STORE) {
                db.create_object_store(VOCAB_STORE)
                    .with_key_path(KeyPath::One(String::from("concept_id")))
                    .build()?;
            }
            Ok(())
        });

    // Await the request directly, as its error type now matches the function's.
    db_req.await
}