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
    /// constructs a lambda which can be invoked on the final builder.
    pub fn build<'a>(
        &'a self,
        src: &'a str,
    ) -> impl FnOnce(RPMBuilder, Err) -> Result<RPMBuilder, Err> + 'a {
        move |builder: RPMBuilder, err: Err| -> Result<RPMBuilder, Err> {
            let opts = match &self {
                &Self::Simple(ref dst) => RPMFileOptions::new(dst),
                &Self::Complex(ref complex) => {
                    let mut opts = RPMFileOptions::new(&complex.dst);
                    opts = complex.user(opts);
                    opts = complex.group(opts);
                    opts = complex.symlink(opts);
                    opts = complex.mode(opts);
                    opts = complex.is_doc(opts);
                    complex.is_config(opts)
                }
            };
            match builder.with_file(src, opts) {
                Ok(x) => Ok(x),
                Err(e) => Err(err.clone().note("src", src.to_string()).err(
                    format_args!("{:?}", e),
                    format!("failed to load src: {:?}", src.to_string()),
                )),
            }
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
    fn user(&self, arg: RPMFileOptionsBuilder) -> RPMFileOptionsBuilder {
        match &self.user {
            &Option::Some(ref user) => arg.user(user),
            &Option::None => arg,
        }
    }

    fn group(&self, arg: RPMFileOptionsBuilder) -> RPMFileOptionsBuilder {
        match &self.group {
            &Option::Some(ref group) => arg.group(group),
            &Option::None => arg,
        }
    }
    fn symlink(&self, arg: RPMFileOptionsBuilder) -> RPMFileOptionsBuilder {
        match &self.symlink {
            &Option::Some(ref symlink) => arg.symlink(symlink),
            &Option::None => arg,
        }
    }
    fn mode(&self, arg: RPMFileOptionsBuilder) -> RPMFileOptionsBuilder {
        match &self.mode {
            &Option::Some(ref mode) => arg.mode(*mode),
            &Option::None => arg,
        }
    }
    fn is_doc(&self, arg: RPMFileOptionsBuilder) -> RPMFileOptionsBuilder {
        match &self.doc {
            &Option::Some(true) => arg.is_doc(),
            _ => arg,
        }
    }
    fn is_config(&self, arg: RPMFileOptionsBuilder) -> RPMFileOptionsBuilder {
        match &self.config {
            &Option::Some(true) => arg.is_config(),
            _ => arg,
        }
    }
}
