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
