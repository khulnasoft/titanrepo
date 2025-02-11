mod change_detector;
pub mod filter;
mod simple_glob;
pub mod target_selector;

use std::collections::HashMap;

use filter::{FilterResolver, PackageInference};
use titanpath::AbsoluteSystemPath;
use titanrepo_repository::{
    change_mapper::PackageInclusionReason,
    package_graph::{PackageGraph, PackageName},
};
use titanrepo_scm::SCM;

pub use crate::run::scope::filter::ResolutionError;
use crate::{opts::ScopeOpts, titan_json::TitanJson};

#[tracing::instrument(skip(opts, pkg_graph, scm))]
pub fn resolve_packages(
    opts: &ScopeOpts,
    titan_root: &AbsoluteSystemPath,
    pkg_graph: &PackageGraph,
    scm: &SCM,
    root_titan_json: &TitanJson,
) -> Result<(HashMap<PackageName, PackageInclusionReason>, bool), ResolutionError> {
    let pkg_inference = opts.pkg_inference_root.as_ref().map(|pkg_inference_path| {
        PackageInference::calculate(titan_root, pkg_inference_path, pkg_graph)
    });

    FilterResolver::new(
        opts,
        pkg_graph,
        titan_root,
        pkg_inference,
        scm,
        root_titan_json,
    )?
    .resolve(&opts.affected_range, &opts.get_filters())
}
