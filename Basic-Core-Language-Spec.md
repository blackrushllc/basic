---
Title: Basic Core Language Specification (Basil v0 Core)
Status: Draft (Normative for Core)
Version: 1.0.0
Date: 2025-11-06
Note on Scope: >
  This document specifies only the source language accepted by the `basic` interpreter (Basil v0 core) and the interpreter-visible behavior: lexical grammar, parsing, evaluation, diagnostics, and observable I/O/exit status. It excludes host/runtime internals (bytecode format, Rust details, VM layout, object registry internals) and any non-observable embedding APIs.
---

1. Introduction and Conformance
1.1 Purpose and Non-goals (informative)
This document defines an implementation-neutral core language specification for the Basic language as implemented by the `basic` executable (Basil v0 core). It is sufficient for an independent implementation to accept and execute programs with the same observable behavior: program I/O, environment effects, and exit status.

Non-goals:
- Specify internal compilation or VM details.
- Specify non-core object libraries and optional features not observable in core.
- Define platform-specific terminal control side effects beyond textual I/O.

1.2 Document Structure (informative)
The spec is organized from source text/lexing to parsing, statements/expressions, runtime semantics, and appendices with grammars and precedence.

1.3 Conformance Classes (normative)
- Core-Conformant Interpreter: MUST accept and execute programs according to this specification, including all tokens, keywords, expressions, statements, control structures, and error/diagnostic behaviors marked as Required. It MAY implement additional libraries and objects provided they do not change core language syntax or semantics.
- Core-Conformant Compiler: MUST accept the same source and produce behavior-equivalent results as defined here when executed.

A conformant implementation MUST:
- Implement the lexical grammar and concrete syntax as specified.
- Implement evaluation rules, type/truthiness, operator precedence, and control-flow semantics.
- Produce diagnostics for syntax errors and for runtime errors defined by this spec.

1.4 Versioning (informative)
This document uses semantic versioning. Language‑breaking changes increment MAJOR, additive features increment MINOR, clarifications/fixes increment PATCH.

2. Design Overview (informative)
2.1 Philosophy
The language intentionally supports both a “classic BASIC” block form (`BEGIN`/`END`, `IF … THEN`) and a “modern” brace form (`{ … }`), freely mixable. It favors clarity, left‑to‑right evaluation, and friendly diagnostics.

2.2 Execution Model
Programs are parsed into statements. Newline acts as a statement terminator (like `;`) except in explicit continuation contexts, and `:` is a synonym for `;`. Statements execute sequentially unless control flow redirects execution. Functions (`FUNC`/`SUB`) provide local scopes. There is a global scope for top-level variables and labels.

2.3 Minimal Core Environment
- Standard output stream (text) for `PRINT`/`PRINTLN`.
- Standard error stream for diagnostics.
- Environment variable access for `SETENV`/`EXPORTENV` (Implementation‑Defined details below).
- Process execution via `SHELL` (Implementation‑Defined).
- Process exit via `EXIT`.

3. Source Text and Environment (normative)
3.1 Character Encoding
- Source files SHOULD be UTF‑8. An implementation MAY accept other encodings; behavior is Implementation‑Defined if not UTF‑8.
- Identifiers and keywords are ASCII‑based; string literals MAY contain arbitrary Unicode characters.

3.2 Lines, Newlines, and Statement Terminators
- Physical newlines (`\n`) separate lines. A carriage return (`\r`) is ignored if present.
- A logical statement terminator is any of:
  - A newline that is not in a continuation context (see 4.5).
  - A semicolon token `;`.
  - A colon token `:` (treated identically to `;`).
- An implementation MUST treat terminators as separating statements; redundant terminators are ignored.

3.3 Comments
The following line comments are recognized; each consumes from the introducer to the end of the physical line (excluding the terminating newline):
- Single quote: `' comment`.
- `// comment`.
- `# comment`.
- `REM comment` (case‑insensitive); `REM` MUST start at the current token position (not mid‑identifier).
Comments are ignored by the parser and never generate tokens.

3.4 Modules and Files
A program is a single source text. Include/import mechanisms are out of scope for the core spec. Object class loading via `CLASS("file")` is described as an expression in 5.

4. Lexical Grammar (normative)
4.1 Tokens and Punctuation
- Punctuation: `(` `)` `{` `}` `[` `]` `,` `;` `:` `+` `-` `*` `/` `.` `=` `==` `!=` `<>` `<` `<=` `>` `>=`
- Keywords (case‑insensitive): `FUNC` `FUNCTION` `SUB` `RETURN` `IF` `THEN` `ELSE` `WHILE` `DO` `BEGIN` `END` `ENDIF` `ENDFUNC` `ENDFUNCTION` `ENDSUB` `ENDWHILE` `ENDBLOCK` `WITH` `BREAK` `CONTINUE` `LET` `PRINT` `PRINTLN` `TRUE` `FALSE` `NULL` `AND` `OR` `NOT` `AUTHOR` `FOR` `TO` `STEP` `NEXT` `EACH` `IN` `FOREACH` `ENDFOR` `DIM` `AS` `DESCRIBE` `NEW` `CLASS` `TYPE` `SELECT` `CASE` `IS` `TRY` `CATCH` `FINALLY` `RAISE` `SETENV` `EXPORTENV` `SHELL` `EXIT` `STOP` `LABEL` `GOTO` `GOSUB` `MOD` `EXEC` `EVAL`.
- Identifiers: see 4.3.
- Literals: numbers (4.4.1), strings (4.4.2), booleans (`TRUE`/`FALSE`), `NULL`.

4.2 Keywords (reserved words)
Keywords are reserved regardless of case. However, after a dot in member access (e.g., `.Length`), the parser accepts most keywords as member names (implementation convenience). Implementations SHOULD allow keyword member names following `.`.

4.3 Identifiers (naming rules, case sensitivity)
- Start: ASCII letter `A–Z`/`a–z` or underscore `_`.
- Continue: ASCII letters/digits/underscore plus the suffix characters `$` `%` `@` `&`.
- Identifiers are case‑sensitive for user variables and labels in the core implementation; implementations MAY normalize case but MUST remain consistent.
- Conventional suffixes: `%` denotes integer variables/arrays, `$` denotes string variables/arrays, `@` denotes object variables/arrays. These suffixes influence certain semantics (see 6.3, 8).

4.4 Literals
4.4.1 Numeric
- Decimal integers (e.g., `42`), and decimals with a dot (e.g., `3.14`). Scientific notation is not part of the core lexical form.
- Numeric literals are parsed as floating‑point numbers. Integer contexts may coerce (see 5.3).

4.4.2 String
- Delimited by double quotes `"..."`.
- Escapes: `\"` `\n` `\t` `\r` `\}` (literal `}`), `\#{` (literal `#{`), and generic `\x` inserts `x`.
- Interpolation: within a string, the sequence `#{ expr }` evaluates `expr` and concatenates its string representation into the result. Nested braces and strings inside the interpolation are supported. The entire interpolated string is reduced into a concatenation expression at lexing time.
- Unterminated interpolation or empty `#{ }` MUST be diagnosed as a syntax error.

4.5 Whitespace and Line Continuation
- Whitespace (spaces/tabs) separates tokens.
- Newline acts as a statement terminator unless suppressed by continuation:
  1) Parentheses depth > 0.
  2) Previous token requires a continuation (binary operators `+ - * / . = == != <> < <= > >= AND OR`, comma, `TO`, `STEP`).
  3) The next nonspace character of the following line is a continuation operator among `+ - * . ,`.
- Explicit continuation: a standalone identifier `_` followed only by optional spaces and/or a line comment to the end of the line indicates continuation. The `_` is not a token; the line break is ignored.

5. Expressions (normative)
5.1 Types and Type System
- Dynamic types: Null, Bool, Number (float), Integer, String, Array (typed, fixed-size), Object, List (dynamic), Dict (dynamic map), Function, 2D fixed String arrays.
- Variable suffixes indicate preferred kind in some contexts (`%` → integer; `$` → string; `@` → object). Implementations SHOULD honor integer `%` variables in operations such as FOR counters (6.6).

5.2 Operators (set, precedence, associativity, short-circuiting)
- Postfix (highest): function call `f(args)`, index `a[i]`, member access `x.y` and `x.y(args)`.
- Prefix: unary minus `-e`, logical NOT `NOT e`.
- Multiplicative: `*` `/` `MOD`.
- Additive: `+` `-`.
- Comparisons: `=` (alias for `==`), `==`, `!=`, `<>` (alias for `!=`), `<` `<=` `>` `>=`.
- Logical: `AND` `OR`.

Associativity:
- Binary operators are left‑associative.
- Function calls, member access, and indexing associate left‑to‑right in a single chain.

Short-circuit:
- `AND` and `OR` MUST short‑circuit using the truthiness rules (5.3) and produce a Boolean `TRUE`/`FALSE` result.
- `NOT` applies to the truthiness of its operand and produces a Boolean.

5.3 Conversions and Truthiness
- Truthiness:
  - `NULL` → false.
  - `BOOL` → its value.
  - `NUM` → `n != 0.0`.
  - `INT` → `i != 0`.
  - `STRING` → non‑empty.
  - Arrays, Objects, Lists, Dicts → true unless empty (Lists/Dicts are truthy only if non‑empty; 2D string arrays truthy if rows×cols > 0).
- Numeric contexts MAY coerce floats to integers where required (e.g., `%` loop counters). Rounding mode is Implementation‑Defined; the reference implementation truncates toward zero in counter updates.
- Equality `=`/`==` and comparisons SHOULD compare numbers numerically and strings lexicographically; cross‑type comparisons are Implementation‑Defined and MAY raise a runtime error.

6. Statements and Blocks (normative)
6.1 Statement Separators (`\n`, `;`, `:`) and Single-line Forms
- Statements are terminated by newline, `;`, or `:` (interchangeable).
- Single-line forms are available for `IF … THEN <stmt> [ELSE <stmt>]`, `FOR` and `FOR EACH` bodies, `WITH`, and most constructs; multi-line blocks use `BEGIN`/`END` or `{}`.

6.2 Block Delimiters: `BEGIN`/`END` vs `{`/`}`
- Both forms are valid in all block‑accepting constructs listed below. `END` MAY be followed by an optional suffix (`IF`, `FUNC`/`FUNCTION`/`SUB`, `WHILE`, or identifier `BLOCK`) which is ignored.
- Mixing rules:
  - An `IF` then‑branch opened with `{` MUST close with `}`; a begin block opened with `BEGIN` MUST close with `END`. The `ELSE` branch MAY use a different block style than the `THEN` branch.
  - Standalone brace blocks `{ … }` and `BEGIN … END` blocks act as statements anywhere a statement is expected.

6.3 Variable Declarations and Assignment
- `DIM` declares arrays, fixed strings, and objects (8.2). Examples:
  - `DIM a(10)`; `DIM s$[20]` (fixed‑length string); `DIM s$ AS STRING * 20`.
  - `DIM p@ AS TypeName(args)`; `DIM arr@(5,10) [AS TypeName]`.
  - `DIM name = expr` initializes a scalar; if `name` ends with `%` or `$` and `expr` is a list literal, it desugars into an array declaration plus element assignments (1‑based indexes).
- `LET` assigns to variables/array elements and object properties:
  - `LET x = expr`.
  - `LET arr(i, j) = expr` (parentheses indexing for arrays).
  - `LET obj.Prop = expr`.
  - `LET list[key] = expr` (square‑bracket index for lists/dicts).
- Assignment without `LET`:
  - Allowed only for object property set `obj.Prop = expr` and square‑bracket index set `obj[expr] = expr`.
  - Otherwise, using `=` in an expression is equality test, not assignment; implementations MUST diagnose `Use LET for assignment` in invalid contexts.

6.4 Labels, `GOTO`, `GOSUB`, `RETURN` (constraints)
- Label declaration: `LABEL name` or colon form `name:` at statement start.
- `GOTO name` transfers control unconditionally.
- `GOSUB name` is a subroutine call; `RETURN` returns from the most recent `GOSUB`. `RETURN TO name` unwinds to the frame for the named label.
- Constraints: Jumping into the middle of multi‑statement constructs (e.g., into a `TRY`, `CATCH`, `FINALLY`, `WITH`, `FOR` body) is Implementation‑Defined; the reference implementation does not enforce static checks and behavior may be undefined at runtime.

6.5 `IF/THEN/ELSE`
Forms:
- Brace form:
  - `if expr { … } [else if expr { … }] [else { … }]`
- Classic forms:
  - `IF expr THEN BEGIN … [ELSE BEGIN … END] END`
  - `IF expr THEN <stmt> [ELSE <stmt>]` (single‑line branches); the `IF` MUST be closed by `END` if any branch is a single statement following `BEGIN` in the then‑branch.
Semantics: Evaluate `expr`; if truthy, execute then‑branch else else‑branch. `ELSE IF` chains are parsed as nested `IF` in the `ELSE` position.

6.6 Loops: `WHILE/DO`, `FOR/TO/STEP/NEXT`, `FOR EACH`
- `WHILE expr BEGIN … END` or `while expr { … }`.
  - Evaluate `expr` before each iteration; execute body while truthy.
- `FOR var = start TO end [STEP step] <body> NEXT [var]`.
  - `var` is assigned `start`. After each iteration, increment by `step` if present, else by `1` (integer for `%` counters). Loop continues while `(step >= 0 ? var <= end : var >= end)`. Loop variable updates for `%` counters use integer math (Implementation‑Defined rounding; reference truncates).
  - Body forms: `BEGIN…END`, `{…}`, or single statement. `NEXT` MAY optionally repeat the loop variable.
- `FOR EACH ident IN expr <body> NEXT [ident]`.
  - Iterates over items of a list/dict/object enumerable (semantics are Implementation‑Defined for non-list/dict types in core). Body forms as above.
- `BREAK` exits the innermost loop; `CONTINUE` skips to the next iteration. Only valid inside loops.

6.7 Selection: `SELECT CASE`
- Header: `SELECT CASE expr`.
- Body forms:
  - Classic: sequence of `CASE` arms ending with `END` or `END SELECT`.
  - Brace: `select case expr { … }` ending with `}`.
- Patterns in `CASE` (one or more, comma‑separated):
  - Value: `CASE value`.
  - Range: `CASE lo TO hi`.
  - Comparator: `CASE IS <op> expr` where `<op>` is one of `=` `==` `!=` `<` `<=` `>` `>=`.
- `CASE ELSE` provides a default arm; at most one is allowed.
- Matching: First matching arm executes; there is no fall‑through between arms.

6.8 Flow control: `BREAK`, `CONTINUE`
- As above; using them outside loops MUST be diagnosed as a syntax error.

6.9 Error handling: `TRY/CATCH/FINALLY`, `RAISE`
- `TRY` body followed by optional `CATCH [err$]` and/or `FINALLY` body; ends with `END TRY`.
- `CATCH` MAY introduce a string variable name ending with `$` to receive the error message.
- `RAISE [expr]` raises an error. Without an expression, it is valid only inside `CATCH` and re‑raises the current error.
- Control flow: `FINALLY` always runs after `TRY` or `CATCH`. Uncaught errors abort the program with a runtime error.

7. Functions and Scope (normative)
7.1 `FUNC`/`SUB` Definitions and `RETURN`
- Declaration: `FUNC name(param, …) <body>` or `SUB name(param, …) <body>`.
- Body forms:
  - `BEGIN … END [FUNC|FUNCTION|SUB]`.
  - `{ … }`.
  - Implicit: sequence of statements terminated by `END [FUNC|FUNCTION|SUB]`.
- `RETURN expr` returns from a `FUNC` with a value. In a `SUB`, `RETURN expr` is a compile/runtime error; `SUB` has no value. A bare `RETURN` in a function returns `NULL` (Implementation‑Defined if not explicit; the reference differentiates function vs gosub forms; using `RETURN` without expression outside `GOSUB` is treated as `GOSUB` return when not in a function).

7.2 Parameters (by value/ref), defaults
- Parameters are passed by value semantically. Default arguments are not part of the core.

7.3 Scope: lexical vs dynamic, shadowing, lifetime
- Functions introduce a local scope for parameters and locals. Top-level variables are global. Shadowing is allowed; name resolution prefers locals over globals.
- `WITH` introduces an implicit receiver for member access via leading `.`. Using a leading `.` outside `WITH` is a compile-time error.

8. Built-ins and Standard Library (normative for core)
8.1 Console I/O
- `PRINT expr[, expr, …]` prints the string forms of expressions separated by a single tab (`\t`).
- `PRINTLN expr[, expr, …]` behaves like `PRINT` and appends a newline (`\n`).
- Strings are produced by the language’s default formatting of values; Implementation‑Defined details for complex values.

8.2 Core data structures
- Arrays: fixed-size, 1‑based indexing with parentheses in `LET` and `DIM`. Dimensions are expressions evaluated at declaration time. Out‑of‑bounds access is a runtime error.
- Lists: `[ e1, e2, … ]` literal creates a dynamic list. Index with square brackets `list[i]` (0‑based or 1‑based is Implementation‑Defined; the reference uses 0‑based for list/dict square‑bracket indexing while classic arrays use 1‑based indices; mixing SHOULD be avoided). Assignment: `LET list[i] = v` or `list[i] = v`.
- Dicts: `{ "key": expr, … }` literal. Index with `dict["key"]`. Assignment as for lists.
- Objects/Classes: `NEW Type(args)` creates an object of `Type`. `CLASS("file")` loads a class from a file (Implementation‑Defined search rules). Member access: `obj.Prop`, calls: `obj.Method(args)`.

9. Runtime Semantics (normative)
9.1 Program start/termination, exit codes
- Execution begins at the top of the source file, executing statements in order. Function bodies execute only when called. The process exits when the end of the top-level is reached or an `EXIT` statement executes.
- `EXIT [expr]` terminates the program with exit status derived from `expr` (Implementation‑Defined conversion; reference uses integer if possible, default 0). `STOP` suspends execution (in the reference, it halts the VM; behavior is implementation detail for other hosts).

9.2 Determinism and side effects
- Expression evaluation is left‑to‑right. Side effects from function calls and assignments occur at their program order. Short‑circuiting must prevent evaluation of the right operand when determined by the left for `AND`/`OR`.

9.3 Error taxonomy and propagation
- Syntax errors: MUST stop compilation with a message containing the line number. Examples include unexpected tokens, unterminated strings, illegal `RAISE` without expr outside `CATCH`.
- Runtime errors: MUST abort execution unless caught by `TRY/CATCH`. Examples include out‑of‑bounds array access, calling a `SUB` where a value is required, undefined variable access, type errors in built‑ins.

10. Embedding and Host Interfacing (informative)
10.1 Minimal API expectations
An embedding host commonly exposes: initialize interpreter, evaluate code, set/get globals, capture stdout/stderr. This spec does not mandate an API.

10.2 Diagnostics capture
Implementations SHOULD provide line-aware error messages using the source line mapping. Recovery after a syntax error is not required.

10.3 Conformance considerations for embeddings
Hosts MUST NOT alter language semantics. Environment interactions via `SHELL`, `SETENV`, `EXPORTENV`, `CLASS` loading paths are Implementation‑Defined and SHOULD be documented.

11. Compliance and Tests (normative)
11.1 Required behaviors and prohibited behaviors
- Required: All syntax forms listed; newline/semicolon/colon termination; string interpolation; both block styles; short‑circuit logic; `LET` requirement for assignment; selection/loop semantics; error handling as specified.
- Prohibited: Treating `=` as assignment in expression contexts (except the permitted property/index sets), non‑short‑circuit evaluation for `AND`/`OR`.

11.2 Test suite structure and sample cases
A conformance suite SHOULD include cases for:
- Lexing: comments, explicit `_` continuation, newline insertion, tokens, string escapes and interpolation (including nested), numbers.
- Expressions: precedence and associativity table coverage; short‑circuit tests.
- Statements: each control form in both block styles and single‑line variants; mixing braces with classic blocks (e.g., THEN with `{}` and ELSE with `BEGIN … END`).
- Data structures: list/dict literals, indexing, `DIM` arrays, fixed strings.
- Functions: parameter passing, return, SUB vs FUNC in value context error.
- Flow: labels, `GOTO`, `GOSUB`/`RETURN` variants.
- Errors: `RAISE` rules; syntax diagnostics for unterminated constructs.

Appendix A. Complete Grammar (EBNF)
Note: Terminals are in quotes; keywords are case‑insensitive. `NL` stands for a statement terminator (newline/`;`/`:`). Commas inside lists/dicts allow optional trailing commas.

Program ::= { NL } { (Stmt { NL }) } EOF

Stmt ::= Block
       | IfStmt
       | WhileStmt
       | ForStmt
       | ForEachStmt
       | SelectCaseStmt
       | WithStmt
       | TryStmt
       | FuncDef
       | LabelDecl
       | GotoStmt | GosubStmt | ReturnStmt
       | DimStmt | LetStmt | AssignPropOrIndex
       | PrintStmt | PrintlnStmt | DescribeStmt | ExecStmt
       | SetEnvStmt | ShellStmt | ExitStmt | StopStmt
       | ExprStmt

Block ::= '{' { NL } { Stmt { NL } } '}'
        | 'BEGIN' { NL } { Stmt { NL } } 'END' [ ('IF' | 'FUNC' | 'FUNCTION' | 'SUB' | 'WHILE' | Ident 'BLOCK') ]

IfStmt ::= 'IF' Expr ( '{' { NL } { Stmt { NL } } '}'
                     | 'THEN' ( 'BEGIN' { NL } { Stmt { NL } } 'END'
                             | SingleStmt ) ) [ ElsePart ]
ElsePart ::= 'ELSE' ( '{' { NL } { Stmt { NL } } '}'
                    | 'BEGIN' { NL } { Stmt { NL } } 'END'
                    | SingleStmt
                    | IfStmt )
SingleStmt ::= Stmt  (restricted to non‑block constructs; implementation accepts any Stmt)

WhileStmt ::= 'WHILE' Expr ( '{' { NL } { Stmt { NL } } '}'
                           | 'BEGIN' { NL } { Stmt { NL } } 'END' )

ForStmt ::= 'FOR' Ident '=' Expr 'TO' Expr [ 'STEP' Expr ]
            ( '{' { NL } { Stmt { NL } } '}'
            | 'BEGIN' { NL } { Stmt { NL } } 'END'
            | SingleStmt )
            { NL } 'NEXT' [ Ident ]

ForEachStmt ::= 'FOR' 'EACH' Ident 'IN' Expr
                ( '{' { NL } { Stmt { NL } } '}'
                | 'BEGIN' { NL } { Stmt { NL } } 'END'
                | SingleStmt )
                { NL } 'NEXT' [ Ident ]

SelectCaseStmt ::= 'SELECT' 'CASE' Expr
                   ( { NL } { CaseArm } 'END' [ 'SELECT' ]
                   | '{' { NL } { CaseArm | CaseElse } '}' )
CaseArm ::= 'CASE' ( 'IS' CompareOp Expr | RangeOrValue { ',' RangeOrValue } ) { NL } { Stmt { NL } }
RangeOrValue ::= Expr [ 'TO' Expr ]
CaseElse ::= 'CASE' 'ELSE' { NL } { Stmt { NL } }
CompareOp ::= '=' | '==' | '!=' | '<>' | '<' | '<=' | '>' | '>='

WithStmt ::= 'WITH' Expr { NL } { Stmt { NL } } 'END' 'WITH'

TryStmt ::= 'TRY' { NL } { Stmt { NL } }
            [ 'CATCH' [ Ident ] { NL } { Stmt { NL } } ]
            [ 'FINALLY' { NL } { Stmt { NL } } ]
            'END' 'TRY'

FuncDef ::= ('FUNC' | 'FUNCTION' | 'SUB') Ident '(' [ ParamList ] ')' { NL }
            ( '{' { NL } { Stmt { NL } } '}'
            | 'BEGIN' { NL } { Stmt { NL } } 'END' [ ('FUNC'|'FUNCTION'|'SUB') ]
            | { Stmt { NL } } 'END' [ ('FUNC'|'FUNCTION'|'SUB') ] )
ParamList ::= Ident { ',' Ident }

LabelDecl ::= 'LABEL' Ident | (Ident ':')
GotoStmt ::= 'GOTO' Ident
GosubStmt ::= 'GOSUB' Ident
ReturnStmt ::= 'RETURN' ( 'TO' Ident | Expr | /* empty: returns from GOSUB */ )

DimStmt ::= 'DIM' Ident ('(' [ Expr { ',' Expr } ] ')'
                        [ 'AS' Ident ]
                      | 'AS' ( 'CLASS' '(' Expr ')'
                             | 'STRING' '*' Number
                             | 'TYPE' Ident
                             | Ident [ '(' [ Expr { ',' Expr } ] ')' ] )
                      | '[' Number ']'  /* fixed string for name$ */
                      | '=' Expr )

LetStmt ::= 'LET' ( Ident ( '(' [ Expr { ',' Expr } ] ')' [ '.' Ident ]
                          | '[' Expr ']' ) '=' Expr
                  | Ident '.' Ident '=' Expr )
AssignPropOrIndex ::= ( Primary ('.' Ident | '[' Expr ']') ) '=' Expr

PrintStmt ::= 'PRINT' Expr { ',' Expr }
PrintlnStmt ::= 'PRINTLN' Expr { ',' Expr }
DescribeStmt ::= 'DESCRIBE' Expr
ExecStmt ::= 'EXEC' '(' Expr ')'
SetEnvStmt ::= ('SETENV' | 'EXPORTENV') Ident '=' Expr
ShellStmt ::= 'SHELL' Expr
ExitStmt ::= 'EXIT' [ Expr ]
StopStmt ::= 'STOP'
ExprStmt ::= Expr

Expr ::= OrExpr
OrExpr ::= AndExpr { 'OR' AndExpr }
AndExpr ::= CmpExpr { 'AND' CmpExpr }
CmpExpr ::= AddExpr { ( '=' | '==' | '!=' | '<>' | '<' | '<=' | '>' | '>=' ) AddExpr }
AddExpr ::= MulExpr { ( '+' | '-' ) MulExpr }
MulExpr ::= PrefixExpr { ( '*' | '/' | 'MOD' ) PrefixExpr }
PrefixExpr ::= [ '-' | 'NOT' ] PostfixExpr
PostfixExpr ::= Primary { '(' [ ArgList ] ')' | '[' Expr ']' | '.' Ident [ '(' [ ArgList ] ')' ] }
ArgList ::= Expr { ',' Expr }

Primary ::= Number | String | 'TRUE' | 'FALSE' | 'NULL'
          | Ident
          | 'NEW' Ident '(' [ ArgList ] ')'
          | 'CLASS' '(' Expr ')'
          | 'EVAL' '(' Expr ')'
          | '[' [ Expr { ',' Expr } [ ',' ] ] ']'
          | '{' [ String ':' Expr { ',' String ':' Expr } [ ',' ] ] '}'
          | '(' Expr ')'
          | '.' Ident [ '(' [ ArgList ] ')' ]   /* WITH implicit receiver */

Appendix B. Operator Precedence Table
From highest to lowest (within same level, left‑associative):
- Postfix: call `()`, index `[]`, member access `.`
- Prefix: unary `-`, `NOT`
- Multiplicative: `*`, `/`, `MOD`
- Additive: `+`, `-`
- Comparisons: `=`, `==`, `!=`, `<>`, `<`, `<=`, `>`, `>=`
- Logical: `AND`, `OR`

Appendix C. Examples (idiomatic and edge cases)
C.1 Mixed block styles
```
IF x > 0 {
    PRINTLN "positive"
} ELSE BEGIN
    PRINTLN "non-positive"
END
```

C.2 Single-line IF
```
IF a = 0 THEN PRINTLN "zero" ELSE PRINTLN "nonzero"
```

C.3 Newline as semicolon and explicit continuation
```
PRINT 1,
_   // explicit continuation
2

x = 1 +
  2  // implicit continuation due to leading '+' on next line
```

C.4 String interpolation and escapes
```
PRINTLN "Hello, #{name}!\n2+2=#{2+2}"
PRINTLN "Literal \#{ not interpolation; and a closing brace: \}"
```

C.5 FOR and FOR EACH
```
FOR i% = 1 TO 5
    PRINT i%
NEXT

FOR EACH v IN [10,20,30] { PRINT v }
```

C.6 SELECT CASE forms
```
SELECT CASE n
CASE 1, 3, 5
    PRINTLN "odd small"
CASE 2 TO 10
    PRINTLN "even or small range"
CASE IS >= 100
    PRINTLN "big"
CASE ELSE
    PRINTLN "default"
END SELECT
```

C.7 WITH and implicit member access
```
WITH person
    .Name = "Ada"
    .Greet()
END WITH
```

C.8 TRY/CATCH/FINALLY and RAISE
```
TRY
    RAISE "boom"
CATCH err$
    PRINTLN err$
FINALLY
    PRINTLN "done"
END TRY
```

C.9 Labels and GOSUB/RETURN
```
GOSUB sub1
PRINTLN "back"
END

sub1:
PRINTLN "in sub"
RETURN
```

C.10 DIM and fixed strings
```
DIM name$ AS STRING * 10
DIM table%(3)  ' 3-int array (1-based)
DIM things@ AS ClassName(42)
DIM xs$[5]    ' fixed-length string variant
```

Change Log
- 1.0.0 (2025-11-06): Initial publication of the Basic Core Language Specification for Basil v0 core.
