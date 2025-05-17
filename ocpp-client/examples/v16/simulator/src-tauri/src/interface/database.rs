use ocpp_client::v16::{Database, TableOperation};
use sled::Db;

#[derive(Clone)]
pub struct DatabaseService {
    root: Db,
}

impl DatabaseService {
    pub fn new(path: &str) -> Self {
        let root = sled::open(path).expect("failed to open sled db");
        DatabaseService { root }
    }
    pub fn get_firmware_version(&mut self) -> String {
        self.get("firmware_version", "version")
            .unwrap_or(format!("0.0.0"))
    }
    pub fn set_firmware_version(&mut self, version: String) {
        self.transaction(
            "firmware_version",
            vec![TableOperation::insert("version".into(), version.into())],
        );
    }
}

impl Database for DatabaseService {
    fn init(&mut self) {
        // nothing needed here for sled
    }

    fn transaction(&mut self, table: &str, ops: Vec<TableOperation>) {
        let tree = self.root.open_tree(table).unwrap();
        for op in ops {
            match op {
                TableOperation::Insert { key, value } => {
                    let _ = tree.insert(key.as_bytes(), value.as_bytes());
                }
                TableOperation::Delete { key } => {
                    let _ = tree.remove(key.as_bytes());
                }
            }
        }
    }

    fn get(&mut self, table: &str, key: &str) -> Option<String> {
        let tree = self.root.open_tree(table).ok()?;
        tree.get(key.as_bytes())
            .ok()
            .flatten()
            .map(|ivec| String::from_utf8(ivec.to_vec()).ok())
            .flatten()
    }

    fn get_all(&mut self, table: &str) -> Vec<(String, String)> {
        let tree = match self.root.open_tree(table) {
            Ok(tree) => tree,
            Err(_) => return vec![],
        };

        tree.iter()
            .filter_map(|res| res.ok())
            .filter_map(|(k, v)| {
                let key = String::from_utf8(k.to_vec()).ok()?;
                let val = String::from_utf8(v.to_vec()).ok()?;
                Some((key, val))
            })
            .collect()
    }

    fn delete_table(&mut self, table: &str) {
        let _ = self.root.drop_tree(table);
    }
}
