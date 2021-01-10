use std::collections::BTreeMap;

use super::chrono::NaiveDateTime;
use super::errors::Err;
use super::rpm::{Dependency, RPMBuilder, RPMError, RPMPackage};
use super::serde::{Deserialize, Serialize};

use super::changelog::ChangeLogEntry;
use super::fileopts::FileOptions;
use super::rpm_meta::RPM;
use super::scripts::Scripts;
use super::sign::Sign;
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
    #[serde(default)]
    pub signature: Option<Sign>,
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
        builder = self
            .contents
            .iter()
            .map(|(source, options)| options.build(source))
            .fold(Ok(builder), |builder_res, opts_bundle| {
                (opts_bundle)(builder_res, &err)
            })?;

        // package the change log
        builder = self
            .changelog
            .iter()
            .map(|(time, entry)| entry.build(time))
            .fold(builder, |b, e| (e)(b));

        // package database interactions
        builder = (add_rpm_db_interaction(self.requires.iter(), RPMBuilder::requires))(builder);
        builder = (add_rpm_db_interaction(self.obsoletes.iter(), RPMBuilder::obsoletes))(builder);
        builder = (add_rpm_db_interaction(self.conflicts.iter(), RPMBuilder::conflicts))(builder);
        builder = (add_rpm_db_interaction(self.provides.iter(), RPMBuilder::provides))(builder);

        // load scripts if we need to
        builder = (Scripts::build(&self.scripts))(builder, &err)?;

        // signing occurs last
        let finalizer = Sign::build(&self.signature);
        finalizer(builder, &err).map_err(|e| e.note("sucess", false))
    }
}

fn add_rpm_db_interaction<'a, I, F>(
    iter: I,
    lambda: F,
) -> impl FnOnce(RPMBuilder) -> RPMBuilder + 'a
where
    I: Iterator<Item = (&'a String, &'a String)> + 'a,
    F: Fn(RPMBuilder, Dependency) -> RPMBuilder + 'static,
{
    move |builder| -> RPMBuilder {
        iter.map(|(name, version)| into_dependency(name, version))
            .fold(builder, |builder, dep| lambda(builder, dep))
    }
}
