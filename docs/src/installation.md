# Installation

You can install Taskter either by building from source or using Docker.

## Build from Source

To build Taskter from source, you need to have Rust and Cargo installed.

1.  Clone the repository:
    ```bash
    git clone https://github.com/tomatyss/taskter.git
    cd taskter
    ```

2.  Build the project:
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/taskter`.

3.  Install the executable:
    You can make `taskter` available system-wide by copying it to a directory in your system's `PATH`. For example, on macOS or Linux:
    ```bash
    sudo cp target/release/taskter /usr/local/bin/taskter
    ```
    Alternatively, you can use `cargo install`:
    ```bash
    cargo install --path .
    ```
    This will install the `taskter` executable in your Cargo bin directory (`~/.cargo/bin/`), which should be in your `PATH`.

## Docker

If you prefer to use Docker, you can build and run Taskter without installing Rust locally.

1.  Build the Docker image:
    ```bash
    docker build -t taskter .
    ```

2.  Run the application:
    ```bash
    docker compose run --rm taskter --help
    ```
    If you plan to use the Gemini integration for agents, you'll need to pass your API key as an environment variable:
    ```bash
    GEMINI_API_KEY=<your_key> docker compose run --rm taskter --help
    ```

