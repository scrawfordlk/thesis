# Grammar

## Top-level definitions

```
language -> { function | enum }

function -> [ "unsafe" ] "fn" identifier
              "(" [ variable { "," variable } [ "," ] ] ")" [ "->" type ] block

enum     -> "enum" identifier "{" variant "," { variant "," } "}"

variant  -> identifier [ "(" type { "," type } ")" ]

block    -> "{" { ( binding | expression ) ";" } [ expression ] "}"
```

## Statement

```
binding   -> "let" variable "=" expression

variable  -> pattern ":" type

type       -> "u8"  | "usize" | "bool" | "char"
                | "&" "str"
                | identifier
                | ( "&" [ "mut" ] | "*" "mut" ) type
```

## Expression

```
expression -> [ "return" [ expression ] ] | assignment

assignment -> comparison [ "=" assignment ]

comparison -> arithmetic [ ( "==" | "!=" | "<" | ">" | "<=" | ">=" ) arithmetic ]

arithmetic -> term [ ( "+" | "-" ) arithmetic ] .

term       -> cast [ ( "*" | "/" | "%" ) term ] .

cast       -> factor { "as" type }

factor     -> { "-" | "*" | ( "&" [ "mut" ] ) }
                ( literal
                | identifier
                | call
                | "(" expression ")"
                | [ "unsafe" ] block
                | if
                | while
                | match )
```

## Remaining Control Flow

```
if      -> "if" expression block [ "else" [ if | block ] ]

while   -> "while" expression block

match   -> "match" expression "{" arms "}"

arms    -> pattern "=>" expression "," { expression "=>" expression "," }

pattern -> literal
             | [ "mut" ]  identifier
             | identifier "::" identifier [ "(" pattern { "," pattern } ")" ] )
             | "_"

call    -> identifier "(" [ expression { "," expression } [ "," ] ] ")"
```

## Literals

```
literal   -> integer | string | character | boolean | "()"

integer   -> digit { digit }

string    -> """ { printable_character } """

character -> "'" printable_character "'"

boolean   -> "true" | "false"

digit     -> "0" | ... | "9"

identifier -> ( letter | "_" ) { letter | digit | "_" }

letter     -> "a" | ... | "z" | "A" | ... | "Z"
```

## TODO

- Currently, this grammar does not have any rules for anything related to `::`, use in
  - enum instances (e.g. `Token::Let`)
  - boostrapped functions (e.g. `std::mem::size_of`)
