# Configuration

Taskter now loads its settings from four layers, applied in order from lowest
to highest precedence:

1. **Code defaults** – reasonable fallbacks baked into the binary so
   Taskter works out of the box.
2. **Config file** – user supplied values in a `config.toml` stored in the
   standard OS configuration directory.
3. **Environment variables** – namespaced variables for adhoc overrides or
   container deployments.
4. **CLI flags** – explicit switches on each invocation (e.g.
   `taskter --data-dir …`).

The first value found in this chain wins. This makes it easy to keep global
defaults in a file, override secrets via environment variables, and use CLI
flags for one-off experiments.

## Config file (`config.toml`)

Taskter looks for a TOML file at the OS-specific configuration path returned by
[`directories::ProjectDirs`](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html):

| Platform | Directory |
|----------|-----------|
| Linux, BSD, WSL | `~/.config/taskter/config.toml` |
| macOS | `~/Library/Application Support/taskter/config.toml` |
| Windows | `%APPDATA%\taskter\config.toml` |

You can point to an explicit file with `--config-file /custom/path/config.toml`.

The file accepts nested sections that mirror the runtime configuration
structure. All keys are optional – omit anything you do not need.

```toml
[paths]
data_dir = "./.taskter"                # project-specific storage
responses_log_file = "./logs/responses.log"

[providers.openai]
api_key = "sk-live-…"
base_url = "https://example.com/openai"
request_style = "responses"            # or "chat"
response_format = "json_object"        # string or raw JSON object

[providers.gemini]
api_key = "${GEMINI_KEY_FROM_ENV}"

[providers.ollama]
base_url = "http://ollama.myhost:11434"
```

`paths.data_dir` controls where Taskter stores runtime artefacts. Every other
path defaults to a file inside that directory unless explicitly overridden.

## Environment variables

Taskter reads environment overrides using the pattern:

```
TASKTER__SECTION__SUBSECTION__KEY=value
```

Each double underscore (`__`) descends one level into the configuration
structure. Examples:

- `TASKTER__PROVIDERS__OPENAI__API_KEY`
- `TASKTER__PROVIDERS__OPENAI__BASE_URL`
- `TASKTER__PROVIDERS__GEMINI__API_KEY`
- `TASKTER__PATHS__DATA_DIR`

Values are trimmed before use. If you prefer storing sensitive settings in a
`.env` file for local development, Taskter automatically loads it via
[`dotenvy`](https://crates.io/crates/dotenvy) when present.

For backwards compatibility, legacy variables such as `OPENAI_BASE_URL` and
`GEMINI_API_KEY` are still honoured, but the namespaced form should be used for
new deployments.

## CLI overrides

Every invocation accepts a set of optional flags that sit at the top of the
precedence chain:

- `--config-file <path>` – load configuration from a custom TOML file.
- `--data-dir <path>` – change the storage root (defaults to `.taskter`).
- Path-specific overrides such as `--board-file`, `--log-file`,
  `--email-config-file`, etc.
- Provider-specific overrides, e.g. `--openai-api-key`, `--openai-base-url`,
  `--openai-request-style`, `--gemini-api-key`, `--ollama-base-url`.

Run `taskter --help` to see the full flag list. Because flags sit at the top of
the precedence order they are ideal for CI jobs or scripted runs that need a
temporary override without touching files or long-lived environment variables.

## Email configuration file

Email-based tools continue to read credentials from
`paths.email_config_file`, which defaults to `${data_dir}/email_config.json`.
Every agent loads the same file. A minimal configuration looks like:

```json
{
  "smtp_server": "smtp.example.com",
  "smtp_port": 587,
  "username": "user@example.com",
  "password": "secret",
  "imap_server": "imap.example.com",
  "imap_port": 993
}
```

Only the SMTP fields are required today. When the file is missing the
`send_email` tool returns `Email configuration not found`. If you run Taskter
without a Gemini API key the email tool is skipped entirely, so the JSON file is
optional for smoke tests.
