use log::info;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub meaning: String,
    pub success_count: i32,
    pub threshold: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub memory_threshold: i32,
    pub window_opacity: f64,
    pub always_on_top: bool,
    pub mouse_penetrate: bool,
    pub float_delay: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            memory_threshold: 5,
            window_opacity: 1.0,
            always_on_top: false,
            mouse_penetrate: false,
            float_delay: 3,
        }
    }
}

pub fn get_db_path() -> PathBuf {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let data_dir = PathBuf::from("/data/data/com.wordrember.app/databases");
        fs::create_dir_all(&data_dir).ok();
        data_dir.join("words.db")
    }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("WordRember");
        fs::create_dir_all(&data_dir).ok();
        data_dir.join("words.db")
    }
}

pub fn init_db() -> SqlResult<Connection> {
    let db_path = get_db_path();
    info!("Database path: {:?}", db_path);
    let conn = Connection::open(db_path)?;
    
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS words (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            word TEXT NOT NULL UNIQUE,
            meaning TEXT NOT NULL,
            success_count INTEGER DEFAULT 0,
            threshold INTEGER DEFAULT 5
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS learned_words (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            word_id INTEGER NOT NULL,
            learned_at TEXT NOT NULL,
            FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 先清理已有的重复记录
    conn.execute(
        "DELETE FROM learned_words WHERE id NOT IN (SELECT MIN(id) FROM learned_words GROUP BY word_id)",
        [],
    )?;

    // 再添加唯一索引防止重复记录
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_learned_words_word_id ON learned_words(word_id)",
        [],
    )?;

    Ok(conn)
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[tauri::command]
fn get_words(state: State<AppState>) -> Result<Vec<Word>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, word, meaning, success_count, threshold FROM words WHERE id NOT IN (SELECT word_id FROM learned_words) ORDER BY RANDOM()")
        .map_err(|e| e.to_string())?;

    let words = stmt
        .query_map([], |row| {
            Ok(Word {
                id: row.get(0)?,
                word: row.get(1)?,
                meaning: row.get(2)?,
                success_count: row.get(3)?,
                threshold: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(words)
}

#[tauri::command]
fn get_all_words(state: State<AppState>) -> Result<Vec<Word>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, word, meaning, success_count, threshold FROM words ORDER BY word")
        .map_err(|e| e.to_string())?;

    let words = stmt
        .query_map([], |row| {
            Ok(Word {
                id: row.get(0)?,
                word: row.get(1)?,
                meaning: row.get(2)?,
                success_count: row.get(3)?,
                threshold: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(words)
}

#[tauri::command]
fn get_learned_words(state: State<AppState>) -> Result<Vec<Word>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT w.id, w.word, w.meaning, w.success_count, w.threshold FROM words w INNER JOIN learned_words lw ON w.id = lw.word_id ORDER BY w.word")
        .map_err(|e| e.to_string())?;

    let words = stmt
        .query_map([], |row| {
            Ok(Word {
                id: row.get(0)?,
                word: row.get(1)?,
                meaning: row.get(2)?,
                success_count: row.get(3)?,
                threshold: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(words)
}

#[tauri::command]
fn get_word_stats(state: State<AppState>) -> Result<(i64, i64), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let learned: i64 = conn
        .query_row("SELECT COUNT(DISTINCT word_id) FROM learned_words", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok((total, learned))
}

#[tauri::command]
fn add_word(state: State<AppState>, word: String, meaning: String) -> Result<Word, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO words (word, meaning, success_count, threshold) VALUES (?1, ?2, 0, 5)",
        params![word, meaning],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, word, meaning, success_count, threshold FROM words WHERE word = ?1")
        .map_err(|e| e.to_string())?;

    stmt.query_row(params![word], |row| {
        Ok(Word {
            id: row.get(0)?,
            word: row.get(1)?,
            meaning: row.get(2)?,
            success_count: row.get(3)?,
            threshold: row.get(4)?,
        })
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn import_words(state: State<AppState>, content: String, format: String) -> Result<i32, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut count = 0;

    match format.as_str() {
        "csv" => {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let word = parts[0].trim().trim_matches('"');
                    let meaning = parts[1].trim().trim_matches('"');
                    if !word.is_empty() && !meaning.is_empty() {
                        conn.execute(
                            "INSERT OR IGNORE INTO words (word, meaning, success_count, threshold) VALUES (?1, ?2, 0, 5)",
                            params![word, meaning],
                        )
                        .ok();
                        count += 1;
                    }
                }
            }
        }
        "json" => {
            if let Ok(words) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                for item in words {
                    if let (Some(word), Some(meaning)) = (item.get("word"), item.get("meaning")) {
                        let word_str = word.as_str().unwrap_or("");
                        let meaning_str = meaning.as_str().unwrap_or("");
                        if !word_str.is_empty() && !meaning_str.is_empty() {
                            conn.execute(
                                "INSERT OR IGNORE INTO words (word, meaning, success_count, threshold) VALUES (?1, ?2, 0, 5)",
                                params![word_str, meaning_str],
                            )
                            .ok();
                            count += 1;
                        }
                    }
                }
            }
        }
        "txt" => {
            for line in content.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let word = parts[0].trim();
                    let meaning = parts[1].trim();
                    if !word.is_empty() && !meaning.is_empty() {
                        conn.execute(
                            "INSERT OR IGNORE INTO words (word, meaning, success_count, threshold) VALUES (?1, ?2, 0, 5)",
                            params![word, meaning],
                        )
                        .ok();
                        count += 1;
                    }
                }
            }
        }
        _ => return Err("Unsupported format".to_string()),
    }

    info!("Imported {} words", count);
    Ok(count)
}

#[tauri::command]
fn mark_remembered(state: State<AppState>, word_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE words SET success_count = success_count + 1 WHERE id = ?1",
        params![word_id],
    )
    .map_err(|e| e.to_string())?;

    let threshold: i32 = conn
        .query_row(
            "SELECT threshold FROM words WHERE id = ?1",
            params![word_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let success_count: i32 = conn
        .query_row(
            "SELECT success_count FROM words WHERE id = ?1",
            params![word_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if success_count >= threshold {
        // 检查是否已存在于 learned_words 表
        let already_learned: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM learned_words WHERE word_id = ?1",
                params![word_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !already_learned {
            let now = chrono_lite_now();
            conn.execute(
                "INSERT INTO learned_words (word_id, learned_at) VALUES (?1, ?2)",
                params![word_id, now],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
fn mark_not_remembered(state: State<AppState>, word_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE words SET success_count = 0 WHERE id = ?1",
        params![word_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_word(state: State<AppState>, word_id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    info!("Deleting word with id: {}", word_id);
    
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    
    tx.execute("DELETE FROM learned_words WHERE word_id = ?1", params![word_id])
        .map_err(|e| {
            info!("Error deleting from learned_words: {}", e);
            e.to_string()
        })?;
    
    let deleted = tx.execute("DELETE FROM words WHERE id = ?1", params![word_id])
        .map_err(|e| {
            info!("Error deleting from words: {}", e);
            e.to_string()
        })?;
    
    info!("Deleted {} rows from words", deleted);
    
    tx.commit().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn reset_progress(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM learned_words", [])
        .map_err(|e| e.to_string())?;
    conn.execute("UPDATE words SET success_count = 0", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut settings = AppSettings::default();

    if let Ok(threshold) = conn.query_row::<i32, _, _>(
        "SELECT value FROM settings WHERE key = 'memory_threshold'",
        [],
        |row| row.get(0),
    ) {
        settings.memory_threshold = threshold;
    }

    if let Ok(opacity) = conn.query_row::<f64, _, _>(
        "SELECT value FROM settings WHERE key = 'window_opacity'",
        [],
        |row| row.get(0),
    ) {
        settings.window_opacity = opacity;
    }

    if let Ok(on_top) = conn.query_row::<i32, _, _>(
        "SELECT value FROM settings WHERE key = 'always_on_top'",
        [],
        |row| row.get(0),
    ) {
        settings.always_on_top = on_top == 1;
    }

    if let Ok(penetrate) = conn.query_row::<i32, _, _>(
        "SELECT value FROM settings WHERE key = 'mouse_penetrate'",
        [],
        |row| row.get(0),
    ) {
        settings.mouse_penetrate = penetrate == 1;
    }

    if let Ok(delay) = conn.query_row::<i32, _, _>(
        "SELECT value FROM settings WHERE key = 'float_delay'",
        [],
        |row| row.get(0),
    ) {
        settings.float_delay = delay;
    }

    Ok(settings)
}

#[tauri::command]
fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('memory_threshold', ?1)",
        params![settings.memory_threshold.to_string()],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('window_opacity', ?1)",
        params![settings.window_opacity.to_string()],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('always_on_top', ?1)",
        params![if settings.always_on_top { "1" } else { "0" }],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('mouse_penetrate', ?1)",
        params![if settings.mouse_penetrate { "1" } else { "0" }],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('float_delay', ?1)",
        params![settings.float_delay.to_string()],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_always_on_top(app: AppHandle, on_top: bool) -> Result<(), String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        if let Some(window) = app.get_webview_window("main") {
            window.set_always_on_top(on_top).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn set_window_opacity(_app: AppHandle, _opacity: f64) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn set_mouse_penetrate(_app: AppHandle, _penetrate: bool) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn minimize_to_tray(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn show_window(_app: AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn create_float_window(_app: AppHandle, _word: String, _meaning: String, _delay: i32) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting WordRember application");

    let db = match init_db() {
        Ok(conn) => conn,
        Err(e) => {
            panic!("Database initialization failed: {}", e);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState { db: Mutex::new(db) })
        .setup(|_app| {
            info!("Setting up application");
            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_words,
            get_all_words,
            get_learned_words,
            get_word_stats,
            add_word,
            import_words,
            mark_remembered,
            mark_not_remembered,
            delete_word,
            reset_progress,
            get_settings,
            save_settings,
            set_always_on_top,
            set_window_opacity,
            set_mouse_penetrate,
            minimize_to_tray,
            show_window,
            create_float_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
