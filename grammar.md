# Grammar

## Literals

```
digit -> 0 | ... | 9

letter -> "a" | ... | "z" | "A" ... "Z"

integer -> digit { digit }

character -> "'" printable_character "'"

string -> """ { printable_character } """

literal -> integer | string | character | "true" | "false"
```

## Expression

```
expression -> arithmetic [ ( "==" | "!=" | "<" | ">" | "<=" | ">=" ) arithmetic ]

arithmetic -> term { ( "+" | "-" ) term } .

term -> factor { ( "*" | "/" | "%" ) factor } .

factor -> [ "-" ] [ "*" ] [ "&" [ "mut" ] ]
            ( literal | identifier | call | "(" expression ")" | block | if | match )

block -> "{" expression "}"

if -> "if" expression block [ "else" [ if | block ] ]

match -> "match" expression "{" arms "}"

arms -> expression "=>" expression "," { expression "=>" expression "," }

call -> identifier "(" { expression "," } ")"

```

## Language constructs

```
identifier -> letter { letter | digit | "_" }

type -> "u8"  | "usize" | "bool" | "char" | "&str" | identifier |
          ( "&" [ "mut" ] | "*mut" ) type

variable -> identifier ":" type

binding -> "let" [ "mut" ] variable "=" expression ";"

statement -> binding | return | expression ";"

while -> "while" expression block

return -> "return" expression ";"
```

## Rust

```
language -> ( function | enum ) { function | enum }

function -> "fn" identifier "(" { variable "," } ")" [ "->" type ] block

enum -> "enum" identifier "{" variant "," { variant "," } "}"

variant -> identifier [ "(" type { "," type } ")" ]
```
