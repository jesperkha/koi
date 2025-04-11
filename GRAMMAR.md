# Grammar

```
program     -> pkg ...topLevel

pkg         -> "package" ident
topLevel    -> funcDecl | structDecl | aliasDecl

funcDecl    -> "pub"? "func" ident namedTuple namedTuple? block
nameTuple   -> "(" empty | ident type "," ... ")"
block       -> "{" ...statement "}"

structDecl  -> "struct" ident "{" ... ident type "}"
aliasDecl   -> "alias" ident type

type        -> baseType | arrayType | ident
baseType    -> "int" | "uint" | "float" | "bool" | "string"
arrayType   -> "[]" type

statement   -> empty | block | varDecl

varDecl     -> ident ":=" expr

expr        -> literal

literal     -> ident | string | number

ident       -> A-Za-z0-9_
empty       ->
```

