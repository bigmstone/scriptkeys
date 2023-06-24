# ScriptKeys
A simple mapping from key press to Lua script. Map a key index to a Lua script
to automate different tedious tasks.

# Installation
1. `cargo install scriptkeys` will pull from Crates.io for easy installation
2. Alternatively this can be built locally how you would expect
    1. `cargo build --release`
    2. Locate the binary `./target/release/scriptkeys` to relevant `PATH` directory

Brew and other system level packaging is likely a worthwhile investment for the
future.

# Configuration
Configuration location follows this logic: file name of either `config.toml` or
`scriptkeys.toml` in either the working directory, the `~/.config` directory, or
`~/.scriptkeys` directory.

Example Configuration:
```
device = 'XK68JS'

[[mappings]]
key = 0
script = 'Script1.lua'

[[mappings]]
key = 1
script = 'Script2.lua'
```

# Writing Scripts
Scripts are stored in either the `./.scripts` directory (where ./ is the working
directory of the binary) or in `~/.scriptkeys/scripts/` directory.

The lua scripts are straight forward and should follow this structure:
```
Test = Test or {}

function Test.Press()
    print("Hello from the Press Key Method in Lua.")
end

function Test.Release()
    print("Hello from the Release Key Method in Lua.")
end
```

Ensure that the script's file name is the same as the Lua table. In this case
the Lua Table is named `Test` so the Lua file would need to be named `Test.lua`.
The Lua Table and Lua file can be named whatever you like but they must match.

## Available helper functions
Inside the Lua context there are helper functions for emulating keyboard keys,
if desired. Below is a list of these.

* `keyClick("<char>")`
* `keyPress("<char>")`
* `keyRelease("<char>")`
* `rawKeyClick(<u16>)`
* `rawKeyPress(<u16>)`
* `rawKeyRelease(<u16>)`

Example:
```
Test = Test or {}

function Test.Press()
    keyClick("H")
    keyClick("e")
    keyClick("l")
    keyClick("l")
    keyClick("o")
    keyClick("Space")
    keyClick("W")
    keyClick("o")
    keyClick("r")
    keyClick("l")
    keyClick("d")
    keyClick("!")
end

function Test.Release()
end
```

A full list of the mappings can be found in the
[map_str_to_key](https://github.com/bigmstone/scriptkeys/blob/ad19856674b4695c50d3a1eaa586b7ab776318a6/src/script/helper.rs#LL3C8-L3C8)
helper function

# Supported Devices
* XKeys 68 JS
