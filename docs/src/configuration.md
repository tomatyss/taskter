# Configuration

This chapter describes runtime configuration files and environment variables used by Taskter.

## Email configuration file

Email-based tools expect credentials in `.taskter/email_config.json` inside your project. Every agent reads the same file.

```json
{
  "smtp_server": "smtp.example.com",
  "smtp_port": 587,
  "username": "user@example.com",
  "password": "secret",
  "imap_server": "imap.example.com",  // optional
  "imap_port": 993                    // optional
}
```

Currently only the SMTP fields are used by the `send_email` tool. The IMAP keys are accepted for future extensions. No default values are provided, so fill in the details that match your mail provider.

If the file is missing the tool outputs `Email configuration not found`. When Taskter runs without a `GEMINI_API_KEY`, email tools are skipped entirely so the file is not required for tests.

## Environment variables

Taskter looks for a couple of optional environment variables:

- `GEMINI_API_KEY` — API key for the Gemini model. When set, agents can call the remote API. If absent or empty Taskter stays in offline mode and only uses built-in tools.
- `SEARCH_API_ENDPOINT` — custom endpoint for the `web_search` tool. Defaults to `https://api.duckduckgo.com`.

Set it directly in your shell or via Docker Compose:

```bash
export GEMINI_API_KEY=your_key_here
```

