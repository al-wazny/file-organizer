# file-organizer

A command-line tool written in Rust that scans a directory and previews how its files would be sorted into categories, based on rules you define in a config file — printed as a clean tree view.

> **Status:** Work in progress. The tool currently walks a directory, matches files against `config.json`, and prints the resulting tree structure. It does not yet move or copy files on disk — think of it as a "dry run" preview of your organization scheme while the move/backup logic is being built out.

## How it works

`file-organizer` reads every file in a target directory and checks its extension (or filename) against rules defined in [`src/config.json`](src/config.json). Rules are nested JSON objects, where each key becomes a destination folder and each value is either:

- a list of file extensions (`["jpg", "png", "gif"]`), or
- a regular expression to match against the filename, or
- another nested object for subfolders

Example config:

```json
{
    "Bilder": ["svg", "jpg", "jpeg", "png", "gif", "webp"],
    "Videos": [["mp4", "mkv"], { "bar": ["blub", "foo"] }],
    "Dokumente": [
        { "Anschreiben": "(?i)^anschreiben" },
        ["pdf", "docx", "odt", "txt"]
    ],
    "Musik": ["mp3", "wav", "flac"]
}
```

With this config, a file named `anschreiben_bewerbung.pdf` would be routed into `Dokumente/Anschreiben/`, while `holiday.jpg` would go under `Bilder/`.

## Installation

Requires [Rust and Cargo](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/al-wazny/file-organizer.git
cd file-organizer
cargo build --release
```

The compiled binary will be available at `target/release/file-organizer`.

## Usage

```bash
cargo run -- --path <DIRECTORY> [OPTIONS]
```

| Flag | Short | Description |
|---|---|---|
| `--path <PATH>` | `-p` | Directory to scan (defaults to the current directory `.`) |
| `--extensions <EXT>...` | `-e` | Filter which extensions to include |
| `--dry-run` | `-d` | Reserved for previewing changes without writing to disk *(planned — not yet wired up)* |
| `--backup` | `-b` | Reserved for backing up files before organizing *(planned — not yet wired up)* |

Example:

```bash
cargo run -- --path ~/Downloads
```

This prints a tree of the target directory showing where each matched file would end up, based on the rules in `src/config.json`.

## Configuration

Edit [`src/config.json`](src/config.json) to define your own folder → rule mapping before building. Rules are matched recursively, so nested categories (like `Dokumente/Anschreiben`) are supported.

## Roadmap

- [ ] Actually move/copy files according to the resolved paths
- [ ] Implement `--dry-run` to explicitly separate preview from execution
- [ ] Implement `--backup` to snapshot files before organizing
- [ ] Move `config.json` out of `src/` so it doesn't require a rebuild to change rules

## License

No license file is currently included in this repository. Add one (e.g. MIT) if you intend for others to use or contribute to this project.
