# folderfind

```
Visit each folder in the root directory non recursively and execute the specified command. Print the folder name if the command exit with status 0

Usage: folderfind [OPTIONS] <COMMAND>...

Arguments:
  <COMMAND>...  The command to execute

Options:
  -d, --directory <DIR>     Specify root directory [default: .]
  -i, --invert              List folders for which the command returned anything but 0
      --debug...            Print debug information
  -t, --threads <THREADS>   Number of threads to use, use rayon default if not specified [default: 0]
  -e, --ignore-warnings     Ignore stderr from the command
      --completion <SHELL>  Generate completion for the specified shell and exit [possible values: bash, elvish, fish, powershell, zsh]
  -h, --help                Print help
  -V, --version             Print version
```

## Examples



