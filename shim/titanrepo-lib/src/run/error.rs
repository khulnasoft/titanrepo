use miette::Diagnostic;
use thiserror::Error;
use titanrepo_repository::package_graph;
use titanrepo_ui::tui;

use super::graph_visualizer;
use crate::{
    config, daemon, engine,
    engine::ValidateError,
    opts,
    run::{global_hash, scope},
    task_graph, task_hash,
};

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("invalid task configuration")]
    EngineValidation(#[related] Vec<ValidateError>),
    #[error(transparent)]
    Graph(#[from] graph_visualizer::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Builder(#[from] engine::BuilderError),
    #[error(transparent)]
    Env(#[from] titanrepo_env::Error),
    #[error(transparent)]
    Opts(#[from] opts::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    PackageJson(#[from] titanrepo_repository::package_json::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    PackageManager(#[from] titanrepo_repository::package_manager::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Config(#[from] config::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    PackageGraphBuilder(#[from] package_graph::builder::Error),
    #[error(transparent)]
    DaemonConnector(#[from] daemon::DaemonConnectorError),
    #[error(transparent)]
    Cache(#[from] titanrepo_cache::CacheError),
    #[error(transparent)]
    Path(#[from] titanpath::PathError),
    #[error(transparent)]
    Scope(#[from] scope::ResolutionError),
    #[error(transparent)]
    GlobalHash(#[from] global_hash::Error),
    #[error(transparent)]
    TaskHash(#[from] task_hash::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Visitor(#[from] task_graph::VisitorError),
    #[error("error registering signal handler: {0}")]
    SignalHandler(std::io::Error),
    #[error(transparent)]
    Daemon(#[from] daemon::DaemonError),
    #[error(transparent)]
    UI(#[from] titanrepo_ui::Error),
    #[error(transparent)]
    Tui(#[from] tui::Error),
}
