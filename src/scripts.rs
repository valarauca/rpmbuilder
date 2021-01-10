use super::errors::Err;
use super::rpm::{RPMBuilder, RPMError};
use super::serde::{Deserialize, Serialize};

/// Scripts defines information about installation/uninstall scripts.
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct Scripts {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_uninstall: Option<String>,
    pub post_uninstall: Option<String>,
}
impl Scripts {
    pub fn build<'a>(
        arg: &'a Option<Scripts>,
    ) -> impl FnOnce(RPMBuilder, &Err) -> Result<RPMBuilder, Err> + 'a {
        move |builder: RPMBuilder, err: &Err| -> Result<RPMBuilder, Err> {
            let interior = match arg {
                &Option::None => return Ok(builder),
                &Option::Some(ref interior) => interior,
            };

            let mut builder: RPMBuilder = builder;

            builder = (Scripts::load_script(
                &interior.post_install,
                "post_install",
                RPMBuilder::post_install_script,
            ))(builder, err)?;
            builder = (Scripts::load_script(
                &interior.pre_install,
                "pre_install",
                RPMBuilder::pre_install_script,
            ))(builder, err)?;
            builder = (Scripts::load_script(
                &interior.post_uninstall,
                "post_uninstall",
                RPMBuilder::post_uninstall_script,
            ))(builder, err)?;
            builder = (Scripts::load_script(
                &interior.pre_uninstall,
                "pre_uninstall",
                RPMBuilder::pre_uninstall_script,
            ))(builder, err)?;

            Ok(builder)
        }
    }

    fn load_script<'a, F>(
        script: &'a Option<String>,
        name: &'static str,
        lambda: F,
    ) -> impl FnOnce(RPMBuilder, &Err) -> Result<RPMBuilder, Err> + 'a
    where
        F: Fn(RPMBuilder, String) -> RPMBuilder + 'static,
    {
        use std::fs::read_to_string;

        move |builder: RPMBuilder, err: &Err| -> Result<RPMBuilder, Err> {
            let path = match script {
                &Option::None => return Ok(builder),
                &Option::Some(ref path) => path,
            };

            match read_to_string(path) {
                Ok(x) => Ok(lambda(builder, x)),
                Err(e) => Err(err
                    .clone()
                    .note("failed to load script", e)
                    .note("failed on script", name)
                    .note("error on path", path)),
            }
        }
    }

    /*
    /// attempts to load a script from the file system
    fn load<'a>(arg: &Option<String>, err: &Err) -> Result<Option<String>, Err> {
        use std::fs::read_to_string;

        let path = match arg {
            &Option::None => return Ok(None),
            &Option::Some(ref path) => path,
        };
        match read_to_string(path) {
            Ok(contents) => Ok(Some(contents)),
            Err(e) => Err(err.clone().note("failed to read script", e)),
        }
    }

    /// constructs a lambda to produce the next stage of the build
    pub fn build<'a>(&'a self) -> impl FnOnce(RPMBuilder,&Err) -> Result<RPMBuilder, Err> + 'a {
        move |arg: RPMBuilder, err: &Err| -> Result<RPMBuilder, Err> {
            let mut arg: RPMBuilder = arg;

            arg = match Scripts::load(&self.pre_install, err) {
                Err(e) => return Err(e.clone().note("pre_install_path", &self.post_install)),
                Ok(Option::None) => arg,
                Ok(Option::Some(post_install)) => arg.pre_install_script(post_install),
            };

            arg = match Scripts::load(&self.post_install, err) {
                Err(e) => return Err(e.clone().note("post_install_path", &self.post_install)),
                Ok(Option::None) => arg,
                Ok(Option::Some(post_install)) => arg.post_install_script(post_install),
            };

            arg = match Scripts::load(&self.pre_uninstall, err) {
                Err(e) => return Err(e.clone().note("pre_uninstall_path", &self.pre_uninstall)),
                Ok(Option::None) => arg,
                Ok(Option::Some(pre_uninstall)) => arg.pre_uninstall_script(pre_uninstall),
            };

            arg = match Scripts::load(&self.post_uninstall, err) {
                Err(e) => return Err(e.clone().note("post_uninstall_path", &self.pre_uninstall)),
                Ok(Option::None) => arg,
                Ok(Option::Some(pre_uninstall)) => arg.post_uninstall_script(pre_uninstall),
            };

            Ok(arg)
        }
    }
    */
}
