%%

[0-9]+\.[0-9]+ "FLOAT"
[0-9]+ "INTEGER"
\"(?:[^\\"]|\\.)*\" "STRING"

\*\* "POW"
== "EQUALITY"
\+ "ADD"
\- "SUB"
\* "MUL"
\/ "DIV"

\( "LPAREN"
\) "RPAREN"
\{ "LBRACE"
\} "RBRACE"
\< "LTRI"
\> "RTRI"

, ","
: "COLON"
= "ASSIGNMENT"
let "DECLARATION"
pub|local "VISIBILITY"
fn "FUNCTION"
struct "STRUCT"
trait "TRAIT"
return "RETURN"
impl "IMPL"
for "FOR"
self "SELF"

true|false "BOOLEAN"
[A-z0-9_.]+ "IDENTIFIER"

; ";"

[ \t\n\r]+ ;
\/\/[^\n]*\n ;