use super::rpm::Dependency;

pub fn into_dependency(name: &str, version: &str) -> Dependency {
    Option::None
        .into_iter()
        /*
         * attempt to infer a standard prefix
         *
         */
        .chain(magic("=", eq, name, version))
        .chain(magic("==", eq, name, version))
        .chain(magic("<", less, name, version))
        .chain(magic("<=", less_eq, name, version))
        .chain(magic(">", greater, name, version))
        .chain(magic(">=", greater_eq, name, version))
        .chain(magic("^", greater_eq, name, version))
        .chain(magic("~", greater_eq, name, version))
        /*
         * allow some wild cards, and empty space
         *
         */
        .chain(any_prefix("*", any, name, version))
        .chain(any_prefix("", any, name, version))
        /*
         * fallback to exactly what was given
         *
         */
        .chain(Some(Dependency::eq(name, version)))
        .next()
        .unwrap()
}

fn any_prefix<F>(prefix: &str, lambda: F, name: &str, version: &str) -> Option<Dependency>
where
    F: Fn(&str) -> Dependency,
{
    if prefix == version {
        Some(lambda(name.trim()))
    } else {
        None
    }
}

fn magic<F>(prefix: &str, lambda: F, name: &str, version: &str) -> Option<Dependency>
where
    F: Fn(&str, &str) -> Dependency,
{
    let version_info = version.trim_start_matches(prefix);
    if (version_info.len() + prefix.len()) == version.len() {
        Some(lambda(name.trim(), version_info.trim()))
    } else {
        None
    }
}

/*
 * make some logical lambdas
 *
 */
fn eq(name: &str, version: &str) -> Dependency {
    Dependency::eq(name, version)
}
fn less_eq(name: &str, version: &str) -> Dependency {
    Dependency::less_eq(name, version)
}
fn greater_eq(name: &str, version: &str) -> Dependency {
    Dependency::greater_eq(name, version)
}
fn greater(name: &str, version: &str) -> Dependency {
    Dependency::greater(name, version)
}
fn less(name: &str, version: &str) -> Dependency {
    Dependency::less(name, version)
}
fn any(name: &str) -> Dependency {
    Dependency::any(name)
}
