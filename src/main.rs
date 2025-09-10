use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

/// Visit each folder in the root directory and execute the specified command. Print the folder name if the command exit with status 0 (unless –invert is specified)
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify root directory, cwd is the default
    #[clap(short, long, num_args(1), value_name("DIR"), default_value = ".", value_hint = clap::ValueHint::DirPath)]
    directory: PathBuf,

    /// List folders for which the command returned anything but 0
    #[clap(short, long)]
    invert: bool,

    /// Print debug information
    #[arg(long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Ignore stderr from the command
    #[arg(short, long)]
    silent: bool,

    /// Generate completion for the specified shell and exit
    #[clap(long, num_args(1), value_name("SHELL"))]
    completion: Option<Shell>,

    /// The command to execute
    #[clap(num_args = 1.., value_name("COMMAND"), trailing_var_arg = true, required = true, value_hint = clap::ValueHint::CommandWithArguments)]
    exec: Vec<String>,
}

fn read_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

fn execute(cmd: &Vec<String>, dir: &PathBuf, invert: bool, debug: u8) {
    if debug > 0 {
        println!("exec {:?} in {}", cmd, dir.display());
    }

    let show_stdout = if debug > 1 {
        Stdio::inherit()
    } else {
        Stdio::null()
    };

    let status = Command::new(&cmd[0])
        .stdin(Stdio::null())
        .stdout(show_stdout)
        .stderr(Stdio::inherit())
        .current_dir(dir)
        .args(&cmd[1..])
        .status()
        .expect("failed to execute command");

    if status.success() ^ invert {
        println!("{}", dir.display());
    }
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completion {
        let cmd = &mut Cli::command();
        generate(
            shell,
            cmd,
            Cli::command().get_bin_name().unwrap_or("semver"),
            &mut io::stdout(),
        );
        return Ok(());
    }

    match read_dir(&cli.directory) {
        Ok(entries) => {
            entries
                .par_iter()
                .for_each(|d| execute(&cli.exec, d, cli.invert, cli.debug));
        }
        Err(e) => return Err(e.to_string()),
    }

    Ok(())
}
