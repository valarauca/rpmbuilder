use super::chrono::NaiveDateTime;
use super::rpm::RPMBuilder;
use super::serde::{Deserialize, Serialize};

/// ChangeLogEntry describes what changed, and by whom
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct ChangeLogEntry {
    pub author: String,
    pub entry: String,
}
impl ChangeLogEntry {
    /// creates a lambda which can modify the existing RPM builder
    pub fn build<'a>(
        &'a self,
        when: &'a NaiveDateTime,
    ) -> impl FnOnce(RPMBuilder) -> RPMBuilder + 'a {
        move |arg: RPMBuilder| -> RPMBuilder {
            // TODO: I'm not 100% sure this is correct but it feels right so idk.
            //       review this code before 2038.
            let timestamp = when.timestamp() as i32;
            arg.add_changelog_entry(&self.author, &self.entry, timestamp)
        }
    }
}
