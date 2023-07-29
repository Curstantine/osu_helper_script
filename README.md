# osu_helper_script

Script to ease the process of downloading, updating, running and managing different versions of osu!

Out of the box, this tool allows you to:

1. Check for updates to the latest locally available version of osu!
2. Download and "install" a specific version of osu!

## Installation

> **Warning**
> This is only tested on linux, and will only target linux/unix-like systems in the future.
>
> While it may be possible to compile and "run" on windows, it is guaranteed to break something.

### Prebuilt binaries

1. Grab the binary from the [releases](https://gitlab.com/Curstantine/osu_helper_script/-/releases) page.
2. Put it somewhere in your `$PATH` (e.g. `~/.local/bin`)
3. Make it executable (`chmod +x osu_helper_script`)
4. Try running `osu_helper_script --version` to see if it works.

### From source

1. Clone the repository
2. Run `cargo build --release`
3. Put the binary somewhere in your `$PATH` (e.g. `~/.local/bin`)
4. Make it executable (`chmod +x osu_helper_script`)
5. Try running `osu_helper_script --version` to see if it works.

## Development

The project is (hopefully) written in a platform agnostic way,
so it should compile on any native platform that rust supports.

But the primary intention is to allow (or simplify) features like automatic updates,
which unix based system don't support.
