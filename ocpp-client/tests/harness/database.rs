use std::{collections::HashMap, fs, path::PathBuf};

use rocpp_client::v16::KeyValueStore;

pub struct MockDatabase {
    tables: HashMap<String, HashMap<String, String>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }
}

impl KeyValueStore for MockDatabase {
    async fn db_init(&mut self) {

    }

    async fn db_transaction(&mut self, table: &str, ops: Vec<(&str, Option<&str>)>) {
        let tbl = self.tables.entry(table.to_string()).or_default();
        for (key, value) in ops {
            if let Some(value) = value {
                tbl.insert(key.to_string(), value.to_string());
            } else {
                tbl.remove(key);
            }
        }
    }

    async fn db_get(&mut self, table: &str, key: &str) -> Option<&str> {
        self.tables.get(table).and_then(|tbl| tbl.get(key).map(|t| t.as_str()))
    }

    async fn db_get_all(&mut self, table: &str) -> Vec<(&str, &str)> {
        self.tables
            .get(table)
            .map(|tbl| tbl.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect())
            .unwrap_or_default()
    }

    async fn db_count_keys(&mut self, table: &str) -> usize {
        self.tables.get(table).map(|t| t.len()).unwrap_or(0)
    }

    async fn db_delete_table(&mut self, table: &str) {
        self.tables.remove(table);
    }

    async fn db_delete_all(&mut self) {
        
    }
}

pub struct FileDatabase {
    dir: PathBuf,
    map: HashMap<String, String>
}

impl FileDatabase {
    pub fn new(dir: PathBuf) -> Self {
        fs::create_dir_all(&dir).unwrap();
        Self { dir, map: HashMap::new() }
    }

    fn load_table(&mut self, table: &str) -> &mut HashMap<String, String> {
        let path = self.dir.join(format!("{table}.json"));
        let fresh_map: HashMap<String, String> = if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        };
        self.map = fresh_map;
        &mut self.map
    }

    fn save_table(&self, table: &str) {
        let path = self.dir.join(format!("{table}.json"));
        let data = serde_json::to_string_pretty(&self.map).unwrap();
        fs::write(path, data).unwrap();
    }
}

impl KeyValueStore for FileDatabase {
    async fn db_init(&mut self) {}

    async fn db_transaction(&mut self, table: &str, ops: Vec<(&str, Option<&str>)>) {
        let map = self.load_table(table);
        for (key, value) in ops {
            if let Some(value) = value {
                map.insert(key.to_string(), value.to_string());
            } else {
                map.remove(key);
            }
        }
        self.save_table(table);
    }

    async fn db_get(&mut self, table: &str, key: &str) -> Option<&str> {
        self.load_table(table).get(key).map(|t| t.as_str())
    }

    async fn db_get_all(&mut self, table: &str) -> Vec<(&str, &str)> {
        self.load_table(table).iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    }

    async fn db_count_keys(&mut self, table: &str) -> usize {
        self.load_table(table).len()
    }
    async fn db_delete_table(&mut self, table: &str) {
        let path = self.dir.join(format!("{table}.json"));
        let _ = fs::remove_file(path);
    }
    async fn db_delete_all(&mut self) {
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}
