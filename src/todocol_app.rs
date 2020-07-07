use config::Config;
use log::{debug, error, warn};
use std::{path::PathBuf, str::FromStr};

use todocol::TodoCollector;

pub fn swap_env(input: &str) -> String {
    if input.contains('$') {
        match shellexpand::env(input) {
            Ok(c) => c.to_string(),
            Err(err) => {
                error!("Can not expand env variables in {} directory: {}", &input, err);
                String::new()
            }
        }
    } else {
        input.to_owned()
    }
}

fn todo_collector_set_config(todo_collector: &mut TodoCollector, config: &Config) -> anyhow::Result<()> {
    let prefixes = config.get::<Vec<String>>("prefix")?;
    let outfile_name = config.get::<String>("outfile.name")?;
    let outfile_format = config.get::<String>("outfile.format")?;
    let ignore = config.get::<Vec<String>>("ignore").unwrap_or_else(|_| Vec::new());

    debug!("prefixes: {:#?}", prefixes);
    debug!("outfile: {} in {} format", outfile_name, outfile_format);
    debug!("ignore list: {:#?}", ignore);

    todo_collector.set_prefixes(prefixes);
    todo_collector.set_result_file_name(&outfile_name);
    todo_collector.set_result_file_format(&outfile_format);
    todo_collector.set_ignore(ignore);

    todo_collector.add_ignore(".vscode");
    todo_collector.add_ignore(".git");
    todo_collector.add_ignore(".gitignore");
    todo_collector.add_ignore(".gitmodules");

    Ok(())
}

fn project_dirs(workspace_dir: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let workspace_path = match PathBuf::from_str(workspace_dir) {
        Ok(w) => w,
        Err(_err) => {
            error!("'{}' is invalid workspace path", workspace_dir);
            return result
        }
    };

    let entry_iter = match workspace_path.read_dir() {
        Ok(ei) => ei,
        Err(err) => {
            warn!("Can not read dir {}: {}", workspace_dir, err);
            return result
        }
    };

    for entry in entry_iter {
        if let Ok(e) = entry {
            let path = e.path();
            if path.is_dir() {
                result.push(path)
            }
        }
    }

    result
}

pub fn run_app_project(config: &Config, project_dir: &str) -> anyhow::Result<()> {
    let project_dir = swap_env(project_dir);
    let mut todo_collector = TodoCollector::new();
    todo_collector_set_config(&mut todo_collector, config)?;

    todo_collector.collect_from_project(&project_dir)?;

    Ok(())
}

pub fn run_app_workspace(config: &Config, workspace_dir: &str) -> anyhow::Result<()> {
    let workspace_dir = swap_env(workspace_dir);
    let mut todo_collector = TodoCollector::new();
    todo_collector_set_config(&mut todo_collector, config)?;

    for pd in project_dirs(&workspace_dir) {
        if let Err(err) = todo_collector.collect_from_project(&pd) {
            error!("Collecting project dir failed: {}", err);
        }
    }

    Ok(())
}

pub fn run_app_workspaces(config: &Config) -> anyhow::Result<()> {
    let mut workspaces = config.get::<Vec<String>>("workspaces")?;
    workspaces = workspaces.iter().map(|w| swap_env(w)).filter(|w| !w.is_empty()).collect();
    debug!("workspaces: {:#?}", workspaces);

    let mut todo_collector = TodoCollector::new();
    todo_collector_set_config(&mut todo_collector, config)?;

    let mut project_paths = Vec::new();
    for w in &workspaces {
        project_paths.extend(project_dirs(w))
    }

    for pd in project_paths {
        if let Err(err) = todo_collector.collect_from_project(&pd) {
            error!("Collecting project dir failed: {}", err);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn env_swap() {
        let p1 = "$HOME/pictures";
        let p2 = "/home/$USER/pictures";
        assert_eq!(shellexpand::env(p1).unwrap(), shellexpand::env(p2).unwrap());
    }
}
