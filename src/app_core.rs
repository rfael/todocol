use config::Config;
use log::{debug, error};

use todocol::TodoCollector;

pub fn run_app(settings: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut worspaces = settings.get::<Vec<String>>("workspace")?;
    let prefixes = settings.get::<Vec<String>>("prefix")?;
    let comment_symbols = settings.get::<Vec<String>>("comment_symbol")?;
    let outfile = settings.get_table("outfile")?;
    let outfile_name = outfile.get("name").expect("Outfile name not set").clone().into_str()?;
    let outfile_format = outfile.get("format").expect("Outfile type not set").clone().into_str()?;
    let ignored = settings.get::<Vec<String>>("ignored").unwrap_or_else(|_| Vec::new());

    let swap_envs = |w: &String| {
        let mut ws = String::from(w);
        if ws.contains('$') {
            ws = match shellexpand::env(&ws) {
                Ok(c) => c.to_string(),
                Err(err) => {
                    error!("Can not expand env variables in {} directory: {}", &ws, err);
                    String::new()
                }
            }
        }
        ws
    };

    worspaces = worspaces.iter().map(swap_envs).rev().collect();

    debug!("workspaces: {:#?}", worspaces);
    debug!("prefixes: {:#?}", prefixes);
    debug!("comment_symbols: {:#?}", comment_symbols);
    debug!("outfile: {} in {} format", outfile_name, outfile_format);

    let mut todo_collector = TodoCollector::new();
    todo_collector.set_prefixes(prefixes);
    todo_collector.set_comment_symbols(comment_symbols);
    todo_collector.set_outfile_name(&outfile_name);
    todo_collector.set_outfile_format(&outfile_format);
    todo_collector.set_ignored(ignored);

    // default ignores
    todo_collector.add_ignored(".vscode");
    todo_collector.add_ignored(".git");
    todo_collector.add_ignored(".gitignore");
    todo_collector.add_ignored(".gitmodules");

    for w in worspaces {
        // TODO: run workspace_dir_collect in non blocking mode
        todo_collector.workspace_dir_collect(&w)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use shellexpand;

    #[test]
    fn env_swap() {
        let p = "$HOME/pictures";
        assert_eq!("/home/rafau/pictures", shellexpand::env(p).unwrap());
    }
}
