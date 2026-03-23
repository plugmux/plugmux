//! Embedded database module (redb).
//!
//! Currently stores request/response logs.
//! Future phases: config, catalog, agents, sync metadata.

pub mod logs;

use std::path::Path;
use std::sync::Arc;

use redb::Database;

pub struct Db {
    pub inner: Database,
}

impl Db {
    pub fn open(path: &Path) -> Result<Arc<Self>, Box<redb::Error>> {
        #[allow(clippy::result_large_err)]
        fn inner(path: &Path) -> Result<Database, redb::Error> {
            let db = Database::create(path)?;
            let write_txn = db.begin_write()?;
            {
                let _ = write_txn.open_table(logs::LOGS_TABLE);
            }
            write_txn.commit()?;
            Ok(db)
        }
        let db = inner(path).map_err(Box::new)?;
        Ok(Arc::new(Self { inner: db }))
    }

    pub fn default_path() -> std::path::PathBuf {
        crate::config::config_dir().join("plugmux.db")
    }
}
