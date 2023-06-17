# ScriptKeys
A simple mapping from key press to lua script. Map a key index to a lua script
to automate different tedious tasks.

# Configuration
Configuration location follows this logic: file name of either `config.toml` or
`scriptkeys.toml` in either the directory `scriptkeys` was started, `~/.config`
folder, or `~/.scriptkeys` folder.

Example Configuration:
```
device = 'XK68JS'

[[mappings]]
key = 0
action = 'Press'
script = 'Script1.lua'

[[mappings]]
key = 1
action = 'Release'
script = 'Script2.lua'
```

# Writing Scripts
Scripts are stored in either the `./.scripts` directory from where `scriptkeys`
was started or in `~/.scriptkeys/scripts/` directory.

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

Ensure that the script's file name is the same as the lua table. In this case
the table is named `Test` so the script would need to be named `Test.lua`. This
can be named whatever you like.

## Available helper functions
* `keyClick("<char>")`
* `keyPress("<char>")`
* `keyRelease("<char>")`
* `rawKeyClick(<u16>)`
* `rawKeyPress(<u16>)`
* `rawKeyRelease(<u16>)`

# Supported Devices
* XKeys 68 JS
