use super::Db;
use crate::model::*;
use anyhow::Result;

impl Db {
    /// Sets many category visibilities in one transaction. Items: (source, key, hidden).
    pub fn set_categories_hidden(
        &self,
        profile: ProfileId,
        media: MediaKind,
        items: &[(SourceId, String, bool)],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for (source, key, hidden) in items {
            self.set_category_hidden(profile, *source, media, key, *hidden)?;
        }
        Ok(tx.commit()?)
    }
}
