mod login;
mod logout;
mod sso;

pub use login::*;
pub use logout::*;
pub use sso::*;
#[cfg(test)]
use titanpath::AbsoluteSystemPathBuf;
use titanrepo_api_client::{CacheClient, Client, TokenClient};
use titanrepo_ui::ColorConfig;

use crate::LoginServer;

const KHULNASOFT_TOKEN_DIR: &str = "com.khulnasoft.cli";
const KHULNASOFT_TOKEN_FILE: &str = "auth.json";

pub struct LoginOptions<'a, T: Client + TokenClient + CacheClient> {
    pub color_config: &'a ColorConfig,
    pub login_url: &'a str,
    pub api_client: &'a T,
    pub login_server: &'a dyn LoginServer,

    pub sso_team: Option<&'a str>,
    pub existing_token: Option<&'a str>,
    pub force: bool,
}
impl<'a, T: Client + TokenClient + CacheClient> LoginOptions<'a, T> {
    pub fn new(
        color_config: &'a ColorConfig,
        login_url: &'a str,
        api_client: &'a T,
        login_server: &'a dyn LoginServer,
    ) -> Self {
        Self {
            color_config,
            login_url,
            api_client,
            login_server,
            sso_team: None,
            existing_token: None,
            force: false,
        }
    }
}

/// Options for logging out.
pub struct LogoutOptions<T> {
    pub color_config: ColorConfig,
    pub api_client: T,
    /// If we should invalidate the token on the server.
    pub invalidate: bool,
    /// Path override for testing
    #[cfg(test)]
    pub path: Option<AbsoluteSystemPathBuf>,
}

fn extract_khulnasoft_token() -> Result<Option<String>, Error> {
    let khulnasoft_config_dir =
        titanrepo_dirs::khulnasoft_config_dir()?.ok_or_else(|| Error::ConfigDirNotFound)?;

    let khulnasoft_token_path =
        khulnasoft_config_dir.join_components(&[KHULNASOFT_TOKEN_DIR, KHULNASOFT_TOKEN_FILE]);
    let contents = std::fs::read_to_string(khulnasoft_token_path)?;

    #[derive(serde::Deserialize)]
    struct KhulnasoftToken {
        // This isn't actually dead code, it's used by serde to deserialize the JSON.
        #[allow(dead_code)]
        token: Option<String>,
    }

    Ok(serde_json::from_str::<KhulnasoftToken>(&contents)?.token)
}
