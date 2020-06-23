use super::{Context, Module, RootModuleConfig, SegmentConfig};

use crate::configs::perl::PerlConfig;
use crate::utils;

/// Creates a module with the current perl version
///
/// Will display the perl version if any of the following criteria are met:
///     - Current directory contains a `.pl` or a `.pm` file
///     - Current directory contains a `composer.json` or `.perl-version` file
pub fn module<'a>(context: &'a Context) -> Option<Module<'a>> {
    let is_perl_project = context
        .try_begin_scan()?
        .set_files(&["Makefile.PL", "cpanfile", "META.json", "META.yml", ".perl-version"])
        .set_extensions(&["pl", "pm"])
        .is_match();

    if !is_perl_project {
        return None;
    }

    let perl_version = utils::exec_cmd("perl", &["-e", "print substr($^V, 1);",])?.stdout;
    let formatted_version = format_perl_version(&perl_version)?;


    let mut module = context.new_module("perl");
    let config: PerlConfig = PerlConfig::try_load(module.config);
    module.set_style(config.style);

    module.create_segment("symbol", &config.symbol);
    module.create_segment("version", &SegmentConfig::new(&formatted_version));

    Some(module)
}

fn format_perl_version(perl_version: &str) -> Option<String> {
    let formatted_version = format!("v{}", &perl_version);
    Some(formatted_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::utils::test::render_module;
    use ansi_term::Color;
    use std::fs::File;
    use std::io;

    #[test]
    fn test_format_perl_version() {
        let input = "5.30.0";
        assert_eq!(format_perl_version(input), Some("v5.30.0".to_string()));
    }

    #[test]
    fn folder_without_perl_files() -> io::Result<()> {
        let dir = tempfile::tempdir()?;

        let actual = render_module("perl", dir.path(), None);

        let expected = None;
        assert_eq!(expected, actual);
        dir.close()
    }

    #[test]
    fn folder_with_cpanfile_file() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("cpanfile"))?.sync_all()?;

        let actual = render_module("perl", dir.path(), None);

        let expected = Some(format!(
            "via {} ",
            Color::Fixed(149).bold().paint("🐪 v5.30.0")
        ));
        assert_eq!(expected, actual);
        dir.close()
    }

    #[test]
    fn folder_with_perl_version() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join(".perl-version"))?.sync_all()?;

        let actual = render_module("perl", dir.path(), None);

        let expected = Some(format!(
            "via {} ",
            Color::Fixed(149).bold().paint("🐪 v5.30.0")
        ));
        assert_eq!(expected, actual);
        dir.close()
    }

    #[test]
    fn folder_with_perl_file() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        File::create(dir.path().join("any.pl"))?.sync_all()?;

        let actual = render_module("perl", dir.path(), None);

        let expected = Some(format!(
            "via {} ",
            Color::Fixed(149).bold().paint("🐪 v5.30.0")
        ));
        assert_eq!(expected, actual);
        dir.close()
    }
}