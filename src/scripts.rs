use super::errors::Err;
use super::rpm::{RPMBuilder, RPMError};
use super::serde::{Deserialize, Serialize};

/// Scripts defines information about installation/uninstall scripts.
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct Scripts {
    pub post_install: Option<String>,
    pub pre_uninstall: Option<String>,
}
impl Scripts {
    /// constructs a lambda to produce the next stage of the build
    pub fn build<'a>(&'a self) -> impl FnOnce(RPMBuilder, Err) -> Result<RPMBuilder, Err> + 'a {
        move |arg: RPMBuilder, err: Err| -> Result<RPMBuilder, Err> {
            let mut arg: RPMBuilder = arg;

            arg = match Scripts::load(
                &self.post_install,
                err.clone().note("post_install_path", &self.post_install),
            ) {
                Err(e) => return Err(e),
                Ok(Option::None) => arg,
                Ok(Option::Some(post_install)) => arg.post_install_script(post_install),
            };

            arg = match Scripts::load(
                &self.pre_uninstall,
                err.clone().note("post_uninstall_path", &self.pre_uninstall),
            ) {
                Err(e) => return Err(e),
                Ok(Option::None) => arg,
                Ok(Option::Some(pre_uninstall)) => arg.pre_uninstall_script(pre_uninstall),
            };
            Ok(arg)
        }
    }

    /// attempts to load a script from the file system
    fn load(arg: &Option<String>, err: Err) -> Result<Option<String>, Err> {
        use std::fs::read_to_string;

        let path = match arg {
            &Option::None => return Ok(None),
            &Option::Some(ref path) => path,
        };
        match read_to_string(path) {
            Ok(contents) => Ok(Some(contents)),
            Err(e) => Err(err.note("failed to read script", e)),
        }
    }
}
