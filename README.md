# Link ( *l is not k* )

Link is an array-oriented programming language with heavy inspiration from [K](https://k-project.fandom.com/wiki/Home) (mainly k6) and [BQN](https://mlochbaum.github.io/BQN/).

```
+/!|1000
```
> Sum all numbers from 0 to 1000.

## Getting Started

Link is written in Rust. Build with:

```sh
cargo build
```

Run a file:

```sh
cargo run --bin link
```

Start the REPL:

```sh
cargo run --bin repl
```

Run the test suite:

```sh
cargo test
```

## Language Overview

A link program is built from the following concepts:

- **Program** -- a list of expressions separated by `;` or newlines
- **Expression** -- one of:
  1. A monadic expression (a monadic train applied to a single value)
  2. A dyadic expression (a dyadic train applied to two values)
  3. An assignment expression
  4. A value
- **Value (unit)** -- data: integers, floats, strings, lists, function blocks
- **Train** -- one or more functions chained together, evaluated right to left
- **Function** -- either a primitive operator or a user-defined function block

### A Quick Example

```
+/!|1000
```

Reading right to left through the train:

1. `1000` -- the input value
2. `!` -- range: produces `0 1 2 3 ... 999`
3. `+/` -- fold with `+`: sums the entire list

Result: `499500`

### The Pipe `|`

The pipe symbol is central to Link's syntax. It disambiguates how expressions are parsed:

- **Monadic expression**: `train|value` -- apply the train to one argument
  ```
  -|5          -- negate 5 => -5
  +/!|1000     -- sum of range 1000
  ```
- **Dyadic expression**: `value|train|value` -- apply the train to two arguments
  ```
  2 |+| 3      -- 2 + 3
  ```

Without the pipe, there is ambiguity between whether a token is a function in a train or a value argument -- a fundamental problem in K-like languages where user functions and primitives look different. Link solves this with explicit pipe delimiters.

```
f: 3 5; f|=!|5     -- f is a value, used as left arg of a dyadic train
f: {=}; f!|1 0 1 0 -- f is a function, used monadically in a train
```

## Data Types

Type      | Syntax            | Examples
---       | ---               | ---
Integer   | digits            | `42`, `-3`
Float     | digits`.`digits   | `3.14`, `0.5`
String    | `"..."`           | `"hello world"`
List      | space-separated   | `1 2 3`, `"a" "b" "c"`
List      | `[`;`-separated]` | `[1;2 3;4 5 6]` (nested)
Boolean   | (internal)        | produced by `=` and comparisons

## Primitives

Every primitive has a symbol, an ASCII alias (usable in the REPL), and up to two meanings depending on whether it is applied monadically (one argument) or dyadically (two arguments).

### Operators

Symbol | Alias | Name       | Monadic (unary)               | Dyadic (binary)
---    | ---   | ---        | ---                           | ---
`+`    | `add` | Plus       | *not yet implemented*         | Add
`-`    | `neg` | Minus      | Negate                        | Subtract
`×`    | `mul` | Times      | *not yet implemented*         | *not yet implemented*
`÷`    | `div` | Divide     | *not yet implemented*         | *not yet implemented*
`¯`    | `max` | Max        | *not yet implemented*         | Maximum of two values
`_`    | `min` | Min/Floor  | Floor (float to int)          | Minimum of two values
`=`    | `eq`  | Equal/Flip | Boolean flip (0 becomes 1)    | *not yet implemented*
`&`    | `amp` | Amp        | *not yet implemented*         | Filter by boolean mask
`!`    | `mod` | Bang       | Range (0 to n-1)              | Modulo
`ρ`    | `rho` | Rho        | Create zeroed array by shape  | Reshape data to shape

### Modifiers

Symbol | Name             | Description
---    | ---              | ---
`:`    | Monadic Override | Forces an operator to be monadic inside a dyadic train

In a dyadic train (`value|train|value`), all operators default to dyadic. Suffix an operator with `:` to force it monadic -- it will only receive the right argument.

```
3 2|ρ!:|6              -- !: forces ! to be monadic (range)
                       -- so !: takes only 6 => 0 1 2 3 4 5
                       -- then ρ reshapes dyadically with 3 2
                       -- result: 3 wide, 2 tall matrix
```

Without `:`, the `!` in `3 2|ρ!|6` would be dyadic (modulo), receiving both `3 2` and `6`.

### Combinators

Symbol | Alias   | Name  | Description
---    | ---     | ---   | ---
`/`    | `fold`  | Fold  | Reduce a list with a dyadic function
`\`    | `scanl` | ScanL | Each-left / outer product
`ǁ`    | `each`  | Each  | *not yet implemented*

### REPL Aliases

In the REPL, you can type the ASCII alias instead of the unicode symbol. It will be replaced with the symbol when you press Enter. For example, typing `rho5` is equivalent to `ρ5`.

Alias   | Symbol | Name
---     | ---    | ---
`add`   | `+`    | Plus
`neg`   | `-`    | Minus
`mul`   | `×`    | Times
`div`   | `÷`    | Divide
`max`   | `¯`    | Max
`min`   | `_`    | Min/Floor
`eq`    | `=`    | Equal/Flip
`amp`   | `&`    | Amp
`mod`   | `!`    | Bang
`rho`   | `ρ`    | Rho
`mon`   | `:`    | Monadic Override
`fold`  | `/`    | Fold
`scanl` | `\`    | ScanL
`each`  | `ǁ`    | Each

## Operators in Detail

### `-` Negate / Subtract

```
-2                  -- => -2
1 - 2               -- => -1 (note: dyadic uses pipe syntax)
```

### `!` Range / Modulo

```
!4                  -- => 0 1 2 3
```

### `=` Boolean Flip

```
=0                  -- => 1
=5                  -- => 0
=0 1 0 1            -- => 1 0 1 0
```

### `_` Floor / Min

```
_3.7                -- => 3 (floor)
```

### `&` Filter

Used dyadically with a boolean mask on the left and a list on the right:

```
-- filter keeps elements where mask is true
```

### `ρ` Shape / Reshape

Monadic `ρ` creates a zeroed array from a shape description. Arguments are x y (width, height):

```
ρ 5                 -- => 0 0 0 0 0
ρ 3 2               -- => 3 wide, 2 tall:
                    --    0 0 0
                    --    0 0 0
```

Dyadic `ρ` reshapes data into the given shape. Left is shape (x y), right is data:

```
3 2|ρ|0 1 2 3 4 5;  -- => 3 wide, 2 tall:
                     --    0 1 2
                     --    3 4 5

3 3|ρ|0 1 2 3;      -- data cycles to fill:
                     --    0 1 2
                     --    3 0 1
                     --    2 3 0

5 5|ρ|0;             -- scalar fills entire array:
                     --    0 0 0 0 0
                     --    0 0 0 0 0
                     --    0 0 0 0 0
                     --    0 0 0 0 0
                     --    0 0 0 0 0
```

Compose `ρ` with `!` using the monadic override `:` to reshape a range:

```
3 2|ρ!:|6            -- !: forces range (monadic) on 6
                     -- then ρ reshapes to 3 wide, 2 tall:
                     --    0 1 2
                     --    3 4 5

5 5|ρ!:|25           -- 5x5 matrix of 0..24:
                     --     0  1  2  3  4
                     --     5  6  7  8  9
                     --    10 11 12 13 14
                     --    15 16 17 18 19
                     --    20 21 22 23 24
```

## Combinators in Detail

### Fold `/`

Reduces a list to a single value:

```
+/!|10              -- sum of 0..10 => 45
```

The function to the left of `/` is used as the reducer.

### ScanL `\`

Applies a dyadic function across elements. When used dyadically with a list on the left and a value on the right, it maps each element of the left against the right:

```
3 5|\!|10           -- each of [3, 5] modulo'd against range(10)
```

## Trains

Trains are the core composition mechanism. A train is a sequence of functions that are applied right to left to produce a result.

### Monadic Trains

```
|functions|value
```

Functions are applied right to left:

```
|+/!|1000           -- range(1000), then fold with +
```

### Dyadic Trains

```
value|functions|value
```

Both arguments are available to the functions in the train.

### Train Blocks

Parentheses group sub-trains within a larger train:

```
+/(¯/=3 5|!\)|&!|1000
```

### Dyadic Train Blocks

A value followed by `|` inside a train temporarily makes that segment dyadic:

```
3 5|!\              -- 3 5 is the left arg for the dyadic segment
```

## Function Blocks

User-defined functions use curly braces. The reserved variables are:

Variable | Role
---      | ---
`a`      | Right argument (alpha)
`w`      | Left argument (omega, dyadic only)

### Monadic Function Block `{...}`

```
{-a}|5              -- negate the argument => -5
```

If the block body does not reference `w`, it is treated as monadic.

### Tacit Monadic Function Block `{...|}`

A monadic function block written in tacit (point-free) style:

```
{+/!|}              -- equivalent to |+/!|
```

The trailing `|` marks it as a tacit train applied to the argument.

### Tacit Dyadic Function Block `{|...|}`

A dyadic function block in tacit style:

```
{|+|}               -- equivalent to w + a
```

### Assignment

Assign a value or function to a name with `:`:

```
x: 42
fn: {+/!|}
3 5 |fn| 1000
```

## Array Display

2D arrays are displayed as right-aligned grids:

```
>> 3 2|ρ|1 2 3 10 20 30;
 1  2  3
10 20 30
```

1D lists are displayed space-separated:

```
>> !5
0 1 2 3 4
```

## Blocks and Grouping

Syntax      | Purpose
---         | ---
`{...}`     | Function block (monadic if no `w`, dyadic otherwise)
`{...\|}`   | Tacit monadic function block
`{\|...\|}` | Tacit dyadic function block
`[...;...]` | List block (`;`-separated elements, supports nesting)
`(...)`     | Expression/train grouping block
`"..."`     | String literal (double `""` for escape)
`op:`       | Monadic override (force operator to monadic in dyadic context)

## Comments

```
-- this is a comment
x: 42 -- inline comment
```

## Error Handling

Type mismatches and unsupported operations produce runtime errors instead of crashing:

```
>> -"foo"
runtime error: - (negate) expects int or float, got string

>> !"hello"
runtime error: ! (range) expects int, got string
```

## VM Architecture

Link compiles source code through a three-stage pipeline:

```
Source -> Parser (PEG) -> AST -> Bytecode Compiler -> VM -> Result
```

The VM is stack-based with 14 opcodes:

Opcode  | Code | Operand        | Description
---     | ---  | ---            | ---
`CONST` | `01` | `u16` index    | Push constant from variable store
`POP`   | `02` |                | Pop top of stack
`JMP`   | `03` | `u16` address  | Jump instruction pointer
`GETL`  | `04` |                | Get left variable (`w`)
`GETR`  | `05` |                | Get right variable (`a`)
`CRVAR` | `06` |                | Create variable
`CLVAR` | `07` |                | Clear variable (pop and print)
`DUP`   | `08` | `u16` address  | Duplicate top of stack
`MBL`   | `09` | `u16` address  | Start monadic block
`DBL`   | `0A` | `u16` address  | Start dyadic block
`END`   | `0B` |                | End block
`MO`    | `0C` | `u8` fn-id     | Monadic function
`DO`    | `0D` | `u8` fn-id     | Dyadic function
`CO`    | `0E` | `u8` cn-id     | Combinator

The VM uses a value stack for computation and a context stack for tracking block nesting and return addresses.

## Working Examples

These are tested and verified:

```
-- negate
-2                      -- => -2

-- arithmetic
2 + 2                   -- => 4
1 - 2                   -- => -1

-- range
!4                      -- => 0 1 2 3

-- fold (sum of range)
|+/!|10                 -- => 45

-- strings
"hello world"           -- => "hello world"

-- create a 3x2 zero matrix
ρ 3 2                   -- => 0 0 0
                        --    0 0 0

-- reshape with explicit data
3 2|ρ|0 1 2 3 4 5;     -- => 0 1 2
                        --    3 4 5

-- reshape with range using monadic override
3 2|ρ!:|6              -- => 0 1 2
                        --    3 4 5

-- scalar fill
2 2|ρ|7;               -- => 7 7
                        --    7 7
```
