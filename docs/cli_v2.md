## rkik CLI v2 design

### Goals
- Split the monolithic flag-based CLI into focused subcommands.
- Offer a persistent config/preset story stored in a TOML file (`$RKIK_CONFIG_DIR` or platform config dir).
- Keep the legacy workflow working (no subcommand = default NTP mode) while encouraging the new UX.

### Subcommands
| Subcommand | Purpose | Key options |
| ---------- | ------- | ----------- |
| `rkik ntp <target>` | NTP probe loop | `--count`, `--interval`, `--timeout`, `--format`, `--json`, `--short`, `--plugin`, `--warning`, `--critical`, `--nts`, `--nts-port`, `--ipv6` |
| `rkik compare <target...>` | concurrent comparison of >=2 servers | inherits probe + output options |
| `rkik ptp <target>` (Linux) | IEEE-1588/802.1AS probing | `--domain`, `--event-port`, `--general-port`, `--hw-timestamp`, `--format`, `--plugin` |
| `rkik sync <target>` | one-shot `--sync` workflow | `--dry-run`, probe options |
| `rkik diag <target>` | verbose troubleshooting helper (single shot) | `--timeout`, `--interval`, `--ipv6` |
| `rkik config <list|get|set|clear|path>` | inspect/update defaults | keys: `default-timeout`, `default-format`, `default-ipv6` |
| `rkik preset <list|add|remove|show|run>` | manage reusable argument presets | `preset add name -- <args...>` stores the trailing args array |

### Config/preset storage
- File: `~/.config/rkik/config.toml` (Linux) or `%APPDATA%\rkik\config.toml` (Windows). Override with `RKIK_CONFIG_DIR`.
- Layout:
  ```toml
  [defaults]
  timeout = 5.0
  format = "json"
  ipv6_only = true

  [presets.nightly]
  args = ["ntp", "pool.ntp.org", "--count", "5"]
  ```
- CLI writes the file lazily (only after `config set`, `config clear`, or `preset add/remove`).
- `preset run` simply spawns `rkik` with the stored argument vector; recursion is allowed but up to the user.

### Compatibility
- Launching `rkik` without a recognized subcommand goes through the legacy parser, so existing scripts keep working (a warning is emitted).
- The legacy parser is now factored into `legacy::run` and can be reused by new commands to avoid duplicating logic.
- `--help`/`--version` now show the v2 subcommand help by default.

