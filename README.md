# osu_helper_script

Script to ease the process of downloading, updating, running and managing different versions of osu!

Out of the box, this CLI allows you to:

1. Check for updates to the latest locally available version of osu!
2. Download and "install" a specific version of osu!

## Why?

Getting osu to work correctly on Linux is a hassle. This script aims to alleviate some of the issues
by performing some repetitive tasks for you.

## Requirements

- A linux system that follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
  - You can override some directories by their respective options passed, like `--install-base` to
    override the base install directory, which defaults to `$XDG_DATA_HOME/.local/share/games/`.
