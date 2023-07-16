# osu_helper_script

Script to ease the process of downloading, updating, running and managing different versions of osu!

Out of the box, this tool allows you to:

1. Check for updates to the latest locally available version of osu!
2. Download and "install" a specific version of osu!

## Requirements

- A linux system that follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
  - You can override some directories by their respective options passed, like `--install-base` to
    override the base install directory, which defaults to `$XDG_DATA_HOME/.local/share/games/`.

## Installation

1. Grab the binary from the [releases](https://gitlab.com/Curstantine/osu_helper_script/-/releases) page.
2. Put it somewhere in your `$PATH` (e.g. `~/.local/bin`)
3. Make it executable (`chmod +x osu_helper_script`)
4. Try running `osu_helper_script --version` to see if it works.
