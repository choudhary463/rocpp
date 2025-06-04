use rocpp_client::v16::KeyValueStore;
use sled::Db;

#[derive(Clone)]
pub struct DatabaseService {
    root: Db,
    scratch: Option<String>,
    scratch_all: Vec<(String, String)>
}

impl DatabaseService {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        let root = sled::open(path).expect("failed to open sled db");
        DatabaseService { root, scratch: None, scratch_all: Vec::new() }
    }
    pub async fn get_firmware_version(&mut self) -> String {
        self.db_get("firmware_version", "version").await
            .unwrap_or("0.0.0").to_string()
    }
    pub async fn set_firmware_version(&mut self, version: String) {
        self.db_transaction(
            "firmware_version",
            vec![("version", Some(version.as_str()))],
        ).await;
    }
}

impl KeyValueStore for DatabaseService {
    async fn db_init(&mut self) {
        // nothing needed here for sled
    }

    async fn db_transaction(&mut self, table: &str, ops: Vec<(&str, Option<&str>)>) {
        let tree = self.root.open_tree(table).unwrap();
        for (key, value) in ops {
            if let Some(value) = value {
                let _ = tree.insert(key.as_bytes(), value.as_bytes());
            } else {
                let _ = tree.remove(key.as_bytes());
            }
        }
    }

    async fn db_get(&mut self, table: &str, key: &str) -> Option<&str> {
        let tree = self.root.open_tree(table).ok()?;
        let bytes = tree.get(key.as_bytes()).ok()??;
        let s = String::from_utf8(bytes.to_vec()).ok()?;
        self.scratch = Some(s);
        Some(self.scratch.as_ref().unwrap().as_str())
    }

    async fn db_get_all(&mut self, table: &str) -> Vec<(&str, &str)> {
        let tree = match self.root.open_tree(table) {
            Ok(tree) => tree,
            Err(_) => return vec![],
        };
        self.scratch_all.clear();
        for item in tree.iter().filter_map(|res| res.ok()) {
            let (k_ivec, v_ivec) = item;
            if let (Ok(key), Ok(val)) = (
                String::from_utf8(k_ivec.to_vec()),
                String::from_utf8(v_ivec.to_vec()),
            ) {
                self.scratch_all.push((key, val));
            }
        }
        self.scratch_all
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    async fn db_count_keys(&mut self, table: &str) -> usize {
        self.root.open_tree(table).map(|t| t.len()).unwrap_or(0)
    }

    async fn db_delete_table(&mut self, table: &str) {
        let _ = self.root.drop_tree(table);
    }

    async fn db_delete_all(&mut self) {
        let _ = self.root.clear();
    }
}
