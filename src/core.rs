use std::collections::BTreeMap;

use super::chrono::NaiveDateTime;
use super::errors::Err;
use super::rpm::{RPMBuilder, RPMError, RPMPackage};
use super::serde::{Deserialize, Serialize};

use super::changelog::ChangeLogEntry;
use super::fileopts::FileOptions;
use super::rpm_meta::RPM;
use super::scripts::Scripts;
use super::versions::into_dependency;

/// ConfigFile is the top level format for specifying how to
/// build an RPM.
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub rpm: RPM,
    #[serde(default)]
    pub contents: BTreeMap<String, FileOptions>,
    #[serde(default)]
    pub changelog: BTreeMap<NaiveDateTime, ChangeLogEntry>,
    #[serde(default)]
    pub requires: BTreeMap<String, String>,
    #[serde(default)]
    pub obsoletes: BTreeMap<String, String>,
    #[serde(default)]
    pub conflicts: BTreeMap<String, String>,
    #[serde(default)]
    pub provides: BTreeMap<String, String>,
    #[serde(default)]
    pub scripts: Option<Scripts>,
}
impl ConfigFile {
    /// build the RPM in memory
    pub fn build(&self) -> Result<RPMPackage, Err> {
        let mut builder = self.rpm.build();

        let err = Err::default()
            .note("rpm", &self.rpm.name)
            .note("version", &self.rpm.version)
            .note("desc", &self.rpm.desc);

        // package the contents
        for (k, v) in self.contents.iter() {
            let lambda = v.build(k);
            builder = match lambda(builder, err.clone().note("key", v).note("value", v)) {
                Ok(builder) => builder,
                Err(e) => return Err(e),
            };
        }

        // package the change log
        for (time, entry) in self.changelog.iter() {
            builder = entry.build(time)(builder);
        }

        // package database interactions
        for (name, version) in self.requires.iter() {
            builder = builder.requires(into_dependency(name, version));
        }
        for (name, version) in self.obsoletes.iter() {
            builder = builder.obsoletes(into_dependency(name, version));
        }
        for (name, version) in self.conflicts.iter() {
            builder = builder.conflicts(into_dependency(name, version));
        }
        for (name, version) in self.provides.iter() {
            builder = builder.provides(into_dependency(name, version));
        }

        // load scripts if we need to
        builder = match &self.scripts {
            &Option::None => builder,
            &Option::Some(ref scripts) => {
                let err = err.clone().note("scripts", scripts);
                let lambda = scripts.build();
                match lambda(builder, err) {
                    Ok(builder) => builder,
                    Err(e) => return Err(e),
                }
            }
        };
        match builder.build() {
            Ok(x) => Ok(x),
            Err(e) => Err(err.note("failed to build", format_args!("{:?}", e))),
        }
    }
}
