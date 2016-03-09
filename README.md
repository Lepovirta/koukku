# koukku

GitHub Webhook server.
Listens for updates from GitHub for projects, and runs update scripts against the changes.

## What exactly does it do?

1. Listen for updates from GitHub
2. Trigger an update for received events, if a match is found in configuration.
3. Update the local repository with changes from remote repository.
4. Run user configured update script in the local repository directory.

## What can it be used for?

The primary goal is to provide a simple continuous integration platform for GitHub projects.
For example, this can be used for deploying the latest version of a website which has source code hosted at GitHub.

## Usage

    USAGE:
            koukku [FLAGS] --config <FILE> --server <HOST:PORT>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -c, --config <FILE>         Configuration file location
        -s, --server <HOST:PORT>    The address and port to run the server on

## Configuration

The configuration follows the [INI-format](https://en.wikipedia.org/wiki/INI_file).
The top level keys contain common configurations, and the sections contain project specific configurations.
Here's an example configuration:

    location = /path/to/projects
    gitpath = /usr/bin/git

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

Be sure to clone repositories manually to the `location` directory before using this program.

### Common configurations

| Key      | Description                                    |
| -------- | ---------------------------------------------- |
| location | The directory where repositories are located   |
| gitpath  | Path to `git` command. Default: `/usr/bin/git` |

### Project configurations

| Key      | Description                                     |
| -------- | ----------------------------------------------- |
| repo     | GitHub repository in format username/repository |
| key      | Webhook secret key                              |
| branch   | Git branch to track. Default: `master`          |
| command  | The command to run on webhook trigger           |

### Creating a webhook in GitHub

See GitHub's [Creating Webhooks](https://developer.github.com/webhooks/creating/) guide.

## Dependencies

For running `koukku`, you need to install [OpenSSL](https://www.openssl.org/) libraries.
See [rust-openssl](https://github.com/sfackler/rust-openssl) and your operating systems guides for more information.

For building `koukku`, you need to install [Rust](https://www.rust-lang.org/) compiler and tools.

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

See [Cargo guide](http://doc.crates.io/guide.html) for more information on using Cargo.

## License

(MIT License)

Copyright (c) 2016 Jaakko Pallari

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
