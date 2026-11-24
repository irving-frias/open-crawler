# OpenCrawler MCP server + headless crawl runner

`open-crawler-mcp` exposes the crawler to MCP clients (Claude Desktop, agents,
CI) over stdio, and can run a crawl from a JSON config file without any
client. It shares the desktop app's SQLite database, so projects and results
are the same ones the GUI sees.

## Build

```sh
cd src-tauri
cargo build --release --bin open-crawler-mcp
# binary: target/release/open-crawler-mcp
```

## Database

Default: the desktop app's database
(`<data_dir>/com.opencrawler.app/open-crawler.db` — on Linux
`~/.local/share/com.opencrawler.app/open-crawler.db`, macOS
`~/Library/Application Support/com.opencrawler.app/open-crawler.db`,
Windows `%APPDATA%\com.opencrawler.app\open-crawler.db`).

Override with `--db PATH` or the `OPEN_CRAWLER_DB` environment variable (for
CI or a throwaway database). The database opens in WAL mode, so the MCP
server and the desktop app can run at the same time.

## MCP server (stdio)

```sh
open-crawler-mcp            # serve on stdio (default)
open-crawler-mcp serve [--db PATH]
```

Client configuration (Claude Desktop `claude_desktop_config.json`, or any
MCP client that spawns a stdio server):

```json
{
  "mcpServers": {
    "open-crawler": {
      "command": "/path/to/open-crawler-mcp",
      "args": []
    }
  }
}
```

### Tools

| Tool | Arguments | Description |
| ---- | --------- | ----------- |
| `list_projects` | — | All crawl projects (`id`, `name`). |
| `start_crawl` | `project_id`, `config` (CrawlConfig object) | Start a crawl in the background. Returns immediately; poll `get_crawl_status`. Errors if a crawl is already running for the project. |
| `get_crawl_status` | `project_id` | Running crawl progress (`urls_crawled`, `urls_queued`, `current_url`, `errors`, `elapsed_secs`), or `null` when idle. |
| `get_results` | `project_id`, `page` (1), `page_size` (50, max 500) | Paginated light result rows with SEO score, plus `total`. |
| `get_page_detail` | `project_id`, `url` | Full detail of the most recent crawl of one URL: page fields, SEO audit, semantic issues, links. |

`config` is a `CrawlConfig` object: `seed_urls`, `url_list` (list mode, T3),
`max_depth`, `respect_robots`, `check_sitemap`, `render_js`, `max_crawl_time_secs`,
`concurrency`, `delay_ms`, `same_origin_only`, `include_patterns`,
`exclude_patterns`, `custom_headers`, `cookies`, `site_auth`, `proxy`, … —
omitted fields take the defaults. Example:

```json
{
  "project_id": "…",
  "config": {
    "seed_urls": ["https://example.com"],
    "max_depth": 5
  }
}
```

## Headless crawl from a config file

```sh
open-crawler-mcp crawl --config crawl.json [--db PATH] [--project-name NAME]
```

`crawl.json`:

```json
{
  "project_id": "optional — reuse an existing project",
  "project_name": "optional — created when there is no project_id",
  "config": {
    "seed_urls": ["https://example.com"],
    "max_depth": 5
  }
}
```

Without `project_id` a project is created (name from `--project-name`, then
`project_name`, then the host of the first seed URL). The command runs the
crawl to completion and prints a JSON summary on stdout:

```json
{"project_id": "…", "pages_crawled": 128, "elapsed_secs": 42}
```

Exit code 0 on success, 1 on failure (error on stderr). Logs go to stderr,
so stdout stays machine-readable.

## Notes

- Only `start_crawl` writes; the other tools are read-only. There is no tool
  to delete projects or results.
- The server is stdio-only: no port is exposed, no authentication — the user
  launches it on their own machine.
- Scheduled crawls and their alert notifications are a desktop-app feature;
  headless crawls skip the desktop notification (the snapshot is still
  created).
- The binary is not bundled with the desktop app installer in this version.
