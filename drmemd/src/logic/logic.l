%x VAR COLOR
%%

->                      "CONTROL"

"[^"]*"                 "STRING"

\(                      "("
\)                      ")"

not                     "B_NOT"
and                     "B_AND"
or                      "B_OR"

[Ii][Ff]                "KW_IF"
[Tt][Hh][Ee][Nn]        "KW_THEN"
[Ee][Ll][Ss][Ee]        "KW_ELSE"
[Ee][Nn][Dd]            "KW_END"

=                       "EQ"
\<>                     "NE"
\<=                     "LT_EQ"
>=                      "GT_EQ"
>                       "GT"
\<                      "LT"

-?[0-9]+\.[0-9]*([eE]-?[0-9]+)? "FLT"
-?[0-9]+                "INT"

\+                      "ADD"
-                       "SUB"
\*                      "MUL"
/                       "DIV"
%                       "REM"

true                    "TRUE"
false                   "FALSE"

\{                      <+VAR>"LBRACE"
<VAR>\}                 <-VAR>"RBRACE"
<VAR>[a-zA-Z][0-9a-zA-Z_]*    "IDENTIFIER"
<VAR>:                  "COLON"

\#			<+COLOR>;
<COLOR>[0-9a-zA-Z]+	<-COLOR>"COLOR"

[\t\n\r ]+ ;

.                       "UNKNOWN"
