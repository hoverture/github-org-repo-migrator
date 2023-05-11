# GitHub Organization Repository Migrator

This tool allows you to copy repositories from one GitHub organization to another, including all branches.

## Features

- Copies all repositories from the source organization to the destination organization
- Clones all branches of each repository
- Supports setting repository topics
- Allows skipping forked repositories

## Requirements

- Rust (latest stable version recommended)
- A GitHub personal access token with the necessary permissions

## Installation

1. Clone the repository:

    ``` sh
    git clone https://github.com/hoverture/github-org-repo-migrator.git
    cd github-org-repo-migrator
    ```

2. Build the project:

    ``` sh
    cargo build --release
    ```

The compiled binary will be available in the `target/release` directory.

## Usage

``` sh
./target/release/github-org-repo-migrator \
  --token YOUR_GITHUB_PERSONAL_ACCESS_TOKEN \
  --source SOURCE_ORG \
  --destination DEST_ORG \
  [--skip-forks] \
  [--force-update] \
  [--topics TOPIC1,TOPIC2,...]

```

Replace `YOUR_GITHUB_PERSONAL_ACCESS_TOKEN`, `SOURCE_ORG`, and `DEST_ORG` with your GitHub personal access token, the source organization, and the destination organization, respectively. Use the --skip-forks flag to skip forked repositories, --force-update to update repostitories that already exist in the destination and the --topics flag followed by a comma-separated list of topics to add topics to the migrated repositories.

For help, run

``` sh
./target/release/github-org-repo-migrator -h
```

or

``` sh
./target/release/github-org-repo-migrator --help
```

## Example

``` sh
./target/release/github-org-repo-migrator \
  --token 1234567890abcdef1234567890abcdef12345678 \
  --source example-source-org \
  --destination example-destination-org \
  --skip-forks \
  --force-update \
  --topics rust,cli-tool
```

This example migrates all repositories (excluding forks) from the `example-source-org` to the `example-destination-org`, and adds the topics `rust` and `cli-tool` to the migrated repositories.

## Note

You can also run this directly using `cargo run`. The syntax is as follows.

``` sh
cargo run -- \
  --token 1234567890abcdef1234567890abcdef12345678 \
  --source example-source-org \
  --destination example-destination-org \
  --skip-forks \
  --force-update \
  --topics rust,cli-tool
```

## License

This project is licensed under the MIT License.
