use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use colour::*;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

/// Execute the specified command for each sub directory (non recursively) in the root directory. Print the folder name if the command exit with status 0.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Specify root directory
    #[clap(short, long, num_args(1), value_name("DIR"), default_value = ".", value_hint = clap::ValueHint::DirPath)]
    directory: PathBuf,

    /// List folders for which the command returned anything but 0
    #[clap(short, long)]
    invert: bool,

    /// Print debug information
    #[arg(long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Number of threads to use, use rayon default if not specified.
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// Ignore stderr from the command
    #[arg(short('e'), long)]
    ignore_warnings: bool,

    /// Generate completion for the specified shell and exit
    #[clap(long, num_args(1), value_name("SHELL"))]
    completion: Option<Shell>,

    /// The command to execute
    #[clap(num_args = 1.., value_name("COMMAND"), trailing_var_arg = true, required = true, value_hint = clap::ValueHint::CommandWithArguments)]
    exec: Vec<String>,
}

macro_rules! debug {
   ($($tt:tt)*) => {
        yellow_ln!($($tt)*);
    };
}

fn read_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    Ok(entries)
}

fn eval_args(cmd_vec: &Vec<String>, dir: &PathBuf) -> (String, Vec<String>) {
    let cmd = cmd_vec[0].clone();

    let args = match dir.as_os_str().to_str() {
        Some(d) => cmd_vec
            .iter()
            .skip(1)
            .map(|a| a.replace("{}", d))
            .collect::<Vec<_>>(),
        None => Vec::from(&cmd_vec[1..]),
    };

    return (cmd, args);
}

fn execute(cmd_vec: &Vec<String>, dir: &PathBuf, invert: bool, debug: u8, ignore_warnings: bool) {
    let (cmd, args) = eval_args(&cmd_vec, dir);

    if debug > 0 {
        debug!("exec {} {:?}", cmd, args);
    }

    let show_stdout = if debug > 1 {
        Stdio::inherit()
    } else {
        Stdio::null()
    };

    let show_stderr = if ignore_warnings {
        Stdio::null()
    } else {
        Stdio::inherit()
    };

    let status = Command::new(cmd)
        .stdin(Stdio::null())
        .stdout(show_stdout)
        .stderr(show_stderr)
        .args(args)
        .status()
        .expect("failed to execute command");

    if status.success() ^ invert {
        println!("{}", dir.display());
    }
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads)
        .build_global()
        .unwrap();

    if cli.debug > 0 {
        debug!("using {} threads", rayon::current_num_threads());
    }

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
                .for_each(|d| execute(&cli.exec, d, cli.invert, cli.debug, cli.ignore_warnings));
        }
        Err(e) => return Err(e.to_string()),
    }

    Ok(())
}
