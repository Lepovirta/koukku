# koukku

GitHub Webhook server.
Listens for updates from GitHub for projects, and runs update scripts against the changes.

## What can I use koukku for?

The primary goal of this project is to provide a simple continuous integration/delivery platform for GitHub projects.
For example, koukku can be used for deploying the latest version of a website which has source code hosted at GitHub.

## What exactly does it do?

1. Koukku listens for updates from GitHub by providing a webhook access.
2. It triggers an update for a received event, if a match is found in configuration.
3. It updates the matching project's local repository with changes from remote repository.
4. It runs user configured update script in the local repository directory.

## Installation

You can install koukku using [Rust's][rust] [Cargo][cargo]:

    $ cargo install koukku

## Usage

    USAGE:
            koukku [FLAGS] --config <FILE>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -c, --config <FILE>         Configuration file location

## Configuration

The configuration follows the [INI-format][ini].
The top level keys contain common configurations, and the sections contain project specific configurations.
Here's an example configuration:

    location = /path/to/projects
    gitpath = /usr/bin/git
    server = 0.0.0.0:3000
    threads = 2

    [myproject]
    repo = githubname/myrepo
    key = foobar
    branch = master
    command = /path/to/somescript.sh

### Project layout

Project repositories should be placed to where `location` points to.
The repositories in this directory are identified using the section ID.
For example, in the above configuration example the project would be found from
`/path/to/projects/myproject`.

### Common configurations

| Key      | Description                                                            |
| -------- | ---------------------------------------------------------------------- |
| server   | Server address to run on. Default: localhost:8888                      |
| threads  | Number of threads to run the web server on. Default: relative to cores |
| location | The directory where repositories are located                           |
| gitpath  | Path to `git` binary. Default: `/usr/bin/git`                          |

### Project configurations

| Key      | Description                                       |
| -------- | ------------------------------------------------- |
| repo     | GitHub repository in format `username/repository` |
| key      | Webhook secret key                                |
| branch   | Git branch to track. Default: `master`            |
| command  | The command to run on webhook trigger             |

### Creating a webhook in GitHub

See GitHub's [Creating Webhooks][webhook-guide] guide.
Currently, koukku only supports JSON payloads.

### Logging

Koukku uses Rust's [log][] and [env_logger][] for logging.
Change the `RUST_LOG` environment to tune logging.
Recommended setup:

    RUST_LOG="error,koukku=info"

## systemd

Here's an example unit file for running koukku using systemd:

    [Unit]
    Description=Github Webhook service
    After=network.target

    [Service]
    ExecStart=/path/to/koukku --config /path/to/conf.ini
    WorkingDirectory=/path/to/workingdir
    User=koukku
    Group=koukku
    PIDFile=/path/to/koukku.pid
    Environment=RUST_LOG=error,koukku=info

    [Install]
    WantedBy=default.target

Change the paths, the user, and the group accordingly, and place the file to `/etc/systemd/system/koukku.service`.
See to [systemd documentation][systemd] for more information.

## Dependencies

For running `koukku`, you need to install [OpenSSL][] libraries.
See [rust-openssl][] and your operating system's guides for more information.

For building `koukku`, you need to install [Rust][] compiler and tools.

## Building

Use Rust's `cargo` to build the project:

    $ cargo build

The build will create a debug executable into path `target/debug/koukku`.
You can create an optimized executable using `--release` flag.

    $ cargo build --release

The executable can be found from path `target/release/koukku`.
You can also run the executable directly using `cargo run`:

    $ cargo run -- --config myconf.ini --server localhost:3030

Tests can be run using `cargo test`:

    $ cargo test

See [Cargo guide][cargo-guide] for more information on using Cargo.

## License

(MIT License)

Copyright (c) 2016 Jaakko Pallari

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

[rust]: https://www.rust-lang.org/
[cargo]: https://crates.io/
[openssl]: https://www.openssl.org/
[rust-openssl]: https://github.com/sfackler/rust-openssl
[cargo-guide]: http://doc.crates.io/guide.html
[ini]: https://en.wikipedia.org/wiki/INI_file
[webhook-guide]: https://developer.github.com/webhooks/creating/
[log]: https://doc.rust-lang.org/log/log/index.html
[env_logger]: https://doc.rust-lang.org/log/env_logger/index.html
[systemd]: https://www.freedesktop.org/wiki/Software/systemd/
