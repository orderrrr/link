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

Type      | Syntax          | Examples
---       | ---             | ---
Integer   | digits          | `42`, `-3`
Float     | digits`.`digits | `3.14`, `0.5`
String    | `"..."`         | `"hello world"`
List      | space-separated | `1 2 3`, `"a" "b" "c"`
List      | `[`;`-separated]` | `[1;2 3;4 5 6]` (nested)
Boolean   | (internal)      | produced by `=` and comparisons

Negative numbers use `_` as a prefix in some contexts (e.g., `_3` for -3 in terms), or standard `-` as a monadic operator.

## Primitive Functions

### Monadic (unary)

Symbol | Name          | Description                        | Example
---    | ---           | ---                                | ---
`-`    | Negate        | Negates a number                   | `-5` => `-5`
`!`    | Range         | Integers from 0 to n-1             | `!4` => `0 1 2 3`
`=`    | Boolean Flip  | 0 becomes 1, nonzero becomes 0     | `=0` => `1`, `=5` => `0`
`_`    | Floor         | Floors a float to an integer       | `_3.7` => `3`

### Dyadic (binary)

Symbol | Name     | Description                          | Example
---    | ---      | ---                                  | ---
`+`    | Add      | Element-wise addition                | `2 + 3` => `5`
`-`    | Subtract | Element-wise subtraction             | `5 - 2` => `3`
`¯`    | Max      | Maximum of two values                | `3 ¯ 5` => `5`
`_`    | Min      | Minimum of two values                | `3 _ 5` => `3`
`!`    | Modulo   | Remainder                            | `3 ! 10` => `1`
`&`    | Filter   | Filter list by boolean mask          | (see combinators)

List operations are element-wise when both sides are lists of equal length:

```
1 2 3 |+| 4 5 6    -- => 5 7 9
```

### Not Yet Implemented

Symbol | Monadic         | Dyadic
---    | ---             | ---
`+`    | Increment       | --
`×`    | --              | Multiply
`÷`    | --              | Divide
`=`    | --              | Equal

## Combinators (Higher-Order Functions)

Combinators modify how a function is applied.

Symbol | Name   | Description
---    | ---    | ---
`/`    | Fold   | Reduce a list using a dyadic function
`\`    | ScanL  | Apply each-left / outer product

### Fold `/`

Reduces a list to a single value:

```
+/!|10       -- sum of 0..10 => 45
```

The function to the left of `/` is used as the reducer. In a monadic context (applied to a single list), fold reduces the list:

```
|+/!|10      -- range 10, then fold with +
```

### ScanL `\`

Applies a dyadic function across elements. When used dyadically with a list on the left and a value on the right, it maps each element of the left against the right:

```
3 5|\!|10    -- each of [3, 5] modulo'd against range(10)
```

## Trains

Trains are the core composition mechanism. A train is a sequence of functions that are applied right to left to produce a result.

### Monadic Trains

```
|functions|value
```

Functions are applied right to left:

```
|+/!|1000    -- range(1000), then fold with +
```

### Dyadic Trains

```
value|functions|value
```

Both arguments are available to the functions in the train.

### Train Blocks

Parentheses group sub-trains within a larger train:

```
+/(^/=3 5|!.\)|&!|1000
     ^^^^^^^^^^
     this is a sub-train
```

### Dyadic Train Blocks

A value followed by `|` inside a train temporarily makes that segment dyadic:

```
3 5|!\    -- 3 5 is the left arg for the dyadic segment
```

## Function Blocks

User-defined functions use curly braces. The reserved variables are:

Variable | Role
---      | ---
`a`      | Right argument (alpha)
`w`      | Left argument (omega, dyadic only)

### Monadic Function Block `{...}`

```
{-a}|5       -- negate the argument => -5
```

If the block body does not reference `w`, it is treated as monadic.

### Tacit Monadic Function Block `{...|}`

A monadic function block written in tacit (point-free) style:

```
{+/!|}       -- equivalent to |+/!|
```

The trailing `|` marks it as a tacit train applied to the argument.

### Tacit Dyadic Function Block `{|...|}` 

A dyadic function block in tacit style:

```
{|+|}        -- equivalent to w + a
```

### Assignment

Assign a value or function to a name with `:`:

```
x: 42
fn: {+/!|}
3 5 |fn| 1000
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

## Comments

```
-- this is a comment
x: 42 -- inline comment
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
```
