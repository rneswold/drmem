# Logic Node Programming Guide

Logic blocks (and the `expression` driver) use a small expression
language. This document describes it.

An expression computes a value from device readings, constants, the
time-of-day, and the position of the sun. In a logic block, each
expression has the form `EXPRESSION -> {OUTPUT}`, where `{OUTPUT}` is a
variable defined in the block's `outputs` table; the result is sent as a
setting to the associated device. (The `expression` driver omits the
`-> {OUTPUT}` part -- its result goes to its `output` device.)

Whitespace is insignificant between tokens. The keywords `if`, `then`,
`else`, `end`, and the function names `coalesce`, `blend`,
`with_alpha`, and `brightness` are case-insensitive. The operators `not`,
`and`, `or`, `true`, and `false` must be lowercase.

## Primitives

| Form | Description |
|------|-------------|
| `{var}` | The most recent reading of the device associated with `var` in the block's `inputs` table (or in the `expression` driver's `vars` table.) A variable name starts with a letter and contains letters, digits, and underscores. |
| `true`, `false` | Boolean values |
| `123`, `-5` | 32-bit signed integers |
| `1.5`, `-0.25`, `1.0e3` | 64-bit floating point (no +/-inf or NaN.) A float needs a digit before the decimal point. |
| `"string"` | Text. A string can't contain a double quote and has no escape sequences. |
| `#RGB`, `#RGBA`, `#RRGGBB`, `#RRGGBBAA` | RGB color with an optional alpha (opacity) component. If omitted, the color is fully opaque. Each digit is hexadecimal. |
| `#4000K`, `#4000K@128`, `#4000K@50%` | White color temperature in Kelvin (2000 to 6500) with an optional alpha. The alpha is either an integer from 0 to 255 or a percentage from 0% to 100%. |

A negative literal is a single token, so `-` directly followed by a digit
is part of a number. Write `{a} - 1`, not `{a} -1` or `{a}-1`; the
latter two are parse errors.

## Missing Values

An expression doesn't always have a value. A variable has no value until
its device has been read for the first time, the `solar` variables have no
value until the first solar update, and some errors (dividing by zero,
mismatched types, etc.) log a message and yield no value. A value that
is missing makes the expression that uses it missing, except for these:

- `coalesce()` skips missing values.
- `or` and `and` can still yield a value if the other operand determines
  the answer (`true or <missing>` is `true` and `false and <missing>` is
  `false`.)
- `blend()` ignores missing layers.

If an expression has no value, no setting is sent and the output isn't
updated.

## Built-in Variables

The logic block environment tracks the time-of-day and solar position. The
`latitude` and `longitude` in `drmem.toml` are used for the solar
calculations. These are available to expressions through specially named
variables:

| Variable | Type | Description |
|----------|------|-------------|
| `{local:second}` | integer | second (0-59) of local time |
| `{local:minute}` | integer | minute (0-59) of local time |
| `{local:hour}` | integer | hour (0-23) of local time |
| `{local:day}` | integer | day of month (1-31) of local time |
| `{local:month}` | integer | month (1-12) of local time |
| `{local:year}` | integer | year of local time |
| `{local:DOW}` | integer | day of week (0-6, 0 is Monday) of local time |
| `{local:DOY}` | integer | day of year (0-364) of local time. Leap years are ignored: Feb 29th has the same value as Feb 28th and every later date has the same value as in a non-leap year. |
| `{local:SOM}` | integer | which occurrence (1-5) of the current day of the week it is, counting from the start of the month. The first Monday of the month is 1, the second is 2, etc. |
| `{local:EOM}` | integer | which occurrence (1-5) of the current day of the week it is, counting from the end of the month. The last Monday of the month is 1, the second to last is 2, etc. |
| `{local:LY}` | boolean | `true` if it is a leap year in local time |
| `{utc:second}` | integer | second (0-59) of UTC time |
| `{utc:minute}` | integer | minute (0-59) of UTC time |
| `{utc:hour}` | integer | hour (0-23) of UTC time |
| `{utc:day}` | integer | day of month (1-31) of UTC time |
| `{utc:month}` | integer | month (1-12) of UTC time |
| `{utc:year}` | integer | year of UTC time |
| `{utc:DOW}` | integer | day of week (0-6, 0 is Monday) of UTC time |
| `{utc:DOY}` | integer | day of year (0-364) of UTC time, ignoring leap years as described above |
| `{utc:SOM}` | integer | occurrence of the day of the week, counting from the start of the month, in UTC |
| `{utc:EOM}` | integer | occurrence of the day of the week, counting from the end of the month, in UTC |
| `{utc:LY}` | boolean | `true` if it is a leap year in UTC |
| `{solar:alt}` | float | elevation of the sun in degrees (90 to -90.) Negative values mean the sun is below the horizon. |
| `{solar:az}` | float | azimuth of the sun in degrees (0 to 360) |
| `{solar:dec}` | float | declination of the sun in degrees |
| `{solar:ra}` | float | right ascension of the sun in degrees |

An expression is re-evaluated when one of its inputs changes. The time
variables only trigger an evaluation when the *shortest* time unit the
expression uses changes: an expression that only uses `{local:hour}` is
evaluated once an hour, not once a second. Solar values are updated every
15 seconds.

## Operators

Operators are listed from the lowest to the highest precedence. Use
parentheses to group subexpressions.

| Operator | Description |
|----------|-------------|
| `A or B` | Boolean OR. |
| `A and B` | Boolean AND. |
| `A = B`, `A <> B`, `A < B`, `A <= B`, `A > B`, `A >= B` | Comparisons; the result is a boolean. They can't be chained (`a < b < c` is an error); use `and`. |
| `A + B`, `A - B` | Addition and subtraction. |
| `A * B`, `A / B`, `A % B` | Multiplication, division, and remainder. |
| `not A` | Complements the boolean `A`. It applies to the single factor that follows it, so `not {a} = {b}` is `(not {a}) = {b}`. Write `not ({a} = {b})` for the other meaning. |

`or`, `and`, `+`, `-`, `*`, `/`, and `%` group to the *right* when
chained with operators of the same precedence: `10 - 3 - 2` is
`10 - (3 - 2)`, or 9, and `16 / 4 / 2` is 8. Always use parentheses when
mixing operators of the same precedence.

### Types

- `+`, `-`, `*`: both operands must be numbers (integers or floats.)
  Booleans are accepted and count as 0 and 1. If either operand is a
  float, the result is a float; otherwise it is an integer.
- `/`: integers, floats, or a mix. Two integers perform integer
  division. A zero divisor produces no value.
- `%`: integers, floats, or a mix. The divisor must be greater than zero
  or there is no value.
- `=` and `<>`: booleans, numbers, or strings. An integer and float can be
  compared. Anything else (colors, mismatched types) produces no value.
- `<`, `<=`, `>`, `>=`: numbers or strings (strings compare
  lexicographically.) Booleans and colors can't be ordered.
- `not`, `and`, `or`: booleans only. `and` and `or` stop evaluating as soon
  as the answer is known.

The type of the final value must match the type of the output device or
the device will reject the setting.

## Conditional

```
if CONDITION then EXPR end
if CONDITION then EXPR else EXPR end
```

`CONDITION` must be a boolean. If it's true, the result is the first
`EXPR`. If it's false, the result is the `else` expression or, if there is
no `else`, there is no value. Conditionals can be nested and used
anywhere a value can be.

```
if {temp} > 75.0 then true else false end -> {fan}
```

## Functions

| Function | Description |
|----------|-------------|
| `blend(TOP, ..., BOTTOM)` | Composites colors using their alpha values. The first argument is the top layer and the last is the bottom layer. Arguments with no value are ignored. Two color temperatures blend to a color temperature; if an RGB color is involved, the result is an RGB color. If there are no layers, the result is transparent black (`#00000000`.) |
| `brightness(COLOR)` | Returns the perceived brightness of a color as a float. For an RGB color it is the luminance (0.0 to 255.0) scaled by the alpha. For a color temperature, it is the alpha value (0.0 to 255.0.) The argument must be a color. It must be a single term; put an operator expression in parentheses. |
| `coalesce(A, B, ...)` | Returns the first argument that has a value. If none has a value, there's no value. |
| `with_alpha(COLOR, ALPHA)` | Returns `COLOR` with a new alpha (opacity). `ALPHA` must be a float; it is clamped to the range 0.0 (transparent) to 1.0 (opaque.) |
