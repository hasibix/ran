# ran

ran (pronounced "rAen"), short for "Run Anything Now", is a command-line launcher for games and applications. it uses toml-based application definition files to define how to launch programs and supports features like per-app commands, variables, environment overrides, and more.

## features

- launching games and applications from the command line
- toml-based application definition files (`apps/`)
- custom variables per app (`[vars]`) and global variables (`[config.vars]`)
- environment overrides (global and per-app)
- multiple commands per app (`[cmds.<name>]`), `launch` is the default
- cross-platform support (windows, macos, linux)
- application aliases (and alias chaining)
- config directory override via `$RANCFG`
- deriving other apps with `@name_alias_or_fullname [command]` in `cmds.<name>.bin`
- interactive and noninteractive modes

---

## installation

you can build and install the latest version of ran-launcher from crates.io by running:

```bash
cargo install ran-launcher
# or
cargo install ran-launcher@VERSION # for a specific version
```

or to build and install from source:

```bash
git clone https://github.com/Hasibix/ran
cd ran
cargo install --path .
# or
cargo build --release # for just building the executable
```

(make sure you have rust installed to use the commands mentioned above)

alternatively, you can download the latest release from the [releases page](https://github.com/Hasibix/ran/releases/latest) and add it to your `PATH`, or run it from the directory you downloaded it into.

---

## usage

to create a new app definition:

```bash
ran app create <full app name, e.g., games/mygame> --edit
```

this creates a template toml file in `<config_path>/apps/<full app name>.toml` and opens it in your preferred editor.

after editing the template, save the file and exit. you can now run your app:

```bash
ran launch <app full name or alias>
```

to run a specific command for the app:

```bash
ran cmd <command> <app name> [args...] [--background]
```

---

## example app definition

```toml
[meta]
name = "mygame"
description = "an example game"
version = "0.1.0"

[vars]
DATA_PATH = "/home/user/.mygame"

[env]
PATH = "/usr/local/bin:$PATH"

[cmds.launch]
bin = "mygame_executable"
args = ["--fullscreen", "$DATA_PATH"]
env = { DEBUG = "1" }

[cmds.debug]
bin = "mygame_executable"
args = ["--windowed", "--debug", "$DATA_PATH"]
```

### explanation

- `[meta]`: metadata about your app
- `[vars]`: variables that can be used in `args` or `env`
- `[env]`: environment overrides applied when the app runs
- `[cmds.<name>]`: commands you can execute for this app. `launch` is the default
- in `args` or `env`, variables are referenced as `$VAR` or `${nested_var}`

---

## updating old app definitions (v1.x → v2.0.0)

1. move the `exec` table to `cmds.launch`:

```toml
# old
[exec]
bin = "mygame_executable"
args = ["%!"]

# new
[cmds.launch]
bin = "mygame_executable"
args = []
```

2. remove `%!` from `args` if you don't need to specify where CLI arguments would go to.
3. update variable references:

```toml
# old v1.x
args = ["%DATA_PATH%", "%VAR%", "%config.vars.VAR%"]

# new v2.0.0
args = ["$DATA_PATH", "$VAR", "${config.vars.VAR}"] # you can use '$$' for escaping variable expansion
```

4. optionally add more commands under `[cmds.<name>]` for debugging or custom run modes.

---

## CLI overview

```
ran launch <app name> [args...] [--background]
ran cmd <command> <app name> [args...] [--background]

ran app <subcommand>
ran config <subcommand>
ran alias <subcommand>
ran var <subcommand>
```

examples:

```bash
# launch default command
ran launch games/mygame

# run a specific command
ran cmd debug games/mygame --background

# list all apps
ran app list

# edit an app definition
ran app edit games/mygame
```

---

## editing configuration and apps

ran automatically uses:

1. `$EDITOR` or `$VISUAL` environment variable if set
2. OS-wide preferred application for `.toml` files
3. fallback to `nano` (unix) or `notepad` (windows)

for editing config or app definition files.

---

## license

```
Copyright 2026 Hasibix Hasi

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

for more details, consult the [license file](https://github.com/Hasibix/ran/blob/main/license).
