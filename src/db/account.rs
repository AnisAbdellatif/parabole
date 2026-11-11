use super::Db;
use crate::model::SourceId;
use anyhow::Result;

impl Db {
    pub fn account_json(&self, source: SourceId) -> Result<Option<String>> {
        Ok(self.conn.query_row(
            "SELECT account_json FROM sources WHERE id = ?1",
            [source],
            |r| r.get(0),
        )?)
    }

    pub fn set_account_json(&self, source: SourceId, json: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sources SET account_json = ?2 WHERE id = ?1",
            (source, json),
        )?;
        Ok(())
    }
}
