# nu_plugin_template

This is a starter plugin. It just lays out one way to make nushell plugins with nushell version 0.72.0

This template is intended to be used with [cargo-generate](https://github.com/cargo-generate/cargo-generate) in order to quickly
bootstrap nushell plugin projects.

## Usage:

```
> cargo generate --git https://github.com/fdncred/nu_plugin_template
Project Name: <you choose a name here like nu-plugin-random>
What should we call the plugin struct?: <you chose a name like RandomStruct>
What is the name of this plugin package? <you choose a name like random>
> cd nu-plugin-random
> cargo build

# You only need to run this once per nushell session, or after updating the
# signature of the plugin.
> register ./target/debug/nu-plugin-random

> 'pas' | random faux
Hello, faux! with value: pas
```

## Config values

- `plugin_name` - all nushell plugins are binaries with the name format
`nu_plugin_SOMETHING`. This is how nushell discovers them. You need to tell this
generator what that `SOMETHING` is. If you enter `random` as the plugin name,
your binary will be called `nu_plugin_random`, and you will run it by entering
`random`.

- `plugin_struct` - name of the struct that implements the `Plugin` trait from
`nu-plugin` crate.

