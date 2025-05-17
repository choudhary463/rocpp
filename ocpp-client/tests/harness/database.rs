use std::{collections::HashMap, fs, path::PathBuf};

use ocpp_client::v16::{Database, TableOperation};

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

impl Database for MockDatabase {
    fn init(&mut self) {}

    fn transaction(&mut self, table: &str, ops: Vec<TableOperation>) {
        let tbl = self.tables.entry(table.to_string()).or_default();
        for op in ops {
            match op {
                TableOperation::Insert { key, value } => {
                    tbl.insert(key, value);
                }
                TableOperation::Delete { key } => {
                    tbl.remove(&key);
                }
            }
        }
    }

    fn get(&mut self, table: &str, key: &str) -> Option<String> {
        self.tables.get(table).and_then(|tbl| tbl.get(key).cloned())
    }

    fn get_all(&mut self, table: &str) -> Vec<(String, String)> {
        self.tables
            .get(table)
            .map(|tbl| tbl.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    fn delete_table(&mut self, table: &str) {
        self.tables.remove(table);
    }
}

pub struct FileDatabase {
    dir: PathBuf,
}

impl FileDatabase {
    pub fn new(dir: PathBuf) -> Self {
        fs::create_dir_all(&dir).unwrap();
        Self { dir }
    }

    fn load_table(&self, table: &str) -> HashMap<String, String> {
        let path = self.dir.join(format!("{table}.json"));
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    fn save_table(&self, table: &str, map: &HashMap<String, String>) {
        let path = self.dir.join(format!("{table}.json"));
        let data = serde_json::to_string_pretty(map).unwrap();
        fs::write(path, data).unwrap();
    }
}

impl Database for FileDatabase {
    fn init(&mut self) {}

    fn transaction(&mut self, table: &str, ops: Vec<TableOperation>) {
        let mut map = self.load_table(table);
        for op in ops {
            match op {
                TableOperation::Insert { key, value } => {
                    map.insert(key, value);
                }
                TableOperation::Delete { key } => {
                    map.remove(&key);
                }
            }
        }
        self.save_table(table, &map);
    }

    fn get(&mut self, table: &str, key: &str) -> Option<String> {
        self.load_table(table).get(key).cloned()
    }

    fn get_all(&mut self, table: &str) -> Vec<(String, String)> {
        self.load_table(table).into_iter().collect()
    }

    fn delete_table(&mut self, table: &str) {
        let path = self.dir.join(format!("{table}.json"));
        let _ = fs::remove_file(path);
    }
}
