%%

[0-9]+ "INTEGER"
[0-9]+\.[0-9]+ "FLOAT"
"(?:[^"\\]|\\.)*" "STRING"
true|false "BOOLEAN"

\*\* "POW"
== "EQUALITY"

\+ "ADD"
\- "SUB"
\* "MULT"
\/ "DIV"

let "DECLARATION"
: "TYPEDEC"
= "ASSIGNMENT"

\( "LPAREN"
\) "RPAREN"
\{ "LBRACE"
\} "RBRACE"
\< "LTRI"
\> "RTRI"

int|str|float|bool "TYPE"

[\t\n ]+ ;