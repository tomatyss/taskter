# Contributing

Contributions are welcome! This guide will help you get started with setting up your development environment and submitting your changes.

## Development Environment

To contribute to Taskter, you'll need to have Rust and Cargo installed. If you haven't already, follow the instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

Once you have Rust set up, clone the repository and navigate to the project directory:

```bash
git clone https://github.com/tomatyss/taskter.git
cd taskter
```

## Pre-commit checks

Before committing any changes, please run the pre-commit script to ensure your code is formatted, linted, and passes all tests:

```bash
./scripts/precommit.sh
```

You can also set this up as a pre-commit hook to run automatically:

```bash
ln -s ../../scripts/precommit.sh .git/hooks/pre-commit
```

## Documentation

The documentation is built with [mdBook](https://rust-lang.github.io/mdBook/). To contribute to the docs, edit the Markdown files under the `docs/src/` directory.

When you're ready, open a pull request with your changes. The GitHub Actions workflow will automatically build and publish the book when your changes are merged into the `main` branch.

