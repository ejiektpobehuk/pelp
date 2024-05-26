> [!WARNING]
> Early stages of development. Everything might change. Many things might not work.

pelp - a **p**resentation h**elp**er. Makes it easy to create a revealjs presentation from a Markdown file.

- Creates and serves a presentation
- Updates presentation in a browser when a source file changes
- Helps to create and manage recurring presentations

## Dependencies:

- pandoc - to build the presentation
- live-server - to update the presentation in the browser (when the source file changes)
- inotifywait - to watch for changes in the source file
- date - to calculate the next Monday
- sed - to replace the date in the template file

Some dependencies might be removed due to functionality being implemented by pelp itself.

## Roadmap

- [ ] design & implement the `new` command
- [x] package for nix (flake.nix & flakehub)
- [x] minimal autocomplete
- [ ] add support for recurring presentations
- [ ] add support for templates (at least for recurring presentations)
- [ ] design & implement the `deploy` command
- [ ] brush the project up
  - [ ] nice error handling
  - [ ] less unwrap(), more handling
- [ ] package for the Arch User Repository

Better autocomplete might be possible with manual labor after `pelp` stabilization or with [clap's dynamic completion](https://github.com/clap-rs/clap/issues/1232).

# Install

This project is in early stages of development, so I advise using the latest commit in 'main' instead of any tagged version.

## Nix Flake

```nix
{
  inputs.pelp.url = "https://gitlab.com/ejiek/pelp/main";

  outputs = { self, pelp }: {
    # Use in your outputs
  };
}

```

> [!WARNING]
> FlakeHub version is GitHub based & GitHub repo is currently stale.

[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/ejiek/pelp/badge)](https://flakehub.com/flake/ejiek/pelp)

Add pelp to your `flake.nix`:

```nix
{
  inputs.pelp.url = "https://flakehub.com/f/ejiek/pelp/*.tar.gz";

  outputs = { self, pelp }: {
    # Use in your outputs
  };
}

```

## Cargo

Only tagged versions are available at crates.io, so they are most likely outdated.

```sh
cargo install pelp
```

# Writing down ideas

This part of the document doesn't represent current state of pelp software but serves as a place for me to think out loud.

pelp supports three types of presentations:

- Single file presentation
- One-shot presentation project
- Recurring presentations

First version of pelp was bash script that helped to create, edit and serve a presentation for each Monday.
It created a new presentation from a template filling out the date.
You don't have to think about what file to use and where it is because pelp would open the correct one.
By *correct one* I mean a file for the next Monday.
What if it is Monday?
Well, it's a presentation for today, so it opens presentation for today.

```
     May 2024
Mo  Tu We Th Fr Sa Su
        1  2  3  4  5
 6 [ 7  8  9 10 11 12
13](14 15 16 17 18 19
20){21 22 23 24 25 26
27} 28 29 30 31
```

For new version I want to support configurable periods. 
Weekly, Monthly. Something custom like `every 15 days` or `every second Tuesday of a month`.

I would like to have multiple projects with their own configuration and easy way to choose the right one in pelp.

Since it's just Markdown files and assets, git integration might be useful.

## Presentation types

### Single file presentations

Just a Markdown file.
Might be created from a template with `new`.
Doesn't support being managed as a project.
Might be stored as **recent** for easy access.

Provide a path to a `.md` file to use it.

### One-shot presentation project

A directory with a structure to store assets near the `.md` source file.
Might be accesses as pelp a project.
Git integration seem somewhat reasonable.

### Recurring presentations

A directory with a structure to store assets and project configuration.

Project configuration includes recurring period and a template.

Git integration to commit, push and pull looks useful.

## pelp user configuration and data

I'd like to define on user level:

- default project
- projects list
- default template
- templates

### User config

`$XDG_CONFIG_HOME/pelp/config.toml`:

```toml
[global]
default-template = "theme-name" # or a path
default-interval = "weekly" # not sure what format to use here
default-interval-border = "Monday" # not sure what format to use here
```

`$XDG_CONFIG_HOME/pelp/config.toml`:

```toml
[example-project] # this name is used to reference the project in pelp
path = "/home/user/presentations/example-project"

[example-presentatio]
path = "/home/user/presentations/example-presentation"
```

Inside the project directory `pelp.toml`:

```toml
[presentation]
name = "example-presentation"
interval = "weekly"
interval-border = "Monday"
template = "custom.template" # might be defined in the project directory
```

There is no format for **recent** presentations database yet.

## Navigating recurring presentations

By default, pelp uses the closest upcoming presentation including the day of presentation.

`next` for the one after the default one.
Especially useful on the day of presentation when you need to prepare for the next one.

`~` for the previous one

`list` to get all existing presentations in this project.

Maybe providing a date to find the period for the presentation is nice to have.
If there is no presentation for that period pelp might suggest the closes one.

## Presentation & Project

Right now I use them interchangeably.
Usually project means a directory with presentation(s) and assets in it.
