use std::env::current_dir;
use std::path::{Path, PathBuf};

use super::errors::Err;
use super::rpm::{RPMBuilder, RPMError, RPMFileOptions, RPMFileOptionsBuilder};
use super::serde::{Deserialize, Serialize};

/// FileOptions provides either a simplified or complex view of where
/// files will end up when installed.
#[derive(Clone, Hash, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FileOptions {
    Simple(String),
    Complex(ComplexFileOptions),
}
impl FileOptions {
    /// constructs a lambda expression which contains all the options to add
    /// this entry into the final RPM
    pub fn build<'a>(
        &'a self,
        source: &'a str,
    ) -> impl FnOnce(Result<RPMBuilder, Err>, &Err) -> Result<RPMBuilder, Err> + 'a {
        move |builder_result, err| -> Result<RPMBuilder, Err> {
            builder_result?
                .with_file(source, self.make_opts())
                .map_err(|e| {
                    err.clone()
                        .note("failed to load src", format_args!("{:?}", e))
                        .note("src", source.to_string())
                })
        }
    }

    fn make_opts(&self) -> RPMFileOptionsBuilder {
        match self {
            &FileOptions::Simple(ref dst) => RPMFileOptions::new(dst.to_string()),
            &FileOptions::Complex(ref cmp) => cmp.build(),
        }
    }
}

/// ComplexFileOptions encodes RPM specific options for an individual file.
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct ComplexFileOptions {
    pub dst: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub symlink: Option<String>,
    #[serde(default)]
    pub mode: Option<i32>,
    #[serde(default)]
    pub doc: Option<bool>,
    #[serde(default)]
    pub config: Option<bool>,
}
impl ComplexFileOptions {
    fn build(&self) -> RPMFileOptionsBuilder {
        let mut opts = RPMFileOptions::new(&self.dst.clone());
        opts = (Self::add_optional(&self.user, RPMFileOptionsBuilder::user))(opts);
        opts = (Self::add_optional(&self.group, RPMFileOptionsBuilder::group))(opts);
        opts = (Self::add_optional(&self.symlink, RPMFileOptionsBuilder::symlink))(opts);
        opts = (Self::add_optional(&self.mode, RPMFileOptionsBuilder::mode))(opts);
        if self.doc == Option::Some(true) {
            opts = opts.is_doc();
        }
        if self.config == Option::Some(true) {
            opts = opts.is_config();
        }
        opts
    }

    fn add_optional<'a, T, F>(
        arg: &'a Option<T>,
        lambda: F,
    ) -> impl Fn(RPMFileOptionsBuilder) -> RPMFileOptionsBuilder + 'a
    where
        F: Fn(RPMFileOptionsBuilder, T) -> RPMFileOptionsBuilder + 'static,
        T: Clone,
    {
        move |builder: RPMFileOptionsBuilder| -> RPMFileOptionsBuilder {
            match arg {
                &Option::None => builder,
                &Option::Some(ref arg) => lambda(builder, arg.clone()),
            }
        }
    }
}
