mod local_titan_config;
mod local_titan_state;
mod parser;
mod titan_state;

use std::{backtrace::Backtrace, env, process, process::Stdio, time::Duration};

use dunce::canonicalize as fs_canonicalize;
use local_titan_config::LocalTitanConfig;
use local_titan_state::{titan_version_has_shim, LocalTitanState};
use miette::{Diagnostic, SourceSpan};
use parser::{MultipleCwd, ShimArgs};
use thiserror::Error;
use tiny_gradient::{GradientStr, RGB};
use tracing::{debug, warn};
pub use titan_state::TitanState;
use titan_updater::display_update_check;
use titanpath::AbsoluteSystemPathBuf;
use titanrepo_repository::inference::{RepoMode, RepoState};
use titanrepo_ui::ColorConfig;
use which::which;

use crate::{cli, get_version, spawn_child, tracing::TitanSubscriber};

const TITAN_GLOBAL_WARNING_DISABLED: &str = "TITAN_GLOBAL_WARNING_DISABLED";

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error(transparent)]
    #[diagnostic(transparent)]
    MultipleCwd(Box<MultipleCwd>),
    #[error("No value assigned to `--cwd` flag")]
    #[diagnostic(code(titan::shim::empty_cwd))]
    EmptyCwd {
        #[backtrace]
        backtrace: Backtrace,
        #[source_code]
        args_string: String,
        #[label = "Requires a path to be passed after it"]
        flag_range: SourceSpan,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    Cli(#[from] cli::Error),
    #[error(transparent)]
    Inference(#[from] titanrepo_repository::inference::Error),
    #[error("failed to execute local titan process")]
    LocalTitanProcess(#[source] std::io::Error),
    #[error("failed to resolve local titan path: {0}")]
    LocalTitanPath(String),
    #[error("failed to find npx: {0}")]
    Which(#[from] which::Error),
    #[error("failed to execute titan via npx")]
    NpxTitanProcess(#[source] std::io::Error),
    #[error("failed to resolve repository root: {0}")]
    RepoRootPath(AbsoluteSystemPathBuf),
    #[error(transparent)]
    Path(#[from] titanpath::PathError),
}

/// Attempts to run correct titan by finding nearest package.json,
/// then finding local titan installation. If the current binary is the
/// local titan installation, then we run current titan. Otherwise we
/// kick over to the local titan installation.
///
/// # Arguments
///
/// * `titan_state`: state for current execution
///
/// returns: Result<i32, Error>
fn run_correct_titan(
    repo_state: RepoState,
    shim_args: ShimArgs,
    subscriber: &TitanSubscriber,
    ui: ColorConfig,
) -> Result<i32, Error> {
    if let Some(titan_state) = LocalTitanState::infer(&repo_state.root) {
        try_check_for_updates(&shim_args, titan_state.version());

        if titan_state.local_is_self() {
            env::set_var(
                cli::INVOCATION_DIR_ENV_VAR,
                shim_args.invocation_dir.as_path(),
            );
            debug!("Currently running titan is local titan.");
            Ok(cli::run(Some(repo_state), subscriber, ui)?)
        } else {
            spawn_local_titan(&repo_state, titan_state, shim_args)
        }
    } else if let Some(local_config) = LocalTitanConfig::infer(&repo_state) {
        debug!(
            "Found configuration for titan version {}",
            local_config.titan_version()
        );
        spawn_npx_titan(&repo_state, local_config.titan_version(), shim_args)
    } else {
        let version = get_version();
        try_check_for_updates(&shim_args, version);
        // cli::run checks for this env var, rather than an arg, so that we can support
        // calling old versions without passing unknown flags.
        env::set_var(
            cli::INVOCATION_DIR_ENV_VAR,
            shim_args.invocation_dir.as_path(),
        );
        debug!("Running command as global titan");
        let should_warn_on_global = env::var(TITAN_GLOBAL_WARNING_DISABLED)
            .map_or(true, |disable| !matches!(disable.as_str(), "1" | "true"));
        if should_warn_on_global {
            warn!("No locally installed `titan` found. Using version: {version}.");
        }
        Ok(cli::run(Some(repo_state), subscriber, ui)?)
    }
}

fn spawn_local_titan(
    repo_state: &RepoState,
    local_titan_state: LocalTitanState,
    mut shim_args: ShimArgs,
) -> Result<i32, Error> {
    let local_titan_path = fs_canonicalize(local_titan_state.binary()).map_err(|_| {
        Error::LocalTitanPath(local_titan_state.binary().to_string_lossy().to_string())
    })?;
    debug!(
        "Running local titan binary in {}\n",
        local_titan_path.display()
    );
    let cwd = fs_canonicalize(&repo_state.root)
        .map_err(|_| Error::RepoRootPath(repo_state.root.clone()))?;

    let raw_args = modify_args_for_local(&mut shim_args, repo_state, local_titan_state.version());

    // We spawn a process that executes the local titan
    // that we've found in node_modules/.bin/titan.
    let mut command = process::Command::new(local_titan_path);
    command
        .args(&raw_args)
        // rather than passing an argument that local titan might not understand, set
        // an environment variable that can be optionally used
        .env(
            cli::INVOCATION_DIR_ENV_VAR,
            shim_args.invocation_dir.as_path(),
        )
        .current_dir(cwd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    spawn_child_titan(command, Error::LocalTitanProcess)
}

fn spawn_npx_titan(
    repo_state: &RepoState,
    titan_version: &str,
    mut shim_args: ShimArgs,
) -> Result<i32, Error> {
    debug!("Running titan@{titan_version} via npx");
    let npx_path = which("npx")?;
    let cwd = fs_canonicalize(&repo_state.root)
        .map_err(|_| Error::RepoRootPath(repo_state.root.clone()))?;

    let raw_args = modify_args_for_local(&mut shim_args, repo_state, titan_version);

    let mut command = process::Command::new(npx_path);
    command.arg("-y");
    command.arg(format!("titan@{titan_version}"));

    // rather than passing an argument that local titan might not understand, set
    // an environment variable that can be optionally used
    command
        .args(&raw_args)
        .env(
            cli::INVOCATION_DIR_ENV_VAR,
            shim_args.invocation_dir.as_path(),
        )
        .current_dir(cwd)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    spawn_child_titan(command, Error::NpxTitanProcess)
}

fn modify_args_for_local(
    shim_args: &mut ShimArgs,
    repo_state: &RepoState,
    local_version: &str,
) -> Vec<String> {
    let supports_skip_infer_and_single_package = titan_version_has_shim(local_version);
    let already_has_single_package_flag = shim_args
        .remaining_titan_args
        .contains(&"--single-package".to_string());
    let should_add_single_package_flag = repo_state.mode == RepoMode::SinglePackage
        && !already_has_single_package_flag
        && supports_skip_infer_and_single_package;

    debug!(
        "supports_skip_infer_and_single_package {:?}",
        supports_skip_infer_and_single_package
    );

    let mut raw_args: Vec<_> = if supports_skip_infer_and_single_package {
        vec!["--skip-infer".to_string()]
    } else {
        Vec::new()
    };

    raw_args.append(&mut shim_args.remaining_titan_args);

    // We add this flag after the raw args to avoid accidentally passing it
    // as a global flag instead of as a run flag.
    if should_add_single_package_flag {
        raw_args.push("--single-package".to_string());
    }

    raw_args.push("--".to_string());
    raw_args.append(&mut shim_args.forwarded_args);

    raw_args
}

fn spawn_child_titan(
    command: process::Command,
    err: fn(std::io::Error) -> Error,
) -> Result<i32, Error> {
    let child = spawn_child(command).map_err(err)?;

    let exit_status = child.wait().map_err(err)?;
    let exit_code = exit_status.code().unwrap_or_else(|| {
        debug!("child titan failed to report exit code");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            let signal = exit_status.signal();
            let core_dumped = exit_status.core_dumped();
            debug!(
                "child titan caught signal {:?}. Core dumped? {}",
                signal, core_dumped
            );
        }
        2
    });

    Ok(exit_code)
}

/// Checks for `TITAN_BINARY_PATH` variable. If it is set,
/// we do not try to find local titan, we simply run the command as
/// the current binary. This is due to legacy behavior of `TITAN_BINARY_PATH`
/// that lets users dynamically set the path of the titan binary. Because
/// that conflicts with finding a local titan installation and
/// executing that binary, these two features are fundamentally incompatible.
fn is_titan_binary_path_set() -> bool {
    env::var("TITAN_BINARY_PATH").is_ok()
}

fn try_check_for_updates(args: &ShimArgs, current_version: &str) {
    if args.should_check_for_update() {
        // custom footer for update message
        let footer = format!(
            "Follow {username} for updates: {url}",
            username = "@titanrepo".gradient([RGB::new(0, 153, 247), RGB::new(241, 23, 18)]),
            url = "https://x.com/titanrepo"
        );

        let interval = if args.force_update_check {
            // force update check
            Some(Duration::ZERO)
        } else {
            // use default (24 hours)
            None
        };
        // check for updates
        let _ = display_update_check(
            "titan",
            "https://github.com/khulnasoft/titanrepo",
            Some(&footer),
            current_version,
            // use default for timeout (800ms)
            None,
            interval,
        );
    }
}

pub fn run() -> Result<i32, Error> {
    let args = ShimArgs::parse()?;
    let color_config = args.color_config();
    if color_config.should_strip_ansi {
        // Let's not crash just because we failed to set up the hook
        let _ = miette::set_hook(Box::new(|_| {
            Box::new(
                miette::MietteHandlerOpts::new()
                    .color(false)
                    .unicode(false)
                    .build(),
            )
        }));
    }
    let subscriber = TitanSubscriber::new_with_verbosity(args.verbosity, &color_config);

    debug!("Global titan version: {}", get_version());

    // If skip_infer is passed, we're probably running local titan with
    // global titan having handled the inference. We can run without any
    // concerns.
    if args.skip_infer {
        return Ok(cli::run(None, &subscriber, color_config)?);
    }

    // If the TITAN_BINARY_PATH is set, we do inference but we do not use
    // it to execute local titan. We simply use it to set the `--single-package`
    // and `--cwd` flags.
    if is_titan_binary_path_set() {
        let repo_state = RepoState::infer(&args.cwd)?;
        debug!("Repository Root: {}", repo_state.root);
        return Ok(cli::run(Some(repo_state), &subscriber, color_config)?);
    }

    match RepoState::infer(&args.cwd) {
        Ok(repo_state) => {
            debug!("Repository Root: {}", repo_state.root);
            run_correct_titan(repo_state, args, &subscriber, color_config)
        }
        Err(err) => {
            // If we cannot infer, we still run global titan. This allows for global
            // commands like login/logout/link/unlink to still work
            debug!("Repository inference failed: {}", err);
            debug!("Running command as global titan");
            Ok(cli::run(None, &subscriber, color_config)?)
        }
    }
}
