use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
};

use clap::ValueEnum;
use itertools::Itertools;
use titanpath::AbsoluteSystemPathBuf;

use super::{ConfigurationOptions, Error, ResolvedConfigurationOptions};
use crate::{
    cli::{EnvMode, LogOrder},
    titan_json::UIMode,
};

const TITAN_MAPPING: &[(&str, &str)] = [
    ("titan_api", "api_url"),
    ("titan_login", "login_url"),
    ("titan_team", "team_slug"),
    ("titan_teamid", "team_id"),
    ("titan_token", "token"),
    ("titan_remote_cache_timeout", "timeout"),
    ("titan_remote_cache_upload_timeout", "upload_timeout"),
    ("titan_ui", "ui"),
    (
        "titan_dangerously_disable_package_manager_check",
        "allow_no_package_manager",
    ),
    ("titan_daemon", "daemon"),
    ("titan_env_mode", "env_mode"),
    ("titan_cache_dir", "cache_dir"),
    ("titan_preflight", "preflight"),
    ("titan_scm_base", "scm_base"),
    ("titan_scm_head", "scm_head"),
    ("titan_root_titan_json", "root_titan_json_path"),
    ("titan_force", "force"),
    ("titan_log_order", "log_order"),
    ("titan_remote_only", "remote_only"),
    ("titan_remote_cache_read_only", "remote_cache_read_only"),
    ("titan_run_summary", "run_summary"),
    ("titan_allow_no_titan_json", "allow_no_titan_json"),
]
.as_slice();

pub struct EnvVars {
    output_map: HashMap<&'static str, String>,
}

impl EnvVars {
    pub fn new(environment: &HashMap<OsString, OsString>) -> Result<Self, Error> {
        let titan_mapping: HashMap<_, _> = TITAN_MAPPING.iter().copied().collect();
        let output_map = map_environment(titan_mapping, environment)?;
        Ok(Self { output_map })
    }

    fn truthy_value(&self, key: &str) -> Option<Option<bool>> {
        Some(truth_env_var(
            self.output_map.get(key).filter(|s| !s.is_empty())?,
        ))
    }
}

impl ResolvedConfigurationOptions for EnvVars {
    fn get_configuration_options(
        &self,
        _existing_config: &ConfigurationOptions,
    ) -> Result<ConfigurationOptions, Error> {
        // Process signature
        let signature = self
            .truthy_value("signature")
            .map(|value| value.ok_or_else(|| Error::InvalidSignature))
            .transpose()?;

        // Process preflight
        let preflight = self
            .truthy_value("preflight")
            .map(|value| value.ok_or_else(|| Error::InvalidPreflight))
            .transpose()?;

        // Process enabled
        let enabled = self
            .truthy_value("enabled")
            .map(|value| value.ok_or_else(|| Error::InvalidRemoteCacheEnabled))
            .transpose()?;

        let force = self.truthy_value("force").flatten();
        let remote_only = self.truthy_value("remote_only").flatten();
        let remote_cache_read_only = self.truthy_value("remote_cache_read_only").flatten();
        let run_summary = self.truthy_value("run_summary").flatten();
        let allow_no_titan_json = self.truthy_value("allow_no_titan_json").flatten();

        // Process timeout
        let timeout = self
            .output_map
            .get("timeout")
            .map(|s| s.parse())
            .transpose()
            .map_err(Error::InvalidRemoteCacheTimeout)?;

        let upload_timeout = self
            .output_map
            .get("upload_timeout")
            .map(|s| s.parse())
            .transpose()
            .map_err(Error::InvalidUploadTimeout)?;

        // Process experimentalUI
        let ui =
            self.truthy_value("ui")
                .flatten()
                .map(|ui| if ui { UIMode::Tui } else { UIMode::Stream });

        let allow_no_package_manager = self.truthy_value("allow_no_package_manager").flatten();

        // Process daemon
        let daemon = self.truthy_value("daemon").flatten();

        let env_mode = self
            .output_map
            .get("env_mode")
            .map(|s| s.as_str())
            .and_then(|s| match s {
                "strict" => Some(EnvMode::Strict),
                "loose" => Some(EnvMode::Loose),
                _ => None,
            });

        let cache_dir = self.output_map.get("cache_dir").map(|s| s.clone().into());

        let root_titan_json_path = self
            .output_map
            .get("root_titan_json_path")
            .filter(|s| !s.is_empty())
            .map(AbsoluteSystemPathBuf::from_cwd)
            .transpose()?;

        let log_order = self
            .output_map
            .get("log_order")
            .filter(|s| !s.is_empty())
            .map(|s| LogOrder::from_str(s, true))
            .transpose()
            .map_err(|_| {
                Error::InvalidLogOrder(
                    LogOrder::value_variants()
                        .iter()
                        .map(|v| v.to_string())
                        .join(", "),
                )
            })?;

        // We currently don't pick up a Spaces ID via env var, we likely won't
        // continue using the Spaces name, we can add an env var when we have the
        // name we want to stick with.
        let spaces_id = None;

        let output = ConfigurationOptions {
            api_url: self.output_map.get("api_url").cloned(),
            login_url: self.output_map.get("login_url").cloned(),
            team_slug: self.output_map.get("team_slug").cloned(),
            team_id: self.output_map.get("team_id").cloned(),
            token: self.output_map.get("token").cloned(),
            scm_base: self.output_map.get("scm_base").cloned(),
            scm_head: self.output_map.get("scm_head").cloned(),
            // Processed booleans
            signature,
            preflight,
            enabled,
            ui,
            allow_no_package_manager,
            daemon,
            force,
            remote_only,
            remote_cache_read_only,
            run_summary,
            allow_no_titan_json,

            // Processed numbers
            timeout,
            upload_timeout,
            spaces_id,
            env_mode,
            cache_dir,
            root_titan_json_path,
            log_order,
        };

        Ok(output)
    }
}

const KHULNASOFT_ARTIFACTS_MAPPING: &[(&str, &str)] = [
    ("khulnasoft_artifacts_token", "token"),
    ("khulnasoft_artifacts_owner", "team_id"),
]
.as_slice();

pub struct OverrideEnvVars<'a> {
    environment: &'a HashMap<OsString, OsString>,
    output_map: HashMap<&'static str, String>,
}

impl<'a> OverrideEnvVars<'a> {
    pub fn new(environment: &'a HashMap<OsString, OsString>) -> Result<Self, Error> {
        let khulnasoft_artifacts_mapping: HashMap<_, _> =
            KHULNASOFT_ARTIFACTS_MAPPING.iter().copied().collect();

        let output_map = map_environment(khulnasoft_artifacts_mapping, environment)?;
        Ok(Self {
            environment,
            output_map,
        })
    }

    fn ui(&self) -> Option<UIMode> {
        let value = self
            .environment
            .get(OsStr::new("ci"))
            .or_else(|| self.environment.get(OsStr::new("no_color")))?;
        match truth_env_var(value.to_str()?)? {
            true => Some(UIMode::Stream),
            false => None,
        }
    }
}

impl<'a> ResolvedConfigurationOptions for OverrideEnvVars<'a> {
    fn get_configuration_options(
        &self,
        _existing_config: &ConfigurationOptions,
    ) -> Result<ConfigurationOptions, Error> {
        let ui = self.ui();
        let output = ConfigurationOptions {
            team_id: self.output_map.get("team_id").cloned(),
            token: self.output_map.get("token").cloned(),
            api_url: None,
            ui,
            ..Default::default()
        };

        Ok(output)
    }
}

fn truth_env_var(s: &str) -> Option<bool> {
    match s {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn map_environment<'a>(
    mapping: HashMap<&str, &'a str>,
    environment: &HashMap<OsString, OsString>,
) -> Result<HashMap<&'a str, String>, Error> {
    let mut output_map = HashMap::new();
    mapping
        .into_iter()
        .try_for_each(|(mapping_key, mapped_property)| -> Result<(), Error> {
            if let Some(value) = environment.get(OsStr::new(mapping_key)) {
                let converted = value
                    .to_str()
                    .ok_or_else(|| Error::Encoding(mapping_key.to_ascii_uppercase()))?;
                output_map.insert(mapped_property, converted.to_owned());
            }
            Ok(())
        })?;
    Ok(output_map)
}

#[cfg(test)]
mod test {
    use camino::Utf8PathBuf;

    use super::*;
    use crate::{
        cli::LogOrder,
        config::{DEFAULT_API_URL, DEFAULT_LOGIN_URL},
    };

    #[test]
    fn test_env_setting() {
        let mut env: HashMap<OsString, OsString> = HashMap::new();

        let titan_api = "https://example.com/api";
        let titan_login = "https://example.com/login";
        let titan_team = "khulnasoft";
        let titan_teamid = "team_nLlpyC6REAqxydlFKbrMDlud";
        let titan_token = "abcdef1234567890abcdef";
        let cache_dir = Utf8PathBuf::from("nebulo9");
        let titan_remote_cache_timeout = 200;
        let root_titan_json = if cfg!(windows) {
            "C:\\some\\dir\\yolo.json"
        } else {
            "/some/dir/yolo.json"
        };

        env.insert("titan_api".into(), titan_api.into());
        env.insert("titan_login".into(), titan_login.into());
        env.insert("titan_team".into(), titan_team.into());
        env.insert("titan_teamid".into(), titan_teamid.into());
        env.insert("titan_token".into(), titan_token.into());
        env.insert(
            "titan_remote_cache_timeout".into(),
            titan_remote_cache_timeout.to_string().into(),
        );
        env.insert("titan_ui".into(), "true".into());
        env.insert(
            "titan_dangerously_disable_package_manager_check".into(),
            "true".into(),
        );
        env.insert("titan_daemon".into(), "true".into());
        env.insert("titan_preflight".into(), "true".into());
        env.insert("titan_env_mode".into(), "strict".into());
        env.insert("titan_cache_dir".into(), cache_dir.clone().into());
        env.insert("titan_root_titan_json".into(), root_titan_json.into());
        env.insert("titan_force".into(), "1".into());
        env.insert("titan_log_order".into(), "grouped".into());
        env.insert("titan_remote_only".into(), "1".into());
        env.insert("titan_remote_cache_read_only".into(), "1".into());
        env.insert("titan_run_summary".into(), "true".into());
        env.insert("titan_allow_no_titan_json".into(), "true".into());

        let config = EnvVars::new(&env)
            .unwrap()
            .get_configuration_options(&ConfigurationOptions::default())
            .unwrap();
        assert!(config.preflight());
        assert!(config.force());
        assert_eq!(config.log_order(), LogOrder::Grouped);
        assert!(config.remote_only());
        assert!(config.remote_cache_read_only());
        assert!(config.run_summary());
        assert!(config.allow_no_titan_json());
        assert_eq!(titan_api, config.api_url.unwrap());
        assert_eq!(titan_login, config.login_url.unwrap());
        assert_eq!(titan_team, config.team_slug.unwrap());
        assert_eq!(titan_teamid, config.team_id.unwrap());
        assert_eq!(titan_token, config.token.unwrap());
        assert_eq!(titan_remote_cache_timeout, config.timeout.unwrap());
        assert_eq!(Some(UIMode::Tui), config.ui);
        assert_eq!(Some(true), config.allow_no_package_manager);
        assert_eq!(Some(true), config.daemon);
        assert_eq!(Some(EnvMode::Strict), config.env_mode);
        assert_eq!(cache_dir, config.cache_dir.unwrap());
        assert_eq!(
            config.root_titan_json_path,
            Some(AbsoluteSystemPathBuf::new(root_titan_json).unwrap())
        );
    }

    #[test]
    fn test_empty_env_setting() {
        let mut env: HashMap<OsString, OsString> = HashMap::new();
        env.insert("titan_api".into(), "".into());
        env.insert("titan_login".into(), "".into());
        env.insert("titan_team".into(), "".into());
        env.insert("titan_teamid".into(), "".into());
        env.insert("titan_token".into(), "".into());
        env.insert("titan_ui".into(), "".into());
        env.insert("titan_daemon".into(), "".into());
        env.insert("titan_env_mode".into(), "".into());
        env.insert("titan_preflight".into(), "".into());
        env.insert("titan_scm_head".into(), "".into());
        env.insert("titan_scm_base".into(), "".into());
        env.insert("titan_root_titan_json".into(), "".into());
        env.insert("titan_force".into(), "".into());
        env.insert("titan_log_order".into(), "".into());
        env.insert("titan_remote_only".into(), "".into());
        env.insert("titan_remote_cache_read_only".into(), "".into());
        env.insert("titan_run_summary".into(), "".into());
        env.insert("titan_allow_no_titan_json".into(), "".into());

        let config = EnvVars::new(&env)
            .unwrap()
            .get_configuration_options(&ConfigurationOptions::default())
            .unwrap();
        assert_eq!(config.api_url(), DEFAULT_API_URL);
        assert_eq!(config.login_url(), DEFAULT_LOGIN_URL);
        assert_eq!(config.team_slug(), None);
        assert_eq!(config.team_id(), None);
        assert_eq!(config.token(), None);
        assert_eq!(config.ui, None);
        assert_eq!(config.daemon, None);
        assert_eq!(config.env_mode, None);
        assert!(!config.preflight());
        assert_eq!(config.scm_base(), None);
        assert_eq!(config.scm_head(), None);
        assert_eq!(config.root_titan_json_path, None);
        assert!(!config.force());
        assert_eq!(config.log_order(), LogOrder::Auto);
        assert!(!config.remote_only());
        assert!(!config.remote_cache_read_only());
        assert!(!config.run_summary());
        assert!(!config.allow_no_titan_json());
    }

    #[test]
    fn test_override_env_setting() {
        let mut env: HashMap<OsString, OsString> = HashMap::new();

        let khulnasoft_artifacts_token = "correct-horse-battery-staple";
        let khulnasoft_artifacts_owner = "bobby_tables";

        env.insert(
            "khulnasoft_artifacts_token".into(),
            khulnasoft_artifacts_token.into(),
        );
        env.insert(
            "khulnasoft_artifacts_owner".into(),
            khulnasoft_artifacts_owner.into(),
        );
        env.insert("ci".into(), "1".into());

        let config = OverrideEnvVars::new(&env)
            .unwrap()
            .get_configuration_options(&ConfigurationOptions::default())
            .unwrap();
        assert_eq!(khulnasoft_artifacts_token, config.token.unwrap());
        assert_eq!(khulnasoft_artifacts_owner, config.team_id.unwrap());
        assert_eq!(Some(UIMode::Stream), config.ui);
    }
}
