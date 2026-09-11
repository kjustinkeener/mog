# MCP clients

`mog` ships an MCP (Model Context Protocol) server. It is a standard stdio
JSON-RPC server, so **any MCP-capable agent can use it** by launching
`mog mcp`. The installer and `mog mcp install` additionally *auto-register*
the server with the clients listed below by editing their config for you.

## Auto-registered (supported) clients

`mog mcp install` detects and writes the `mog` server into these clients' own
config files:

| Client | Config file |
| --- | --- |
| Claude Code | `~/.claude.json` (via `claude mcp add` when the CLI is on PATH) |
| Claude Desktop | `%APPDATA%\Claude\claude_desktop_config.json` (Windows) |
| Cursor | `~/.cursor/mcp.json` |
| Windsurf | `~/.codeium/windsurf/mcp_config.json` |

A bare `mog mcp install` registers the one client it detects; with several
present it lists them and writes nothing unless you pass `--client <id>` or
`--all`.

## Any other MCP client (manual)

The server is just `mog mcp`. Point any client at it. The command and args are
the same everywhere; only the config file format differs.

JSON clients (most) use an `mcpServers` map:

```json
{
  "mcpServers": {
    "mog": {
      "command": "C:\\Users\\you\\AppData\\Local\\mog\\mog.exe",
      "args": ["mcp"],
      "env": { "MOG_HOME": "C:\\Users\\you\\AppData\\Roaming\\mog" }
    }
  }
}
```

### OpenAI Codex CLI

Codex reads TOML at `~/.codex/config.toml`. Add:

```toml
[mcp_servers.mog]
command = "C:\\Users\\you\\AppData\\Local\\mog\\mog.exe"
args = ["mcp"]

[mcp_servers.mog.env]
MOG_HOME = "C:\\Users\\you\\AppData\\Roaming\\mog"
```

`mog mcp install` prints your exact command path (run it once to copy the
ready-made block). Auto-registration for Codex is planned.
