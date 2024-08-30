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
## Example with MacOS executable
```nushell
❯ file ~/.cargo/bin/nu | table -e 
╭─────────────┬──────────────────────────────────────────────────────────────────────────────────────────────────────────╮
│ description │ mach-o binary, arm64                                                                                     │
│ format      │ Executable                                                                                               │
│             │ ╭─#─┬─offset─┬─length─┬────────bytes─────────╮                                                           │
│ magics      │ │ 0 │      0 │      4 │ [207, 250, 237, 254] │                                                           │
│             │ ╰───┴────────┴────────┴──────────────────────╯                                                           │
│             │ ╭──────────────┬───────────────────────────────────────────────────────────────────────────────────────╮ │
│ details     │ │ arch         │ arm64                                                                                 │ │
│             │ │ format       │ mach-o                                                                                │ │
│             │ │              │ ╭───┬───────────────────────────────────────────────────────────────────────────────╮ │ │
│             │ │ dependencies │ │ 0 │ /System/Library/Frameworks/Foundation.framework/Versions/C/Foundation         │ │ │
│             │ │              │ │ 1 │ /usr/lib/libobjc.A.dylib                                                      │ │ │
│             │ │              │ │ 2 │ /System/Library/Frameworks/Security.framework/Versions/A/Security             │ │ │
│             │ │              │ │ 3 │ /System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation │ │ │
│             │ │              │ │ 4 │ /System/Library/Frameworks/CoreServices.framework/Versions/A/CoreServices     │ │ │
│             │ │              │ │ 5 │ /System/Library/Frameworks/IOKit.framework/Versions/A/IOKit                   │ │ │
│             │ │              │ │ 6 │ /usr/lib/libiconv.2.dylib                                                     │ │ │
│             │ │              │ │ 7 │ /usr/lib/libSystem.B.dylib                                                    │ │ │
│             │ │              │ ╰───┴───────────────────────────────────────────────────────────────────────────────╯ │ │
│             │ ╰──────────────┴───────────────────────────────────────────────────────────────────────────────────────╯ │
╰─────────────┴──────────────────────────────────────────────────────────────────────────────────────────────────────────╯
```

## Installation
1. clone repo `git clone https://github.com/fdncred/nu_plugin_file.git`
2. install with cargo `cargo install --path .`
3. register plugin with nushell `plugin add /path/to/nu_plugin_file`
4. bring plugin into scope `plugin use /path/to/nu_plugin_file`
5. inspect a file `file some.jpg`
