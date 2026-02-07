# Link ( *l is not k* )

Link is an array-oriented programming language with heavy inspiration from [K](https://k-project.fandom.com/wiki/Home) (mainly k6) and [BQN](https://mlochbaum.github.io/BQN/). Link uses S-expression (Lisp-style) syntax with array-oriented semantics.

```
(+/! 10)
```
> Sum all numbers from 0 to 9 = 45.

## Getting Started

Link is written in Rust. Build with:

```sh
cargo build
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

A Link program is a single S-expression. Everything is built from parenthesized expressions:

```
(↻
  (: square (λ (x) (× x x)))
  (: data (! 5))
  (square data)
)
```

### Core Concepts

- **S-expression** -- `(head elements...)` -- the universal syntax form
- **Atom** -- a bare value: integer, float, string, or name
- **Application** -- `(operator args...)` -- apply a function/train to arguments
- **Train** -- a chain of operators/combinators, evaluated right to left
- **List literal** -- `(1 2 3)` -- first element is a value, not an operator
- **Lambda** -- `(λ (params...) body...)` -- function definition
- **Do-block** -- `(↻ expr1 expr2 ... exprN)` -- sequence expressions, return last
- **Assignment** -- `(: name expr)` -- bind a value to a name

### Disambiguation Rule

The first element of a parenthesized expression determines its type:

First element         | Meaning
---                   | ---
Operator (`+`, `-`, `ρ`, etc.) | Application
Train (`+/!`, etc.)   | Application
Name (`foo`, `square`)| Function call
`λ`                   | Lambda definition
`↻`                   | Do-block
`:`                   | Assignment
Number, float, string | List literal

### A Quick Example

```
(+/! 1000)
```

Reading the train `+/!` right to left:

1. `!` -- range: produces `0 1 2 3 ... 999`
2. `+/` -- fold with `+`: sums the entire list

Result: `499500`

## Data Types

Type      | Syntax            | Examples
---       | ---               | ---
Integer   | digits            | `42`, `-3`
Float     | digits`.`digits   | `3.14`, `0.5`
String    | `"..."`           | `"hello world"`
List      | `(values...)`     | `(1 2 3)`, `("a" "b" "c")`
2D array  | `((rows...)...)`  | `((1 2 3) (4 5 6))`
Boolean   | (internal)        | produced by `=` and comparisons

## Primitives

Every primitive has a symbol, an ASCII alias (usable in the REPL), and up to two meanings depending on whether it is applied monadically (one argument) or dyadically (two arguments).

### Operators

Symbol | Alias | Name       | Monadic (1 arg)               | Dyadic (2 args)
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

In a dyadic application with a multi-element train, all operators default to their context-appropriate arity. Suffix an operator with `:` to force it monadic -- it will only receive the right argument.

```
(ρ!: (3 2) 6)          ; !: forces ! to be monadic (range)
                        ; so !: takes only 6 => 0 1 2 3 4 5
                        ; then ρ reshapes dyadically with (3 2)
                        ; result: 3 cols, 2 rows
```

Without `:`, the `!` in `(ρ! (3 2) 6)` would be dyadic (modulo), receiving both `(3 2)` and `6`.

### Combinators

Symbol | Alias   | Name  | Description
---    | ---     | ---   | ---
`/`    | `fold`  | Fold  | Reduce a list with a dyadic function
`\`    | `scanl` | ScanL | Each-left / outer product
`ǁ`    | `each`  | Each  | *not yet implemented*

### Special Forms

Symbol | Alias  | Name     | Description
---    | ---    | ---      | ---
`λ`    | `lam`  | Lambda   | Define a function: `(λ (params...) body...)`
`↻`    | `loop` | Do-block | Sequence expressions: `(↻ expr1 expr2 ... exprN)`
`:`    | `mon`  | Assign   | Bind a name: `(: name expr)`

### REPL Aliases

In the REPL, type the ASCII alias instead of the unicode symbol. It is replaced when you press Enter.

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
`mon`   | `:`    | Monadic Override / Assign
`fold`  | `/`    | Fold
`scanl` | `\`    | ScanL
`each`  | `ǁ`    | Each
`lam`   | `λ`    | Lambda
`loop`  | `↻`    | Do-block

## Operators in Detail

### `-` Negate / Subtract

```
(- 2)                   ; => -2
(- 1 2)                 ; => -1
```

### `!` Range / Modulo

```
(! 4)                   ; => 0 1 2 3
(! 3 10)                ; => 10 mod 3 = 1
```

### `=` Boolean Flip

```
(= 0)                   ; => 1
(= 5)                   ; => 0
(= (0 1 0 1))           ; => 1 0 1 0
```

### `_` Floor / Min

```
(_ 3.7)                 ; => 3 (floor)
(_ 2 5)                 ; => 2 (min)
```

### `&` Filter

Used dyadically with a boolean mask on the left and data on the right:

```
(& (1 0 1 0) (10 20 30 40))  ; => 10 30
```

### `ρ` Shape / Reshape

Monadic `ρ` creates a zeroed array from a shape description:

```
(ρ 5)                   ; => 0 0 0 0 0
(ρ (3 2))               ; => 3 cols, 2 rows:
                        ;    0 0 0
                        ;    0 0 0
```

Dyadic `ρ` reshapes data into the given shape. Left is shape, right is data:

```
(ρ (3 2) (0 1 2 3 4 5)) ; => 0 1 2
                         ;    3 4 5

(ρ (3 3) (0 1 2 3))     ; data cycles to fill:
                         ;    0 1 2
                         ;    3 0 1
                         ;    2 3 0

(ρ (5 5) 0)             ; scalar fills entire array:
                         ;    0 0 0 0 0
                         ;    0 0 0 0 0
                         ;    0 0 0 0 0
                         ;    0 0 0 0 0
                         ;    0 0 0 0 0
```

Compose `ρ` with `!` using the monadic override `:` to reshape a range:

```
(ρ!: (3 2) 6)           ; !: forces range (monadic) on 6
                         ; then ρ reshapes dyadically with (3 2):
                         ;    0 1 2
                         ;    3 4 5

(ρ!: (5 5) 25)          ; 5x5 matrix of 0..24:
                         ;     0  1  2  3  4
                         ;     5  6  7  8  9
                         ;    10 11 12 13 14
                         ;    15 16 17 18 19
                         ;    20 21 22 23 24
```

## Combinators in Detail

### Fold `/`

Reduces a list to a single value:

```
(+/ (! 10))             ; sum of 0..9 => 45
```

The function to the left of `/` is the reducer.

### ScanL `\`

Applies a dyadic function across elements:

```
(!\  (3 5) (! 10))      ; each of [3, 5] modulo'd against range(10)
```

## Trains

Trains are the core composition mechanism. A train is a sequence of operators and combinators written adjacently inside an application. They are applied right to left.

### Monadic Trains

```
(+/! 10)                ; range(10), then fold with +
```

Reading right to left: `!` produces `0..9`, then `+/` sums it.

### Dyadic Trains

```
(ρ!: (3 2) 6)           ; !: on rhs (range 6), then ρ dyadically with lhs
```

In a multi-element dyadic train:
- Rightmost operators apply **monadically** to the right argument
- The leftmost operator applies **dyadically** (combining lhs with the chain result)

### The `:` Monadic Override

In a dyadic train, operators would normally be dyadic. The `:` suffix forces an operator to be monadic:

```
(ρ!: (3 2) 6)           ; ! is forced monadic (range on 6)
                         ; ρ is dyadic (reshape (3 2) with the result)
```

## Lambda Functions

Define functions with `λ`. The first argument is a parameter list, the rest is the body:

```
(λ (x) (× x x))                    ; square function
(λ (a b) (+ a b))                   ; add two values
(λ (x) (: y (+ x 1)) (* y y))      ; multi-expression body
```

### Naming Functions

Use `:` to bind a lambda to a name:

```
(: square (λ (x) (× x x)))
(square 5)                          ; => 25
```

## Do-blocks

Use `↻` to sequence multiple expressions. The last expression's value is returned:

```
(↻
  (: x 5)
  (: y 10)
  (+ x y)
)
; => 15
```

The entire program should be wrapped in `(↻ ...)` when it contains multiple top-level expressions.

## Lists

List literals are parenthesized expressions where the first element is a value (not an operator):

```
(1 2 3)                 ; => 1 2 3
(10 20 30)              ; => 10 20 30
```

2D arrays are lists of lists:

```
((1 2 3) (4 5 6))       ; => 1 2 3
                         ;    4 5 6
```

For multi-dimensional arrays from flat data, use `ρ`:

```
(ρ (3 2) (1 2 3 4 5 6)) ; => 1 2 3
                         ;    4 5 6
```

## Assignment

Bind values to names with `:`:

```
(: x 42)
(: greeting "hello")
(: nums (1 2 3 4 5))
(: square (λ (x) (× x x)))
```

## Array Display

2D arrays are displayed as right-aligned grids:

```
>> (ρ (3 2) (1 2 3 10 20 30))
 1  2  3
10 20 30
```

1D lists are displayed space-separated:

```
>> (! 5)
0 1 2 3 4
```

## Comments

Use `;` for comments (Lisp-style):

```
; this is a comment
(+ 1 2)                ; inline comment
```

## Error Handling

Type mismatches and unsupported operations produce runtime errors instead of crashing:

```
>> (- "foo")
runtime error: - (negate) expects int or float, got string

>> (! "hello")
runtime error: ! (range) expects int, got string
```

## VM Architecture

Link compiles source code through a three-stage pipeline:

```
Source -> Parser (PEG) -> AST -> Bytecode Compiler -> VM -> Result
```

The VM is stack-based with 17 opcodes:

Opcode  | Code | Operand        | Description
---     | ---  | ---            | ---
`CONST` | `01` | `u16` index    | Push constant from variable store
`POP`   | `02` |                | Pop top of stack
`JMP`   | `03` | `u16` address  | Jump instruction pointer
`GETL`  | `04` |                | Get left variable (`w`)
`GETR`  | `05` |                | Get right variable (`a`)
`CRVAR` | `06` |                | Create variable
`CLVAR` | `07` |                | Clear variable
`DUP`   | `08` | `u16` address  | Duplicate top of stack
`MBL`   | `09` | `u16` address  | Start monadic block
`DBL`   | `0A` | `u16` address  | Start dyadic block
`END`   | `0B` |                | End block
`MO`    | `0C` | `u8` fn-id     | Monadic function
`DO`    | `0D` | `u8` fn-id     | Dyadic function
`CO`    | `0E` | `u8` cn-id     | Combinator
`CALL`  | `0F` | `u16` nargs    | Call user-defined function
`STORE` | `10` | `u16` name-idx | Store variable binding
`LOAD`  | `11` | `u16` name-idx | Load variable by name

The VM uses a value stack for computation and a context stack for tracking block nesting and return addresses.

## Working Examples

These are tested and verified:

```
; negate
(- 2)                           ; => -2

; arithmetic
(+ 2 2)                         ; => 4
(- 1 2)                         ; => -1

; range
(! 4)                           ; => 0 1 2 3

; fold (sum of range)
(+/! 10)                        ; => 45

; strings
"hello world"                   ; => "hello world"

; list literal
(1 2 3)                         ; => 1 2 3

; do-block (returns last expression)
(↻ (+ 1 2) (+ 3 4))            ; => 7

; reshape with monadic override
(ρ!: (3 2) 6)                   ; => 0 1 2
                                ;    3 4 5

; reshape with explicit data
(ρ (3 2) (0 1 2 3 4 5))        ; => 0 1 2
                                ;    3 4 5

; 2D list literal
((1 2 3) (4 5 6))               ; => 1 2 3
                                ;    4 5 6
```
