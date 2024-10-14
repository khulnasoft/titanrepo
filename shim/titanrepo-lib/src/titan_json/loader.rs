use std::collections::HashMap;

use tracing::debug;
use titanpath::{AbsoluteSystemPath, AbsoluteSystemPathBuf};
use titanrepo_errors::Spanned;
use titanrepo_repository::{
    package_graph::{PackageInfo, PackageName},
    package_json::PackageJson,
};

use super::{Pipeline, RawTaskDefinition, TitanJson, CONFIG_FILE};
use crate::{
    cli::EnvMode,
    config::Error,
    run::{task_access::TASK_ACCESS_CONFIG_PATH, task_id::TaskName},
};

/// Structure for loading TitanJson structures.
/// Depending on the strategy used, TitanJson might not correspond to
/// `titan.json` file.
#[derive(Debug, Clone)]
pub struct TitanJsonLoader {
    repo_root: AbsoluteSystemPathBuf,
    cache: HashMap<PackageName, TitanJson>,
    strategy: Strategy,
}

#[derive(Debug, Clone)]
enum Strategy {
    SinglePackage {
        root_titan_json: AbsoluteSystemPathBuf,
        package_json: PackageJson,
    },
    Workspace {
        // Map of package names to their package specific titan.json
        packages: HashMap<PackageName, AbsoluteSystemPathBuf>,
    },
    WorkspaceNoTitanJson {
        // Map of package names to their scripts
        packages: HashMap<PackageName, Vec<String>>,
    },
    TaskAccess {
        root_titan_json: AbsoluteSystemPathBuf,
        package_json: PackageJson,
    },
    Noop,
}

impl TitanJsonLoader {
    /// Create a loader that will load titan.json files throughout the workspace
    pub fn workspace<'a>(
        repo_root: AbsoluteSystemPathBuf,
        root_titan_json_path: AbsoluteSystemPathBuf,
        packages: impl Iterator<Item = (&'a PackageName, &'a PackageInfo)>,
    ) -> Self {
        let packages = package_titan_jsons(&repo_root, root_titan_json_path, packages);
        Self {
            repo_root,
            cache: HashMap::new(),
            strategy: Strategy::Workspace { packages },
        }
    }

    /// Create a loader that will construct titan.json structures based on
    /// workspace `package.json`s.
    pub fn workspace_no_titan_json<'a>(
        repo_root: AbsoluteSystemPathBuf,
        packages: impl Iterator<Item = (&'a PackageName, &'a PackageInfo)>,
    ) -> Self {
        let packages = workspace_package_scripts(packages);
        Self {
            repo_root,
            cache: HashMap::new(),
            strategy: Strategy::WorkspaceNoTitanJson { packages },
        }
    }

    /// Create a loader that will load a root titan.json or synthesize one if
    /// the file doesn't exist
    pub fn single_package(
        repo_root: AbsoluteSystemPathBuf,
        root_titan_json: AbsoluteSystemPathBuf,
        package_json: PackageJson,
    ) -> Self {
        Self {
            repo_root,
            cache: HashMap::new(),
            strategy: Strategy::SinglePackage {
                root_titan_json,
                package_json,
            },
        }
    }

    /// Create a loader that will load a root titan.json or synthesize one if
    /// the file doesn't exist
    pub fn task_access(
        repo_root: AbsoluteSystemPathBuf,
        root_titan_json: AbsoluteSystemPathBuf,
        package_json: PackageJson,
    ) -> Self {
        Self {
            repo_root,
            cache: HashMap::new(),
            strategy: Strategy::TaskAccess {
                root_titan_json,
                package_json,
            },
        }
    }

    /// Create a loader that will only return provided titan.jsons and will
    /// never hit the file system.
    /// Primarily intended for testing
    pub fn noop(titan_jsons: HashMap<PackageName, TitanJson>) -> Self {
        Self {
            // This never gets read from so we populate it with
            repo_root: AbsoluteSystemPath::new(if cfg!(windows) { "C:\\" } else { "/" })
                .expect("wasn't able to create absolute system path")
                .to_owned(),
            cache: titan_jsons,
            strategy: Strategy::Noop,
        }
    }

    /// Load a titan.json for a given package
    pub fn load<'a>(&'a mut self, package: &PackageName) -> Result<&'a TitanJson, Error> {
        if !self.cache.contains_key(package) {
            let titan_json = self.uncached_load(package)?;
            self.cache.insert(package.clone(), titan_json);
        }
        Ok(self
            .cache
            .get(package)
            .expect("just inserted value for this key"))
    }

    fn uncached_load(&self, package: &PackageName) -> Result<TitanJson, Error> {
        match &self.strategy {
            Strategy::SinglePackage {
                package_json,
                root_titan_json,
            } => {
                if !matches!(package, PackageName::Root) {
                    Err(Error::InvalidTitanJsonLoad(package.clone()))
                } else {
                    load_from_root_package_json(&self.repo_root, root_titan_json, package_json)
                }
            }
            Strategy::Workspace { packages } => {
                let path = packages.get(package).ok_or_else(|| Error::NoTitanJSON)?;
                load_from_file(&self.repo_root, path)
            }
            Strategy::WorkspaceNoTitanJson { packages } => {
                let script_names = packages.get(package).ok_or(Error::NoTitanJSON)?;
                if matches!(package, PackageName::Root) {
                    root_titan_json_from_scripts(script_names)
                } else {
                    workspace_titan_json_from_scripts(script_names)
                }
            }
            Strategy::TaskAccess {
                package_json,
                root_titan_json,
            } => {
                if !matches!(package, PackageName::Root) {
                    Err(Error::InvalidTitanJsonLoad(package.clone()))
                } else {
                    load_task_access_trace_titan_json(
                        &self.repo_root,
                        root_titan_json,
                        package_json,
                    )
                }
            }
            Strategy::Noop => Err(Error::NoTitanJSON),
        }
    }
}

/// Map all packages in the package graph to their titan.json path
fn package_titan_jsons<'a>(
    repo_root: &AbsoluteSystemPath,
    root_titan_json_path: AbsoluteSystemPathBuf,
    packages: impl Iterator<Item = (&'a PackageName, &'a PackageInfo)>,
) -> HashMap<PackageName, AbsoluteSystemPathBuf> {
    let mut package_titan_jsons = HashMap::new();
    package_titan_jsons.insert(PackageName::Root, root_titan_json_path);
    package_titan_jsons.extend(packages.filter_map(|(pkg, info)| {
        if pkg == &PackageName::Root {
            None
        } else {
            Some((
                pkg.clone(),
                repo_root
                    .resolve(info.package_path())
                    .join_component(CONFIG_FILE),
            ))
        }
    }));
    package_titan_jsons
}

/// Map all packages in the package graph to their scripts
fn workspace_package_scripts<'a>(
    packages: impl Iterator<Item = (&'a PackageName, &'a PackageInfo)>,
) -> HashMap<PackageName, Vec<String>> {
    packages
        .map(|(pkg, info)| {
            (
                pkg.clone(),
                info.package_json.scripts.keys().cloned().collect(),
            )
        })
        .collect()
}

fn load_from_file(
    repo_root: &AbsoluteSystemPath,
    titan_json_path: &AbsoluteSystemPath,
) -> Result<TitanJson, Error> {
    match TitanJson::read(repo_root, titan_json_path) {
        // If the file didn't exist, throw a custom error here instead of propagating
        Err(Error::Io(_)) => Err(Error::NoTitanJSON),
        // There was an error, and we don't have any chance of recovering
        // because we aren't synthesizing anything
        Err(e) => Err(e),
        // We're not synthesizing anything and there was no error, we're done
        Ok(titan) => Ok(titan),
    }
}

fn load_from_root_package_json(
    repo_root: &AbsoluteSystemPath,
    titan_json_path: &AbsoluteSystemPath,
    root_package_json: &PackageJson,
) -> Result<TitanJson, Error> {
    let mut titan_json = match TitanJson::read(repo_root, titan_json_path) {
        // we're synthesizing, but we have a starting point
        // Note: this will have to change to support task inference in a monorepo
        // for now, we're going to error on any "root" tasks and turn non-root tasks into root
        // tasks
        Ok(mut titan_json) => {
            let mut pipeline = Pipeline::default();
            for (task_name, task_definition) in titan_json.tasks {
                if task_name.is_package_task() {
                    let (span, text) = task_definition.span_and_text("titan.json");

                    return Err(Error::PackageTaskInSinglePackageMode {
                        task_id: task_name.to_string(),
                        span,
                        text,
                    });
                }

                pipeline.insert(task_name.into_root_task(), task_definition);
            }

            titan_json.tasks = pipeline;

            titan_json
        }
        // titan.json doesn't exist, but we're going try to synthesize something
        Err(Error::Io(_)) => TitanJson::default(),
        // some other happened, we can't recover
        Err(e) => {
            return Err(e);
        }
    };

    // TODO: Add location info from package.json
    for script_name in root_package_json.scripts.keys() {
        let task_name = TaskName::from(script_name.as_str());
        if !titan_json.has_task(&task_name) {
            let task_name = task_name.into_root_task();
            // Explicitly set cache to Some(false) in this definition
            // so we can pretend it was set on purpose. That way it
            // won't get clobbered by the merge function.
            titan_json.tasks.insert(
                task_name,
                Spanned::new(RawTaskDefinition {
                    cache: Some(Spanned::new(false)),
                    ..RawTaskDefinition::default()
                }),
            );
        }
    }

    Ok(titan_json)
}

fn root_titan_json_from_scripts(scripts: &[String]) -> Result<TitanJson, Error> {
    let mut titan_json = TitanJson {
        ..Default::default()
    };
    for script in scripts {
        let task_name = TaskName::from(script.as_str()).into_root_task();
        titan_json.tasks.insert(
            task_name,
            Spanned::new(RawTaskDefinition {
                cache: Some(Spanned::new(false)),
                env_mode: Some(EnvMode::Loose),
                ..Default::default()
            }),
        );
    }
    Ok(titan_json)
}

fn workspace_titan_json_from_scripts(scripts: &[String]) -> Result<TitanJson, Error> {
    let mut titan_json = TitanJson {
        extends: Spanned::new(vec!["//".to_owned()]),
        ..Default::default()
    };
    for script in scripts {
        let task_name = TaskName::from(script.clone());
        titan_json.tasks.insert(
            task_name,
            Spanned::new(RawTaskDefinition {
                cache: Some(Spanned::new(false)),
                env_mode: Some(EnvMode::Loose),
                ..Default::default()
            }),
        );
    }
    Ok(titan_json)
}

fn load_task_access_trace_titan_json(
    repo_root: &AbsoluteSystemPath,
    titan_json_path: &AbsoluteSystemPath,
    root_package_json: &PackageJson,
) -> Result<TitanJson, Error> {
    let trace_json_path = repo_root.join_components(&TASK_ACCESS_CONFIG_PATH);
    let titan_from_trace = TitanJson::read(repo_root, &trace_json_path);

    // check the zero config case (titan trace file, but no titan.json file)
    if let Ok(titan_from_trace) = titan_from_trace {
        if !titan_json_path.exists() {
            debug!("Using titan.json synthesized from trace file");
            return Ok(titan_from_trace);
        }
    }
    load_from_root_package_json(repo_root, titan_json_path, root_package_json)
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, fs};

    use anyhow::Result;
    use tempfile::tempdir;
    use test_case::test_case;

    use super::*;
    use crate::{task_graph::TaskDefinition, titan_json::CONFIG_FILE};

    #[test_case(r"{}", TitanJson::default() ; "empty")]
    #[test_case(r#"{ "globalDependencies": ["tsconfig.json", "jest.config.js"] }"#,
        TitanJson {
            global_deps: vec!["jest.config.js".to_string(), "tsconfig.json".to_string()],
            ..TitanJson::default()
        }
    ; "global dependencies (sorted)")]
    #[test_case(r#"{ "globalPassThroughEnv": ["GITHUB_TOKEN", "AWS_SECRET_KEY"] }"#,
        TitanJson {
            global_pass_through_env: Some(vec!["AWS_SECRET_KEY".to_string(), "GITHUB_TOKEN".to_string()]),
            ..TitanJson::default()
        }
    )]
    #[test_case(r#"{ "//": "A comment"}"#, TitanJson::default() ; "faux comment")]
    #[test_case(r#"{ "//": "A comment", "//": "Another comment" }"#, TitanJson::default() ; "two faux comments")]
    fn test_get_root_titan_no_synthesizing(
        titan_json_content: &str,
        expected_titan_json: TitanJson,
    ) -> Result<()> {
        let root_dir = tempdir()?;
        let repo_root = AbsoluteSystemPath::from_std_path(root_dir.path())?;
        let root_titan_json = repo_root.join_component("titan.json");
        fs::write(&root_titan_json, titan_json_content)?;
        let mut loader = TitanJsonLoader {
            repo_root: repo_root.to_owned(),
            cache: HashMap::new(),
            strategy: Strategy::Workspace {
                packages: vec![(PackageName::Root, root_titan_json)]
                    .into_iter()
                    .collect(),
            },
        };

        let mut titan_json = loader.load(&PackageName::Root)?.clone();

        titan_json.text = None;
        titan_json.path = None;
        assert_eq!(titan_json, expected_titan_json);

        Ok(())
    }

    #[test_case(
        None,
        PackageJson {
             scripts: [("build".to_string(), Spanned::new("echo build".to_string()))].into_iter().collect(),
             ..PackageJson::default()
        },
        TitanJson {
            tasks: Pipeline([(
                "//#build".into(),
                Spanned::new(RawTaskDefinition {
                    cache: Some(Spanned::new(false)),
                    ..RawTaskDefinition::default()
                })
              )].into_iter().collect()
            ),
            ..TitanJson::default()
        }
    )]
    #[test_case(
        Some(r#"{
            "tasks": {
                "build": {
                    "cache": true
                }
            }
        }"#),
        PackageJson {
             scripts: [("test".to_string(), Spanned::new("echo test".to_string()))].into_iter().collect(),
             ..PackageJson::default()
        },
        TitanJson {
            tasks: Pipeline([(
                "//#build".into(),
                Spanned::new(RawTaskDefinition {
                    cache: Some(Spanned::new(true).with_range(81..85)),
                    ..RawTaskDefinition::default()
                }).with_range(50..103)
            ),
            (
                "//#test".into(),
                Spanned::new(RawTaskDefinition {
                     cache: Some(Spanned::new(false)),
                    ..RawTaskDefinition::default()
                })
            )].into_iter().collect()),
            ..TitanJson::default()
        }
    )]
    fn test_get_root_titan_with_synthesizing(
        titan_json_content: Option<&str>,
        root_package_json: PackageJson,
        expected_titan_json: TitanJson,
    ) -> Result<()> {
        let root_dir = tempdir()?;
        let repo_root = AbsoluteSystemPath::from_std_path(root_dir.path())?;
        let root_titan_json = repo_root.join_component(CONFIG_FILE);

        if let Some(content) = titan_json_content {
            fs::write(&root_titan_json, content)?;
        }

        let mut loader = TitanJsonLoader::single_package(
            repo_root.to_owned(),
            root_titan_json,
            root_package_json,
        );
        let mut titan_json = loader.load(&PackageName::Root)?.clone();
        titan_json.text = None;
        titan_json.path = None;
        for (_, task_definition) in titan_json.tasks.iter_mut() {
            task_definition.path = None;
            task_definition.text = None;
        }
        assert_eq!(titan_json, expected_titan_json);

        Ok(())
    }

    #[test_case(
        Some(r#"{ "tasks": {"//#build": {"env": ["SPECIAL_VAR"]}} }"#),
        Some(r#"{ "tasks": {"build": {"env": ["EXPLICIT_VAR"]}} }"#),
        TaskDefinition { env: vec!["EXPLICIT_VAR".to_string()], .. Default::default() }
    ; "both present")]
    #[test_case(
        None,
        Some(r#"{ "tasks": {"build": {"env": ["EXPLICIT_VAR"]}} }"#),
        TaskDefinition { env: vec!["EXPLICIT_VAR".to_string()], .. Default::default() }
    ; "no trace")]
    #[test_case(
        Some(r#"{ "tasks": {"//#build": {"env": ["SPECIAL_VAR"]}} }"#),
        None,
        TaskDefinition { env: vec!["SPECIAL_VAR".to_string()], .. Default::default() }
    ; "no titan.json")]
    #[test_case(
        None,
        None,
        TaskDefinition { cache: false, .. Default::default() }
    ; "both missing")]
    fn test_task_access_loading(
        trace_contents: Option<&str>,
        titan_json_content: Option<&str>,
        expected_root_build: TaskDefinition,
    ) -> Result<()> {
        let root_dir = tempdir()?;
        let repo_root = AbsoluteSystemPath::from_std_path(root_dir.path())?;
        let root_titan_json = repo_root.join_component(CONFIG_FILE);

        if let Some(content) = titan_json_content {
            root_titan_json.create_with_contents(content.as_bytes())?;
        }
        if let Some(content) = trace_contents {
            let trace_path = repo_root.join_components(&TASK_ACCESS_CONFIG_PATH);
            trace_path.ensure_dir()?;
            trace_path.create_with_contents(content.as_bytes())?;
        }

        let mut scripts = BTreeMap::new();
        scripts.insert("build".into(), Spanned::new("echo building".into()));
        let root_package_json = PackageJson {
            scripts,
            ..Default::default()
        };

        let mut loader =
            TitanJsonLoader::task_access(repo_root.to_owned(), root_titan_json, root_package_json);
        let titan_json = loader.load(&PackageName::Root)?;
        let root_build = titan_json
            .tasks
            .get(&TaskName::from("//#build"))
            .expect("root build should always exist")
            .as_inner();

        assert_eq!(
            expected_root_build,
            TaskDefinition::try_from(root_build.clone())?
        );

        Ok(())
    }

    #[test]
    fn test_single_package_loading_non_root() {
        let junk_path = AbsoluteSystemPath::new(if cfg!(windows) {
            "C:\\never\\loaded"
        } else {
            "/never/loaded"
        })
        .unwrap();
        let non_root = PackageName::from("some-pkg");
        let single_loader = TitanJsonLoader::single_package(
            junk_path.to_owned(),
            junk_path.to_owned(),
            PackageJson::default(),
        );
        let task_access_loader = TitanJsonLoader::task_access(
            junk_path.to_owned(),
            junk_path.to_owned(),
            PackageJson::default(),
        );

        for mut loader in [single_loader, task_access_loader] {
            let result = loader.load(&non_root);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(err, Error::InvalidTitanJsonLoad(_)),
                "expected {err} to be no titan json"
            );
        }
    }

    #[test]
    fn test_workspace_titan_json_loading() {
        let root_dir = tempdir().unwrap();
        let repo_root = AbsoluteSystemPath::from_std_path(root_dir.path()).unwrap();
        let a_titan_json = repo_root.join_components(&["packages", "a", "titan.json"]);
        a_titan_json.ensure_dir().unwrap();
        let packages = vec![(PackageName::from("a"), a_titan_json.clone())]
            .into_iter()
            .collect();

        let mut loader = TitanJsonLoader {
            repo_root: repo_root.to_owned(),
            cache: HashMap::new(),
            strategy: Strategy::Workspace { packages },
        };
        let result = loader.load(&PackageName::from("a"));
        assert!(
            matches!(result.unwrap_err(), Error::NoTitanJSON),
            "expected parsing to fail with missing titan.json"
        );

        a_titan_json
            .create_with_contents(r#"{"tasks": {"build": {}}}"#)
            .unwrap();

        let titan_json = loader.load(&PackageName::from("a")).unwrap();
        assert_eq!(titan_json.tasks.len(), 1);
    }

    #[test]
    fn test_titan_json_caching() {
        let root_dir = tempdir().unwrap();
        let repo_root = AbsoluteSystemPath::from_std_path(root_dir.path()).unwrap();
        let a_titan_json = repo_root.join_components(&["packages", "a", "titan.json"]);
        a_titan_json.ensure_dir().unwrap();
        let packages = vec![(PackageName::from("a"), a_titan_json.clone())]
            .into_iter()
            .collect();

        let mut loader = TitanJsonLoader {
            repo_root: repo_root.to_owned(),
            cache: HashMap::new(),
            strategy: Strategy::Workspace { packages },
        };
        a_titan_json
            .create_with_contents(r#"{"tasks": {"build": {}}}"#)
            .unwrap();

        let titan_json = loader.load(&PackageName::from("a")).unwrap();
        assert_eq!(titan_json.tasks.len(), 1);
        a_titan_json.remove().unwrap();
        assert!(loader.load(&PackageName::from("a")).is_ok());
    }

    #[test]
    fn test_no_titan_json() {
        let root_dir = tempdir().unwrap();
        let repo_root = AbsoluteSystemPath::from_std_path(root_dir.path()).unwrap();
        let packages = vec![
            (
                PackageName::Root,
                vec!["build".to_owned(), "lint".to_owned(), "test".to_owned()],
            ),
            (
                PackageName::from("pkg-a"),
                vec!["build".to_owned(), "lint".to_owned(), "special".to_owned()],
            ),
        ]
        .into_iter()
        .collect();

        let mut loader = TitanJsonLoader {
            repo_root: repo_root.to_owned(),
            cache: HashMap::new(),
            strategy: Strategy::WorkspaceNoTitanJson { packages },
        };

        {
            let root_json = loader.load(&PackageName::Root).unwrap();
            for task_name in ["//#build", "//#lint", "//#test"] {
                if let Some(def) = root_json.tasks.get(&TaskName::from(task_name)) {
                    assert_eq!(
                        def.cache.as_ref().map(|cache| *cache.as_inner()),
                        Some(false)
                    );
                } else {
                    panic!("didn't find {task_name}");
                }
            }
        }

        {
            let pkg_a_json = loader.load(&PackageName::from("pkg-a")).unwrap();
            for task_name in ["build", "lint", "special"] {
                if let Some(def) = pkg_a_json.tasks.get(&TaskName::from(task_name)) {
                    assert_eq!(
                        def.cache.as_ref().map(|cache| *cache.as_inner()),
                        Some(false)
                    );
                } else {
                    panic!("didn't find {task_name}");
                }
            }
        }
        // Should get no titan.json error if package wasn't declared
        let goose_err = loader.load(&PackageName::from("goose")).unwrap_err();
        assert!(matches!(goose_err, Error::NoTitanJSON));
    }
}
