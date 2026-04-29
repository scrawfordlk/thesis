# Grammar of LLVM-IR

```
llvm -> { function | string }

function -> "define" type global "("
              [ type register { "," type register } ] ")"
              "{" { block } "}"

string -> global "=" "constant" array "c"" { printable_character } "\""

global -> "@" identifier
```

```
block -> label { instruction }

register -> "%" identifier

instruction -> label | return | branch | assignment

label -> identifier ":"

return -> "ret" ( "void" | type value )

branch -> "br" "label" register
        | "br" "i1" value "," "label" register "," "label" register

assignment -> register "="
                      ( binary
                      | icmp
                      | call
                      | gep )

binary -> ( "add" | "sub" | "mul" | "udiv" | "urem" ) type value "," value

icmp -> "icmp" "ult" type value "," value

call -> "call" type global "(" [ argument { "," argument } ] ")"

argument -> type value

gep -> "getelementptr" type "," "ptr" value "," type value { "," type value }

type -> integer | "void" | "ptr" | "[" number "x" type "]"

integer -> "i64" | "i32" | "i8" | "i1"

value -> register | integer | global

literal -> number | array

array -> "[" [ type literal { "," type literal } ] "]"

number -> [ "-" ] digit { digit }

identifier -> ( letter | "_" | "." ) { letter | digit | "_" | "." }
```
