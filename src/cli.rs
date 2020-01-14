use super::clap::{App, Arg, ArgMatches, SubCommand};

pub fn cli_build<'a>() -> ArgMatches<'a> {
    App::new("rpm_builder")
        .author("valarauca, <codylaeder@gmail.com>")
        .version("0.0.1")
        .about("transforms a declarative TOML input into an RPM")
        .set_term_width(80)
        .subcommand(
            App::new("fmt")
                .about("applies standard formatting to toml arguments")
                .arg(
                    Arg::with_name("config")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .validator(validate_config),
                ),
        )
        .subcommand(
            App::new("pkg")
                .about("packages the rpm")
                .arg(
                    Arg::with_name("config")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .validator(validate_config),
                )
                .arg(
                    Arg::with_name("output")
                        .index(2)
                        .takes_value(true)
                        .required(true)
                        .multiple(false),
                ),
        )
        .get_matches()
}

pub enum AppWork<'a> {
    Format(Format<'a>),
    Package(Package<'a>),
}
impl<'a> AppWork<'a> {

    pub fn from_args(arg: &'a ArgMatches<'a>) -> AppWork<'a> {
        match arg.subcommand() {
            ("fmt", Option::Some(ref args)) => {
                AppWork::Format(Format{config: args.value_of("config").unwrap() })
            },
            ("pkg", Option::Some(ref args)) => {
                AppWork::Package(Package{config: args.value_of("config").unwrap(), output: args.value_of("output").unwrap() })
            },
            (idk,_) => {
                panic!("unrecongized subcommand: {}", idk)
            }
        }
    }

    pub fn work(&self) -> Result<(),String> {
        match self {
            &Self::Format(ref fmt) => fmt.work(),
            &Self::Package(ref pkg) => pkg.work(),
        }
    }
}

/// Format allows for standard formatting of configuration files.
pub struct Format<'a> {
    pub config: &'a str,
}
impl<'a> Format<'a> {
    fn work(&self) -> Result<(), String> {
        use super::core::ConfigFile;
        use super::toml::de::from_str;
        use super::toml::ser::to_string_pretty;
        use std::fs::{read_to_string, write};

        let data = match read_to_string(self.config) {
            Ok(data) => data,
            Err(e) => {
                return Err(format!(
                    "could not read config:'{}' error:'{:?}'",
                    self.config, e
                ))
            }
        };
        let values = match from_str::<ConfigFile>(&data) {
            Ok(data) => data,
            Err(e) => {
                return Err(format!(
                    "could not read config:'{}' as toml. error:'{:?}'",
                    self.config, e
                ))
            }
        };
        let output =
            match to_string_pretty(&values) {
                Ok(data) => data,
                Err(e) => return Err(format!(
                    "post loading config:'{}' encounter toml error while serializing. error:'{:?}'",
                    self.config, e
                )),
            };
        match write(self.config, output.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!(
                "failed to write formatted output of config:'{}' back error:'{:?}'",
                self.config, e
            )),
        }
    }
}

/// Package allows for creating packages
pub struct Package<'a> {
    pub config: &'a str,
    pub output: &'a str,
}
impl<'a> Package<'a> {
    fn work(&self) -> Result<(), String> {
        use super::core::ConfigFile;
        use super::toml::de::from_str;
        use std::fs::{read_to_string, OpenOptions};
        use std::io::Write;

        let data = match read_to_string(self.config) {
            Ok(data) => data,
            Err(e) => {
                return Err(format!(
                    "could not read config:'{}' error:'{:?}'",
                    self.config, e
                ))
            }
        };
        let values = match from_str::<ConfigFile>(&data) {
            Ok(data) => data,
            Err(e) => {
                return Err(format!(
                    "could not read config:'{}' as toml. error:'{:?}'",
                    self.config, e
                ))
            }
        };
        let output = match values.build() {
            Ok(data) => data,
            Err(e) => return Err(format!("failed to build RPM. error:'{:?}'", e)),
        };
        let mut f = match OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.output)
        {
            Ok(file) => file,
            Err(e) => {
                return Err(format!(
                    "failed to open/create output path:'{}' error:'{:?}'",
                    self.output, e
                ))
            }
        };
        match output.write(&mut f) {
            Ok(()) => {}
            Err(e) => {
                return Err(format!(
                    "failed to write output to path:'{}' error:'{:?}'",
                    self.output, e
                ))
            }
        };
        match f.flush() {
            Ok(()) => Ok(()),
            Err(e) => Err(format!(
                "failed to flush path:'{}' after RPM writing. error:'{:?}'",
                self.output, e
            )),
        }
    }
}

fn validate_config(path: String) -> Result<(), String> {
    use super::core::ConfigFile;
    use super::toml::de::{from_str, Error as TomlError};
    use std::error::Error;
    use std::fs::read_to_string;

    // read the file
    let output = match read_to_string(&path) {
        Ok(output) => output,
        Err(e) => {
            return Err(format!(
                "could not read config from path:'{}' error:'{:?}'",
                &path, e
            ))
        }
    };

    // attempt to parse the file
    match from_str::<ConfigFile>(&output) {
        Ok(_) => Ok(()),
        Err(e) => match e.line_col() {
            Option::None => Err(format!(
                "could not read config from path:'{}' toml_error:'{:?}'",
                &path, e
            )),
            Option::Some((line, _)) => {
                let err_line = output
                    .lines()
                    .enumerate()
                    .filter(|(i, _)| *i == line)
                    .map(|(_, line)| line)
                    .next()
                    .unwrap();
                Err(format!(
                    "could not read config from path:'{}'\nerror:'{}' on line:'{}'\n{}",
                    &path,
                    e.description(),
                    line,
                    err_line
                ))
            }
        },
    }
}
