# Link Manual ( *l is not k* )

Link is a programming language designed with heavy inspiration from [K](https://k-project.fandom.com/wiki/Home) (mainly k6) and [BQN](https://mlochbaum.github.io/BQN/). It uses S-expression (Lisp-style) syntax while retaining array-oriented semantics, trains, and right-to-left evaluation.

## General Structure of a Link Program

A Link program is a single S-expression. The building blocks are:

- **S-expression** -- `(head elements...)` -- the universal syntax form
- **Atom** -- a bare value: integer, float, string, or name
- **Application** -- `(train args...)` -- apply operators to arguments. 1 arg = monadic, 2 args = dyadic.
- **Train** -- one or more operators/combinators chained together, evaluated right to left
- **Lambda** -- `(λ (params...) body...)` -- user-defined function
- **Do-block** -- `(↻ expr1 expr2 ... exprN)` -- sequence of expressions, returns last
- **Assignment** -- `(: name expr)` -- bind a value to a name
- **List literal** -- `(v1 v2 v3)` -- first element is a value, not an operator

### Disambiguation

The first element of any `(...)` determines what it is:

First element                     | Type
---                               | ---
Operator (`+`, `-`, `ρ`, etc.)    | Application
Combinator-attached op (`+/`)     | Application
Name (`foo`, `square`)            | Function call
`λ`                               | Lambda definition
`↻`                               | Do-block
`:`                               | Assignment
Number, float, string, nested `(` | List literal

### Example

```
(+/! 1000)
```

This program sums all numbers from 0 to 999:

1. `!` (range) produces `0 1 2 3 ... 999`
2. `+/` (fold with plus) sums the entire list

Result: `499500`

## Program Symbols

### Operators (Functions)

Symbol | Alias | Monadic (1 arg)              | Dyadic (2 args)
---    | ---   | ---                          | ---
`+`    | `add` | *not yet implemented*        | Add
`-`    | `neg` | Negate                       | Subtract
`×`    | `mul` | *not yet implemented*        | *not yet implemented*
`÷`    | `div` | *not yet implemented*        | *not yet implemented*
`¯`    | `max` | *not yet implemented*        | Maximum
`_`    | `min` | Floor (float to int)         | Minimum
`=`    | `eq`  | Boolean flip (0→1, n→0)      | *not yet implemented*
`&`    | `amp` | *not yet implemented*        | Filter by boolean mask
`!`    | `mod` | Range (0 to n-1)             | Modulo
`ρ`    | `rho` | Create zeroed array by shape | Reshape data to shape

### Combinators (Higher-Order Functions)

Symbol | Alias   | Description
---    | ---     | ---
`/`    | `fold`  | Fold/reduce a list with a dyadic function
`\`    | `scanl` | Each-left / outer product
`ǁ`    | `each`  | Each (*not yet implemented*)

### Special Forms

Symbol | Alias  | Syntax                          | Description
---    | ---    | ---                             | ---
`λ`    | `lam`  | `(λ (params...) body...)`       | Define a function
`↻`    | `loop` | `(↻ expr1 expr2 ... exprN)`     | Sequence expressions, return last
`:`    | `mon`  | `(: name expr)`                 | Assign a value to a name
`:`    |        | suffix on op (e.g. `!:`)        | Force monadic in dyadic train

### REPL Aliases

In the REPL, type the ASCII alias and it is replaced with the unicode symbol on Enter. Aliases can chain without spaces: `rhomodmon` → `ρ!:`.

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

## Syntax In Detail

### Atoms

Atoms are bare values that don't need parentheses:

```
42                      ; integer
-3                      ; negative integer
3.14                    ; float
"hello world"           ; string (use "" to escape quotes)
x                       ; variable name
```

### Application

Apply operators to arguments with parentheses. Arity is determined by argument count:

```
; Monadic (1 arg)
(- 5)                   ; negate: -5
(! 4)                   ; range: 0 1 2 3

; Dyadic (2 args)
(+ 3 4)                 ; add: 7
(- 10 3)                ; subtract: 7
(ρ (3 2) (1 2 3 4 5 6)) ; reshape: 2 rows × 3 cols
```

### Trains

A train is a sequence of operators and combinators written adjacently in the head position. They are evaluated right to left:

```
(+/! 10)                ; train is "+/!" applied to 10
                        ; ! produces 0..9, +/ sums → 45
```

In a **dyadic** multi-element train:
- **Rightmost** operators apply **monadically** to the right argument
- The **leftmost** operator applies **dyadically**, combining the left argument with the chain result

```
(ρ!: (3 2) 6)           ; !: monadically on 6 → 0 1 2 3 4 5
                         ; ρ dyadically: (3 2) reshape 0..5
                         ;    0 1 2
                         ;    3 4 5
```

### The `:` Monadic Override

In a dyadic train, suffix `:` on an operator to force it monadic:

```
(ρ!: (3 2) 6)           ; ! forced monadic (range), ρ stays dyadic
```

Without `:`, `!` in `(ρ! (3 2) 6)` would be dyadic (modulo).

### List Literals

When the first element of `(...)` is a value (not an operator), it's a list:

```
(1 2 3)                 ; list: [1, 2, 3]
(10 20 30)              ; list: [10, 20, 30]
((1 2 3) (4 5 6))       ; 2D list: two rows of three
```

For multi-dimensional arrays from flat data, use `ρ`:

```
(ρ (3 2) (1 2 3 4 5 6)) ; reshape flat data into 2×3 matrix
```

### Assignment

Bind values to names with `:`:

```
(: x 42)
(: greeting "hello")
(: nums (1 2 3 4 5))
(: square (λ (x) (× x x)))
```

Assigned names can be used in operator position (function calls) or as values:

```
(: data (! 10))
(+/ data)               ; sum the data
```

### Lambda Functions

Define functions with `λ`. First arg is the parameter list, rest is the body:

```
(λ (x) (+ x 1))                    ; increment
(λ (a b) (+ a b))                   ; add two values
(λ (x) (: y (+ x 1)) (× y y))      ; multi-expression body, returns last
```

Bind lambdas to names:

```
(: square (λ (x) (× x x)))
(square 5)                          ; => 25
```

Lambdas are callable in operator position and can be used in trains.

### Do-blocks

Use `↻` to sequence multiple expressions. Returns the last result:

```
(↻
  (: x 5)
  (: y 10)
  (+ x y)
)
; => 15
```

The entire program should be wrapped in `(↻ ...)` when it has multiple top-level expressions.

### Comments

Lisp-style `;` comments — from `;` to end of line:

```
; this is a full-line comment
(+ 1 2)                ; this is an inline comment
```

## Operators Reference

### `-` Negate / Subtract

```
(- 2)                   ; => -2
(- 5 3)                 ; => 2
```

### `!` Range / Modulo

```
(! 4)                   ; => 0 1 2 3
(! 3 10)                ; => 1 (10 mod 3)
```

### `=` Boolean Flip

```
(= 0)                   ; => 1
(= 5)                   ; => 0
(= (0 1 0 1))           ; => 1 0 1 0
```

### `_` Floor / Min

```
(_ 3.7)                 ; => 3
(_ 2 5)                 ; => 2
```

### `&` Filter

Dyadic: boolean mask on left, data on right:

```
(& (1 0 1 0) (10 20 30 40))    ; => 10 30
```

### `ρ` Shape / Reshape

Monadic — create zeroed array:

```
(ρ 5)                   ; => 0 0 0 0 0
(ρ (3 2))               ; => 0 0 0
                        ;    0 0 0
```

Dyadic — reshape data:

```
(ρ (3 2) (0 1 2 3 4 5)) ; => 0 1 2
                         ;    3 4 5
```

With monadic override for range:

```
(ρ!: (3 2) 6)           ; => 0 1 2
                         ;    3 4 5
```

### `+/` Fold

Reduce a list:

```
(+/ (! 10))             ; => 45 (sum of 0..9)
(+/ (1 2 3 4 5))        ; => 15
```

### `!\` ScanL

Each-left / outer product:

```
(!\ (3 5) (! 10))       ; each of [3, 5] modulo'd against range(10)
```

## Array Display

2D arrays are displayed as right-aligned grids:

```
>> (ρ (3 2) (1 2 3 10 20 30))
 1  2  3
10 20 30
```

1D lists are space-separated:

```
>> (! 5)
0 1 2 3 4
```

## Error Handling

Type mismatches produce runtime errors instead of crashing:

```
>> (- "foo")
runtime error: - (negate) expects int or float, got string

>> (! "hello")
runtime error: ! (range) expects int, got string
```

## VM Architecture

Link compiles source through a three-stage pipeline:

```
Source → Parser (PEG) → AST → Bytecode Compiler → VM → Result
```

### VM Opcodes

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

### VM Data Structures

The VM has two main data structures:

1. **Value stack** (`Vec<NN>`) — computation stack where all operations happen
2. **Context stack** (`[C; 512]`) — tracks block nesting (MBL/DBL/DUP/JMP) and return addresses

The variable store (`var: Vec<NN>`) holds constants and bound values. The lookup table (`lookup: HashMap<String, u16>`) maps variable names to indices in the variable store.

### Compilation Strategy

The bytecode compiler translates the AST into a flat byte stream:

- **Literals** → `CONST(idx)` — stored in the variable pool
- **Monadic apply** → push arg, `MBL`, train ops (reversed), `END`
- **Dyadic apply** → push both args, `DBL`, train ops (reversed, rightmost=MO, leftmost=DO), `END`
- **Do-block** → compile each expression sequentially, `POP` intermediate results
- **Assignment** → compile rhs, `STORE(name_idx)` — binds name to value
- **Lambda** → `JMP` past body, body code, `END`, then `CONST` of body address

### Block Lifecycle

Blocks (MBL/DBL) use back-patching: the compiler emits `MBL(0)` with a placeholder address, compiles the body, then patches the address to point past the `END` instruction.

In a DBL block, the VM duplicates both stack values so the dyadic operator has access to both arguments even after monadic operations in the train have consumed and replaced the top of stack.

## Not Yet Implemented

- **Lambda CALL** — lambdas compile but the `CALL` opcode is not yet wired up. User function calls through variable names in trains need a call stack mechanism.
- **Anonymous lambda calls** — `((λ (x) (+ x 1)) 5)` — structurally supported but blocked on CALL.
- **Each combinator** (`ǁ`) — parsed but not implemented in the VM.
- **Multiplication** (`×`) and **division** (`÷`) — operators exist but both monadic and dyadic forms return errors.

---
###### This is an experimental language. The implementation is evolving.
