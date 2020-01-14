rpm_builder
---

This is a project that builds RPMS (or it should).

It is fancy wrapping around the [rpm-rs](https://docs.rs/crate/rpm-rs/0.2.1).
Namely this project builds an executable file which can
consume a `.toml` file which declares the contents of
and RPM. 

The goal being this provides a simple, and git-trackable
method of constructing RPM files.

# CLI

There are 2 primary sub commands:

* `fmt`: Attempts to apply standard formatting to your configuration
* `pkg`: Attempts to package the toml.

They are invoked thusly:

```sh
rpmbuilder fmt [config.toml]
rpmbuilder pgk [config.toml] [output.rpm]
```

Naturally `-H` and `--help` will attempt to provide some help.

# Config

You can find an example configuration within `examples`.
