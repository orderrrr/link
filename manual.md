# Link ( *l is not k* )
Link is a programming language designed with heavy inspiration from [K]("https://k-project.fandom.com/wiki/Home") (mainly k6) and [BQN]("https://mlochbaum.github.io/BQN/").

### General structure of a link program:

A link program is built of the following concepts:
- A program - a list of expressions
- An expression - one of three things:
  1. A monadic expression (a monadic[^1] train and a single unit)
  2. A dyadic expression (a dyadic[^2] train and two units)
  3. A unit
- A unit - this is data (functions, lists, strings, numbers, characters, etc.)
- A train - this will be discussed later. Trains are multiple(or single) functions that chain together to create a result.
- A Function - functions come in many different forms. A function can either be user defined or be one of the primitive functions created in link (unlike k a user function in link can be used the same way a primitive function can).
- A list - the main data type all array oriented languages are designed to thrive on. Lists are not limited to one data type<br>
  (which comes with some benefits and some drawbacks). `[1;"hello link";|+/|]`
- A string - under the hood a string is the same datatype as a list.
- TODO - do more

Here is a simple example of a link program 
```
+/!|1000
```

###### What does this do?

Well this program:
takes all numbers from 0 -> 1000 and sums up the total.

###### How does it work?

To understand how it works we first need to understand the order of execution:<br>
First a program is formed, (list of expressions) in this case just one expression [`+/!|1000`]
Then expressions are formed, this is a monadic train so it needs a train and a unit:<br>
The unit in this case is a number: `1000`<br>
The train in this case is: [`|+, /, !|`]<br>
There is one charactar that is present in the program but not present here. That being the pipe `|`<br>
The pipe symbol in link is used to manupulate how link reads your program. For now just know that this is how we tell link it is a monadic train (we will go into this in greater detail later).

Now we have a program so let's execute it.
Programs are read from left to right like any language, however trains are read from right to left.<br>
So our program would do the following:
1. `!1000` - similar to range in pythons `range(1000)`
2. `+/` - `/` receives the range we generated and essentially does a [reduce/fold]("https://en.wikipedia.org/wiki/Fold_(higher-order_function)"))<br>
  the `+` is the function the fold will do on the input. so in this case

That's it! pretty simple function represented in a very readable way once you get an understanding of the symbols meaning.

## Program Symbols ( everything )

##### Functions

Symbol | Monadic              | Dyadic
---    |---                   |---
`+`    | Incriment (TODO)     | Add      (TODO)
`-`    | Negate (TODO)        | Subtract (TODO)
`×`    | Not currently in use | Multiply (TODO)
`÷`    | Not currently in use | Divide   (TODO)
`¯`    | Not currently in use | Max      (TODO)
`_`    | Not currently in use | Min      (TODO)
`=`    | Boolean flip (TODO)  | Equal    (TODO)
`&`    | Not currently in use | Indices  (TODO)
`!`    | Range (TODO)         | Modulo   (TODO)

##### HO Functions

Symbol | Monadic              | Dyadic
---    |---                   |---
`/`    | Not currently in use | Fold     (TODO)
`\`    | Not currently in use | ScanL    (TODO)
`ǁ`    | Each(TODO)           | Each     (TODO)

#### Modules

Symbol | Module
---    |---
`°`    | Higher order

##### Reserved Variables
Symbol | Use
---    |---
`ɑ`    | RIght side of dyadic/monadic train
`ω`    | Left side of dyadic train

##### User variables:
Any unicode character not reserved by the system

###### ASCII KEYS - `` !"#$%&'()*+,-./:;<=>?@\^_`|~ ``
###### NEEDS A USE - `` #$'*,:<>?@\`~.% ``

###### UNICODE KEYS (not currently in use) - `` µ¦¬¯°×÷øǁǂɑʘΦωʃΞ௦ ``

##### Lists

Symbol/s | Module
---      |---
`{}`     | Function block
`[]`     | List block
`()`     | Expression/train block
`""`     | String block

##### Other ( important )
One symbol that is used a lot in link and<br>
is a relatively unique concept is the pipe (`|`) symbol<br>
The pipe symbol does a number of things in different contexts.

Namely:
- Within the context of a monadic expression (eg: `+/!|100`)<br>
is an identifier for a monadic train
- Within the context of a dyadic expression (eg: `3 5|=!|5`)<br>
is an identifier for a dyadic train (used on both sides of the train)<br>

###### Why is this needed?

A number of reasons. Main one being that user functions and reserved functions in some array languages are very distinct from eachother. Creating a bit of a problem if you would like to use them interchangeably.<br>
Take these as examples:<br>
1. `f: 3 5; f|=!|5` - without the pipe: `f: 3 5; f=!5`<br>
2. `f: {=}; f!|1 0 1 0` - without the pipe: `f: {=}; f!1 0 1 0`<br><br>
Should f be used as a function of a monadic train or as the right argument of a dyadic train in no. 1<br>
Same goes for no. 2

- Within the context of a train (eg: `+/(^/=3 5|%!.\)|&!|10`). Ignore the program itself. Just have a look of the use of the pipe here. We can convert a segment of a monadic train to a dyadic one. and drop the right argument and continue the monadic train.


# VM
 OPCODE  | CODE | ARGS            | ROLE
---      |---   |---              |---
 `CONST` | `01` | Index(`u16`)    | Reads a variable
 `POP`   | `02` |                 | Pop the stack
 `JMP`   | `03` | BPointer(`u16`) | Jump 
 `GETL`  | `04` |                 | Get left variable from function
 `GETR`  | `05` |                 | Get right variable from function
 `CRVAR` | `06` |                 | Clear left variable from function
 `CLVAR` | `07` |                 | Clear right variable from function
 `DUP`   | `08` |                 | Duplicate top of stack
 `MBL`   | `09` | BPointer(`u16`) | Definition of monadic block
 `DBL`   | `10` | BPointer(`u16`) | Definition of dyadic block
 `END`   | `11` | BPointer(`u16`) | End of block
 `MO`    | `12` | Index(`u8`)     | Monadic function (system)
 `DO`    | `13` | Index(`u8`)     | Dyadic function (system)
 `CO`    | `14` | Index(`u8`)     | Combinator function (system)

# VM REVISED
There are two data arrays in a link vm.
1. A variable array where we push constants to.
2. An opcode array where we read and execute instructions from.

The vm has the following opcodes:

All instructions are stored in a 4 element array<br>
Each instruction has the following information:

NAME |TYPE |DESC
---  |---  |---
CODE |`u8` |the instruction code
DATA |`u8` |data row 1
DATA |`u8` |data row 2
OUTP |`u8` |output location

Currently supported opcodes are:
CODE   |DATA     |DATA     |OUTP     |DESC
---    |---      |---      |---      |---
`CNST` |_VAR IDX |_VAR IDX |USED     |Put constant[IDX] from variable store and put it into out
`IJMP` |BYTE IDX |BYTE IDX |NOT USED |Jump instruction pointer to specific location
`M_BL` |BYTE IDX |BYTE IDX |USED     |Start of monadic block
`D_BL` |BYTE IDX |BYTE IDX |USED     |Start of dyadic block
`M_FN` |M_OP IDX |C_OP IDX |USED     |Monadic function (system)
`D_FO` |D_OP IDX |C_OP IDX |USED     |Dyadic function (system)

OPCODE |CODE |ARGS            |ROLE
---    |---  |---             |---
`CONST`|`01` |Index(`u16`)    |Reads a variable

Example program bytecode:<br>
program `+/(^/=3 5|%!.\)|&!|1000`

CODE |DATA  |COMB  |OUTP
---  |---   |---   |---
_POP |_NIL  |_NIL  |_NIL
M_BL |0012  |_NIL  |OUTA
CNST |0000  |_NIL  |OUTA
M_OP |0001  |_NIL  |OUTA
D_BL |0008  |_NIL  |OUTA
M_BL |0005  |_NIL  |OUTW
D_BL |0002  |_NIL  |OUTA
CNST |0001  |_NIL  |OUTW
D_OP |0001  |0001  |OUTA
M_OP |0002  |_NIL  |OUTA
M_OP |0002  |_NIL  |OUTA
D_OP |0002  |_NIL  |OUTA
M_OP |0002  |0001  |OUTA

```
POP → pop value a from ↓
  MBL   → start m train ( results in output fed to a of print )
    CONST → grab v[0] ( puts const into a )
    MO !  → run ! on α
    DBL   → start d train
      MBL → start m train
        DBL   → start d train
          CONST → grav v[1]
          DO %  → run \% on ω, α
        MO =  → run eq on α
        MO ^  → run /^ on α
      DO & → run & on ω, α
    MO + → run +/ on α
```

Alternative:<br>
program:
```
fn: {+/(^/=w|%!.\)|&!|}
3 5 |fn| 1000
```


# VM Environment
Our VM is quite nice at this point, it has a decent way of storing state.
However we need our environment to be able to store more. Let's talk about functions.
I'm thinking of two ways we could store a function.
- Firstly, we could store it in the bytecode, to have one large bytecode array
- We could have a seperate state for the current runtime bytecode and any bytecode we would like to use later.
The latter seems to be a lot better to me for re-use.
Link is planned to be heavily built around a REPL type environment so it would be nice to have a succint way to store
previously executed functions/variables

CODE |DATA  |COMB  |OUTP
---  |---   |---   |---
_POP |_NIL  |_NIL  |_NIL
D_BL |TODO  |_NIL  |OUTA
CNST |0000  |_NIL  |OUTA
CNST |0001  |_NIL  |OUTB
E_FN |0001  |_NIL  |OUTA

and the function looks like this:

CODE |DATA  |COMB  |OUTP
---  |---   |---   |---
_POP |_NIL  |_NIL  |_NIL
M_BL |0012  |_NIL  |OUTA
M_OP |0001  |_NIL  |OUTA
D_BL |0008  |_NIL  |OUTA
M_BL |0005  |_NIL  |OUTW
D_BL |0002  |_NIL  |OUTA
SCPE |0001  |_NIL  |OUTA // scope variables
D_OP |0001  |0001  |OUTA
M_OP |0002  |_NIL  |OUTA
M_OP |0002  |_NIL  |OUTA
D_OP |0002  |_NIL  |OUTA
M_OP |0002  |0001  |OUTA

### On execution ByteCode
```
POP → pop value a from ↓
  DBL → start d train
    CONST → grab v[0] into α
    CONST → grab v[1] into ω
    E_FN → evaluate function[0] with current stack into α
```
### On execution FunctionDictionary
```
POP → pop value a from ↓
  MBL   → start m train ( results in output fed to a of print )
    MO !  → run ! on α
    DBL   → start d train
      MBL → start m train
        DBL   → start d train
          CONST → grab s[0][1]
          DO %  → run \% on ω, α
        MO =  → run eq on α
        MO ^  → run /^ on α
      DO & → run & on ω, α
    MO + → run +/ on α
```

---
###### No prior experience in language design or interpreters
Please keep in mind this is my first go of this. To add to that the implementation is a bit ropey at the moment and a lot of things don't work (or even fundamentally don't work).

[^1]: NOT WRITTEN
[^2]: NOT WRITTEN
