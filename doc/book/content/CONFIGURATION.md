# Configuration

## Overview

`drmemd` is configured with a single [TOML](https://toml.io) file named
`drmem.toml`. The file is read once, when `drmemd` starts. Changes
require a restart.

`drmemd` looks for the file in these locations, in order:

1. `./drmem.toml` (the current directory)
2. `$HOME/.drmem.toml` (note the leading period)
3. `/usr/local/etc/drmem.toml`
4. `/usr/pkg/etc/drmem.toml`
5. `/etc/drmem.toml`

The first file that exists is the one used. If it contains errors,
`drmemd` reports them and exits; it does not fall back to the next
location. If no file is found, `drmemd` starts with a default
configuration (no instantiated drivers or logic blocks.)

Two command line options are useful when working on a configuration:

- `--print-config` displays the loaded configuration (the drivers are
  shown after templates have been expanded) and exits.
- `-v` sets the log level to `info`. Repeating it (`-vv`, `-vvv`)
  raises the level to `debug` and `trace`. These override `log_level`
  in the configuration file.

A configuration file is made up of a few top-level fields followed by
tables and arrays of tables:

```toml
log_level = "info"
latitude = 41.88
longitude = -87.63

[backend]
# ...

[graphql]
# ...

[[driver]]
# ...

[[logic]]
# ...
```

## Device Values in TOML

Many configuration fields hold a value that will be sent to, or
compared with, a DrMem device (the `default` of a `map` driver, the
`enabled` and `disabled` values of a `timer` driver, etc.) DrMem
devices can be one of five types, but TOML only has a few native
types, so the configuration parser uses some conventions to select the
device type from the TOML value:

| TOML value                    | Device type | Notes                                                                           |
|-------------------------------|-------------|---------------------------------------------------------------------------------|
| `true`, `false`               | boolean     |                                                                                 |
| `5`, `-12`                    | integer     | Signed, 32-bit. Values outside the range of an `i32` are an error.              |
| `5.0`, `-1.5e3`               | float       | A number with a decimal point or exponent is a float, even if it is a whole number. |
| `"hello"`                     | string      | Any string that isn't a color (see below).                                      |
| `"#RRGGBB"`, `"#RRGGBBAA"`    | color       | RGB, with an optional alpha. Each component is two hex digits (either case.)    |

Because `5` and `5.0` are different types, make sure the type in the
configuration matches the type the driver expects.

Colors are the trickiest convention because TOML has no color type.
Instead, a color is written as a string that starts with `#` and is
followed by exactly six or eight hex digits. When the alpha component is
omitted, it is fully opaque (`ff`). A string that begins with `#` but
doesn't match this pattern (wrong length or a non-hex digit) is *not* an
error; it is treated as an ordinary string. If you expect a color and
the device rejects the value, check the string for typos.

```toml
cfg.default = "#ff8000"      # orange, fully opaque
cfg.default = "#ff800080"    # orange, 50% opaque
cfg.default = "#ff800"       # a string, not a color!
```

Only RGB(A) colors can be specified in the configuration file.
(Expressions in logic blocks can additionally use color temperatures
such as `#4000K`; see the [Logic Node Programming Guide](LN_GRAMMAR.md).)

Device *names* are not values; they're always strings. A device name is
a series of segments separated by colons (e.g. `weather:temperature`).
A segment is made up of one or more letters, digits, or dashes
(`-` can't be the first or last character of a segment.) The last
segment is the device's base name and the preceding segments are its
path.

## Top-Level Fields

| Field       | Type   | Default  | Description                                                           |
|-------------|--------|----------|-----------------------------------------------------------------------|
| `log_level` | string | `"warn"` | One of `"warn"`, `"info"`, `"debug"`, or `"trace"`. Unrecognized values select `"warn"`. |
| `latitude`  | float  | required | Location of this DrMem instance in decimal degrees, from -90 to 90 (north is positive.) |
| `longitude` | float  | required | Location in decimal degrees, from -180 to 180 (east is positive.)     |

The latitude and longitude are used to compute the position of the sun,
which is available to expressions as `{solar:alt}`, `{solar:az}`,
`{solar:ra}`, and `{solar:dec}`. They are required whenever a
configuration file is used.

## `[backend]`

The storage backend is selected when `drmemd` is built (see
[Storage Backends](BACKENDS.md)). The `[backend]` table only has
fields for the redis backend. The simple backend has no configuration
and the table can be omitted.

| Field  | Type    | Default                                       | Description                         |
|--------|---------|-----------------------------------------------|-------------------------------------|
| `addr` | string  | `"127.0.0.1:6379"`                            | Address and port of the redis server. |
| `dbn`  | integer | `0` (`1` in debug builds)                     | The redis database number to use.   |

Redis supports multiple, separate databases. If you want to run DrMem to try something new, but don't want to affect your current, archived data, you can specify a new database.

## `[graphql]`

This table is only used if `drmemd` was built with the `graphql`
feature. All of its fields are optional.

| Field       | Type   | Default                | Description                                                      |
|-------------|--------|------------------------|------------------------------------------------------------------|
| `name`      | string | `"unknown name"`       | A name for this instance, shown to clients and in the mDNS announcement. |
| `location`  | string | `"unknown location"`   | A description of where this instance lives.                      |
| `addr`      | string | `"0.0.0.0:3000"`       | Address and port on which the server listens.                    |
| `pref_host` | string | none                   | A preferred host name for clients. If set, it is added to the mDNS announcement. |
| `pref_port` | integer | `3000`                | The port that goes with `pref_host`.                             |
| `security`  | table  | none                   | If present, the server uses TLS and requires client authentication (see below.) |

The `security` table has these required fields:

| Field       | Type            | Description                                                        |
|-------------|-----------------|--------------------------------------------------------------------|
| `cert_file` | string (path)   | The server's certificate file (PEM).                               |
| `key_file`  | string (path)   | The server's private key file (PEM).                               |
| `clients`   | array of string | The fingerprints of the clients that are allowed to connect. Clients identify themselves with the `X-DrMem-Client-Id` header. |

```toml
[graphql]
name = "Home"
location = "Basement"
addr = "0.0.0.0:3000"

[graphql.security]
cert_file = "/etc/drmem/cert.pem"
key_file = "/etc/drmem/key.pem"
clients = ["..."]
```

## `[[driver]]`

Each `[[driver]]` entry starts an instance of a driver. The drivers are
started in the order they appear in the file. A driver may rely on
devices created by an earlier driver, so list the drivers that produce
devices before the ones that consume them (the `expression` driver is an
example.)

| Field         | Type    | Default  | Description                                                    |
|---------------|---------|----------|----------------------------------------------------------------|
| `name`        | string  | required | The driver to use (e.g. `"timer"`, `"ntp"`). The same driver can be used by more than one entry. |
| `prefix`      | string  | required | The path prepended to the names of every device this instance creates. It is a device path: segments separated by colons. |
| `max_history` | integer | none     | A hint of how many readings to keep for each of this instance's devices. The backend decides how to interpret it; the simple backend only ever keeps the last reading. |
| `cfg`         | table   | empty    | Driver-specific parameters. See the driver's documentation.    |

The names of a driver's devices are `prefix:base`, where `base` is
defined by the driver. Two devices can't have the same full name, so
each instance of a driver needs a unique `prefix`.

```toml
[[driver]]
name = "timer"
prefix = "basement:sump-alarm"
max_history = 1000
cfg = { millis = 5000, enabled = true, disabled = false }
```

## `[[logic]]`

Each `[[logic]]` entry defines a logic block: a task that reads devices,
evaluates expressions, and sends settings to other devices. Logic blocks
are started after all the drivers.

| Field     | Type            | Default  | Description                                                    |
|-----------|-----------------|----------|----------------------------------------------------------------|
| `name`    | string          | required | The name of the block.                                         |
| `summary` | string          | none     | A short description, available to GraphQL clients.             |
| `inputs`  | table           | empty    | Maps a variable name to the name of a device to be read.       |
| `defs`    | table           | empty    | Maps a variable name to an expression. These are intermediate values calculated from `inputs` each time they change and may be used in `exprs`. A definition can use `inputs` but not other definitions. |
| `outputs` | table           | required | Maps a variable name to the name of a device to be set. At least one output is required. |
| `exprs`   | array of string | required | The expressions to evaluate. At least one is required. Each has the form `expression -> {output}`. |

Variable names must be unique within `inputs` and `defs`, as must
`outputs`' names, and a device may only be used once in `inputs` and
once in `outputs`. In an expression, `{name}` refers to a variable and
`{category:field}` refers to a built-in time or solar value.

```toml
[[logic]]
name = "to-f"
inputs = { temp_c = "weather:temperature" }
outputs = { temp_f = "weather:temperature-f" }
exprs = ["{temp_c} * 1.8 + 32.0 -> {temp_f}"]
```

## Templates

When many `[[driver]]` or `[[logic]]` entries differ only in a few
values, a template can generate them. An entry that contains *only* a
`template` field is expanded, when the configuration is loaded, into
zero or more entries that replace it (in the same position in the file.)
The template table has exactly two fields:

- `vars` defines the substitutions.
- `body` is a string of TOML text in which each `${var}` is replaced.

```toml
[[driver]]
template = { vars = { p = ["a", "b"] }, body = '''
name = "timer"
prefix = "${p}"
cfg = { millis = 1000, enabled = true, disabled = false }
''' }
```

The example above is the same as writing two `[[driver]]` entries, one
with `prefix = "a"` and one with `prefix = "b"`. Use
`drmemd --print-config` to see the result of expanding driver
templates.

How `vars` is expanded:

- A table maps each variable to an array. All the arrays must be the
  same length and are consumed in parallel; each index produces one
  expansion.
- An array of such tables produces the cross product of the tables.
  Variable names must be unique across all the tables.

How a value is substituted:

- A string is inserted as-is, *without* quotes. Put quotes around the
  placeholder when you want a string: `prefix = "${p}"`.
- Any other TOML value is inserted in TOML notation: `index = ${n}`
  with `n = [1, 2]` yields `index = 1` and `index = 2`.
- A `${...}` placeholder that doesn't name a variable is left alone.

The expanded text is parsed as the *contents of a single entry*. It is
not a fragment of the file, so it must generate exactly the fields that
the section being expanded requires:

- In a `[[driver]]`, the body must provide at least `name` and
  `prefix`, and can provide `max_history` and `cfg`.
- In a `[[logic]]`, the body must provide at least `name`, `exprs`, and
  `outputs`, and can provide `summary`, `inputs`, and `defs`.
- Don't put `[[driver]]`, `[[logic]]`, or `[section]` headers in the
  body. Use dotted keys or inline tables for nested values (e.g.
  `cfg.millis = 1000` or `cfg = { millis = 1000 }`).
- Fields the section doesn't define are silently ignored, so a
  misspelled field name in a body shows up as a missing field.

Templates are checked when the configuration is loaded. A bad `vars`
definition, text that isn't valid TOML after expansion, or an expansion
that is missing a required field or has a field of the wrong type is a
configuration error. The message is prefixed with the section and the
index of the template entry that caused it (e.g. `driver[2]: ...`).
The same rules apply to every expansion: each one must be a complete,
valid definition.

Remember that device names must still be unique after expansion, so
anything that identifies a device (such as `prefix`) needs to come from
a variable that differs for each expansion.

```toml
[[logic]]
template = { vars = { n = ["a", "b"], d = ["dev:a", "dev:b"] }, body = '''
name = "copy-${n}"
inputs = { v = "in:${n}" }
outputs = { o = "${d}" }
exprs = ["{v} -> {o}"]
''' }
```
