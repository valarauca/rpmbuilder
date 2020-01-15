use super::libflate::gzip::Encoder;
use super::rpm::{Compressor, RPMBuilder};
use super::serde::{Deserialize, Serialize};

/// RPM these are required fields for initializing an RPM package build
#[derive(Clone, Hash, Debug, Deserialize, Serialize, Default)]
pub struct RPM {
    pub name: String,
    pub version: String,
    pub license: String,
    pub arch: String,
    pub desc: String,
    pub release: Option<u16>,
    pub gzip: Option<bool>,
}
impl RPM {
    /// initializes the construct of the RPM builder
    pub fn build(&self) -> RPMBuilder {
        let mut builder = RPMBuilder::new(
            &self.name,
            &self.version,
            &self.license,
            &self.arch,
            &self.desc,
        );

        // check if we should gzip compress this
        builder = match &self.gzip {
            &Option::Some(true) => {
                builder.compression(Compressor::Gzip(Encoder::new(Vec::new()).unwrap()))
            }
            _ => builder,
        };

        // check if we're adding release information
        match &self.release {
            &Option::None => builder,
            &Option::Some(ref release) => builder.release(*release),
        }
    }
}
