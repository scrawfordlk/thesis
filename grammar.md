# Grammar

## Top-level definitions

```
language -> ( function | enum ) { function | enum }

function -> [ "unsafe" ] "fn" identifier
                "(" [ variable { "," variable } [ "," ] ] ")" [ "->" type ] block

enum     -> "enum" identifier "{" variant "," { variant "," } "}"

variant  -> identifier [ "(" type { "," type } ")" ]

block    -> "{" { statement } "}"
```

## Statement

```
statement -> binding | assign | return | expression ";"

binding   -> "let" [ "mut" ] variable "=" expression ";"

assign    -> [ "*" ] identifier "=" expression ";"

variable  -> identifier ":" type

return    -> "return" [ expression ] ";"

type       -> { "&" [ "mut" ] | "*" "mut" }
                  ( "u8"  | "usize" | "bool" | "char" | "&" "str" | identifier )

identifier -> ( letter | "_" ) { letter | digit | "_" }

letter     -> "a" | ... | "z" | "A" ... "Z"
```

## Expression

```
expression -> arithmetic [ ( "==" | "!=" | "<" | ">" | "<=" | ">=" ) arithmetic ]

arithmetic -> term { ( "+" | "-" ) term } .

term       -> cast { ( "*" | "/" | "%" ) cast } .

cast       -> factor { "as" type }

factor     -> [ "-" ] [ "*" ] [ "&" [ "mut" ] ] ( literal | identifier |
                  call | "(" expression ")" | [ "unsafe" ]  block | if | while | match )
```

## Remaining Control Flow

```
if    -> "if" expression block [ "else" [ if | block ] ]

while -> "while" expression block

match -> "match" expression "{" arms "}"

arms  -> expression "=>" expression "," { expression "=>" expression "," }

call  -> identifier "(" [ expression { "," expression } ] ")"
```

## Literals

```
literal   -> integer | string | character | boolean

integer   -> digit { digit }

string    -> """ { printable_character } """

character -> "'" printable_character "'"

boolean   -> "true" | "false"

digit     -> "" | ... | "9"
```

## TODO

- Currently, this grammar does not have any rules for anything related to `::`, use in
  - enum instances (e.g. `Token::Let`)
  - boostrapped functions (e.g. `std::mem::size_of`)
