use super::errors::Err;
use super::rpm::signature::pgp::Signer;
use super::rpm::{RPMBuilder, RPMError, RPMPackage};
use super::serde::{Deserialize, Serialize};

/// Sign expects the path to an ascii pgp asc secret key
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct Sign {
    pub rsa_key_path: String,
}

impl Sign {
    pub fn build<'a>(
        arg: &'a Option<Self>,
    ) -> impl FnOnce(RPMBuilder, &Err) -> Result<RPMPackage, Err> + 'a {
        move |builder: RPMBuilder, err: &Err| -> Result<RPMPackage, Err> {
            let result = match arg {
                Option::None => builder.build(),
                Option::Some(interior) => {
                    let signer = match interior.load(err) {
                        Ok(signer) => signer,
                        Err(e) => return Err(e),
                    };
                    builder.build_and_sign(signer)
                }
            };
            result.map_err(|e| {
                err.clone()
                    .note("failed to build rpm", format_args!("{:?}", e))
            })
        }
    }

    fn load(&self, err: &Err) -> Result<Signer, Err> {
        use std::fs::read_to_string;

        read_to_string(&self.rsa_key_path)
            .map_err(|e| err.clone().note("failed to read rsa key", e))
            .and_then(|key| {
                Signer::load_from_asc(&key).map_err(|e| {
                    err.clone()
                        .note("failed to load asc", format_args!("{:?}", e))
                })
            })
    }
}
