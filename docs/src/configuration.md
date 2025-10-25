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

Agents use a provider abstraction. Each provider defines its own API key env var:

- `GEMINI_API_KEY` — API key for the Gemini provider.
- `OPENAI_API_KEY` — API key when using an OpenAI provider (if added).
- `SEARCH_API_ENDPOINT` — custom endpoint for the `web_search` tool. Defaults to `https://api.duckduckgo.com`.

Export the relevant variable directly in your shell or via Docker Compose. For Gemini:

```bash
export GEMINI_API_KEY=your_key_here
```
