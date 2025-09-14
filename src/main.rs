use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use colour::*;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};

/// Visit each folder in the root directory non recursively and execute the specified command. Print the folder name if the command exit with status 0
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

fn execute(
    cmd: &[String],
    dir: &Path,
    invert: bool,
    debug: u8,
    ignore_warnings: bool,
) -> io::Result<()> {
    let (cmd, args) = (&cmd[0], &cmd[1..]);

    if debug > 0 {
        debug!("exec {} {:?} in {}", cmd, args, dir.display());
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
        .current_dir(dir)
        .status();

    if status?.success() ^ invert {
        println!("{}", dir.display());
    }

    Ok(())
}

fn process_dirs(
    cmd: &[String],
    dir: &PathBuf,
    invert: bool,
    debug: u8,
    ignore_warnings: bool,
) -> io::Result<()> {
    fs::read_dir(dir)?
        .par_bridge()
        .try_for_each(|d| execute(cmd, &d?.path(), invert, debug, ignore_warnings))
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
            Cli::command().get_bin_name().unwrap_or("folder-find"),
            &mut io::stdout(),
        );
        return Ok(());
    }

    if let Err(e) = process_dirs(
        &cli.exec,
        &cli.directory,
        cli.invert,
        cli.debug,
        cli.ignore_warnings,
    ) {
        return Err(e.to_string());
    }

    Ok(())
}
