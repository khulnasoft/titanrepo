use camino::Utf8PathBuf;
use titanpath::{AbsoluteSystemPath, RelativeUnixPath};

use super::{ConfigurationOptions, Error, ResolvedConfigurationOptions};
use crate::titan_json::RawTitanJson;

pub struct TitanJsonReader<'a> {
    repo_root: &'a AbsoluteSystemPath,
}

impl<'a> TitanJsonReader<'a> {
    pub fn new(repo_root: &'a AbsoluteSystemPath) -> Self {
        Self { repo_root }
    }
}

impl<'a> ResolvedConfigurationOptions for TitanJsonReader<'a> {
    fn get_configuration_options(
        &self,
        existing_config: &ConfigurationOptions,
    ) -> Result<ConfigurationOptions, Error> {
        let titan_json_path = existing_config.root_titan_json_path(self.repo_root);
        let titan_json = RawTitanJson::read(self.repo_root, &titan_json_path).or_else(|e| {
            if let Error::Io(e) = &e {
                if matches!(e.kind(), std::io::ErrorKind::NotFound) {
                    return Ok(Default::default());
                }
            }

            Err(e)
        })?;
        let mut opts = if let Some(remote_cache_options) = &titan_json.remote_cache {
            remote_cache_options.into()
        } else {
            ConfigurationOptions::default()
        };

        let cache_dir = if let Some(cache_dir) = titan_json.cache_dir {
            let cache_dir_str: &str = &cache_dir;
            let cache_dir_unix = RelativeUnixPath::new(cache_dir_str).map_err(|_| {
                let (span, text) = cache_dir.span_and_text("titan.json");
                Error::AbsoluteCacheDir { span, text }
            })?;
            // Convert the relative unix path to an anchored system path
            // For unix/macos this is a no-op
            let cache_dir_system = cache_dir_unix.to_anchored_system_path_buf();
            Some(Utf8PathBuf::from(cache_dir_system.to_string()))
        } else {
            None
        };

        // Don't allow token to be set for shared config.
        opts.token = None;
        opts.spaces_id = titan_json
            .experimental_spaces
            .and_then(|spaces| spaces.id)
            .map(|spaces_id| spaces_id.into());
        opts.ui = titan_json.ui;
        opts.allow_no_package_manager = titan_json.allow_no_package_manager;
        opts.daemon = titan_json.daemon.map(|daemon| *daemon.as_inner());
        opts.env_mode = titan_json.env_mode;
        opts.cache_dir = cache_dir;
        Ok(opts)
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_reads_from_default() {
        let tmpdir = tempdir().unwrap();
        let repo_root = AbsoluteSystemPath::new(tmpdir.path().to_str().unwrap()).unwrap();

        let existing_config = ConfigurationOptions {
            ..Default::default()
        };
        repo_root
            .join_component("titan.json")
            .create_with_contents(
                serde_json::to_string_pretty(&serde_json::json!({
                    "daemon": false
                }))
                .unwrap(),
            )
            .unwrap();

        let reader = TitanJsonReader::new(repo_root);
        let config = reader.get_configuration_options(&existing_config).unwrap();
        // Make sure we read the default titan.json
        assert_eq!(config.daemon(), Some(false));
    }

    #[test]
    fn test_respects_root_titan_json_config() {
        let tmpdir = tempdir().unwrap();
        let tmpdir_path = AbsoluteSystemPath::new(tmpdir.path().to_str().unwrap()).unwrap();
        let root_titan_json_path = tmpdir_path.join_component("yolo.json");
        let repo_root = AbsoluteSystemPath::new(if cfg!(windows) {
            "C:\\my\\repo"
        } else {
            "/my/repo"
        })
        .unwrap();
        let existing_config = ConfigurationOptions {
            root_titan_json_path: Some(root_titan_json_path.to_owned()),
            ..Default::default()
        };
        root_titan_json_path
            .create_with_contents(
                serde_json::to_string_pretty(&serde_json::json!({
                    "daemon": false
                }))
                .unwrap(),
            )
            .unwrap();

        let reader = TitanJsonReader::new(repo_root);
        let config = reader.get_configuration_options(&existing_config).unwrap();
        // Make sure we read the correct titan.json
        assert_eq!(config.daemon(), Some(false));
    }
}
