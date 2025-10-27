# Plugin mode (Centreon / Nagios / Zabbix)

Since v1.2, rkik provides a plugin mode to integrate easily with traditional monitoring systems (Centreon, Nagios, Zabbix).

This mode emits a single machine-readable line (the plugin output) and returns standard plugin exit codes so monitoring servers can interpret the result.

## Usage

Example:

```bash
rkik time.google.com --plugin --warning 400 --critical 1000
```

Flags

- `--plugin` : enable plugin mode.
- `--warning <MS>` : warning threshold in milliseconds (requires `--plugin`).
- `--critical <MS>` : critical threshold in milliseconds (requires `--plugin`).

## Output format

The plugin line follows this structure:

```
RKIK <STATE> - offset <offset_ms>ms rtt <rtt_ms>ms from <hostname> (<ip>) | offset_ms=<offset_ms>ms;<warn>;<crit>;0; rtt_ms=<rtt_ms>ms;;;0;
```

- `<STATE>` is one of `OK`, `WARNING`, `CRITICAL`, `UNKNOWN`.
- Perfdata fields follow the `label=value[UOM];warn;crit;min;max` convention used by Nagios plugins.

Example:

```
RKIK OK - offset 4.006ms rtt 9.449ms from time.google.com (216.239.35.4) | offset_ms=4.006ms;400;1000;0; rtt_ms=9.449ms;;;0;
```

## Exit codes

- `0` (OK): request succeeded and `|offset| < warning` (or no thresholds provided).
- `1` (WARNING): request succeeded and `|offset| >= warning` and `< critical`.
- `2` (CRITICAL): request succeeded and `|offset| >= critical`.
- `3` (UNKNOWN): request failed or no usable result (error cases).

## Behavior notes

- The plugin mode suppresses the usual human-readable output; only the plugin line is printed.
- Thresholds are interpreted in milliseconds and compare the absolute value of the clock offset.
- By default, perfdata includes the `ms` unit in the value (e.g., `offset_ms=4.006ms`). If you prefer strictly numeric perfdata values (no unit), request a change and we can provide an option to produce numeric-only perfdata.
- `--plugin` is currently not supported with `--compare` (multi-host compare). If you need plugin-style output for comparisons, specify how you want to aggregate results per host (average, max, or per-host multi-line output).

## Examples

OK example:

```
RKIK OK - offset 4.006ms rtt 9.449ms from time.google.com (216.239.35.4) | offset_ms=4.006ms;400;1000;0; rtt_ms=9.449ms;;;0;
```

Unknown example (network error):

```
RKIK UNKNOWN - request failed | offset_ms=;400;1000;0; rtt_ms=;;;0;
```
