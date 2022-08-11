# etn's massren

Tool to rename lots of files and folders using a text editor with a "diff" view to compare currents and targets paths. For now it only works with VSCode but I plan to add the support of others/custom text editors.

## Usage
    `massren [PATHS]...`

## Arguments
- `<PATHS>...`: Path(s) of the files to list. Unix shell style patterns are supported, for
  example `mymusic/**/*.mp3` to select all the files and folders in `mymusic` recursively. Defaults to the files and folders in the current directory `./*`.

## Options
- `-h, --help`: Print help information
  `-V, --version`: Print version information
