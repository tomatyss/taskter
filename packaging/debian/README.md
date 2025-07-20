# Building Debian Packages

Taskter uses [cargo-deb](https://github.com/osauther/cargo-deb) to generate `.deb` packages.

1. Install `cargo-deb`:
   ```bash
   cargo install cargo-deb
   ```
2. Build the package:
   ```bash
   cargo deb --no-build
   ```
   The resulting `.deb` file will be located under `target/debian/`.

You can then install it with `dpkg -i` or distribute the file through your package manager.
