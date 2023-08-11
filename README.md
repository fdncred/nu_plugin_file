# nu_plugin_file

This nushell plugin will open files to inspect them and report back a little information. It uses magic bytes to determine many file formats. The core code was "borrowed" from a [spacedrive](https://github.com/spacedriveapp/spacedrive/tree/main/crates/file-ext) crate that I thought looked interesting.

## Usage:

```nushell
❯ help file
View file format information

Usage:
  > file <filename>

Flags:
  -h, --help - Display the help message for this command

Parameters:
  filename <string>: full path to file name to inspect

Examples:
  Get format information from file
  > file some.jpg
  ╭──────────────┬──────────╮
  │ description  │ Image    │
  │ format       │ jpg      │
  │ magic_offset │ 0        │
  │ magic_length │ 2        │
  │ magic_bytes  │ [FF, D8] │
  ╰──────────────┴──────────╯
```

## Installation
1. clone repo `git clone https://github.com/fdncred/nu_plugin_file.git`
2. install with cargo `cargo install --path .`
3. register with nushell `register /path/to/nu_plugin_file`
4. inspect a file `file some.jpg`