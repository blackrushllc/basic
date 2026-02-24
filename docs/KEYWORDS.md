# BASIC Keywords

This document lists selected core built-in functions supported by this Basic build. For the full Basil reference, see https://yobasic.com/basil/reference.html

## Core Built-in Functions (recent additions)

- STR$(x)
  - Coerces a single argument to a string using standard formatting.
- VAL(s$)
  - Parses a trimmed string argument as a number. Tries integer first, then floating-point. If parsing fails, returns 0.0.
- SPLIT$(src$[, delim$])
  - Splits a string into an array of strings using `delim$` (default `,`). If `delim$` is empty, returns an array containing the entire string. If the delimiter is not found, returns a one-element array with the original string.

# Basil Core Keywords (for Editor Highlighting)

This list contains every keyword recognized by the core Basil interpreter ("basic" edition here)

Notes:
- All entries are uppercase as they commonly appear in docs; Basil is case-insensitive for keywords.
- Built-in functions are included (they are part of the core language surface without feature flags).
- Multi-word forms are listed as aliases for convenience.
- Preprocessor-like `#...` directives are excluded because the core lexer treats them as comments.

## Reserved Words and Statements (Core Lexer Keywords)
One keyword per line.

```
AND
AS
AUTHOR
BEGIN
BREAK
CATCH
CASE
CLASS
CONTINUE
DECLARE
DESCRIBE
DIM
DO
EACH
ELSE
ENDBLOCK
ENDFOR
ENDFUNC
ENDFUNCTION
ENDIF
ENDSUB
ENDWHILE
END
EVAL
EXEC
EXPORTENV
EXIT
FALSE
FINALLY
FOR
FOREACH
FUNC
FUNCTION
GOSUB
GOTO
IF
IN
IS
LABEL
LET
MOD
NEW
NEXT
NOT
NULL
OR
PRINT
PRINTLN
RAISE
RETURN
SELECT
SETENV
SHELL
STEP
STOP
SUB
THEN
TO
TRUE
TRY
TYPE
WHILE
WITH
```

## Core Built-in Functions (No feature flags)
One keyword per line.

```
ABS
APPENDFILE
ARRAY_COLS%
ARRAY_ROWS%
ASC%
AT
ATN
CHR$
COPY
COS
DELETE
DIR$
ENV$
ESCAPE$
EXP
FCLOSE
FEOF
FFLUSH
FOPEN
FREAD$
FREADLINE$
FSEEK
FTELL&
FWRITE
FWRITELN
GET$
HTML
HTML$
INKEY$
INKEY%
INPUT
INPUT$
INPUTC$
INSTR
INT
LCASE$
LEFT$
LEN
REMOVE$
REPLACE$
INSERT$
DATE$
TIME$
NOW$
EXPLODE
IMPLODE$
LOADENV%
LOG
MID$
MKDIRS%
MOVE
POST$
READFILE$
RENAME
REQUEST$
RIGHT$
RND
SIN
SLEEP
SPC
SQR
STR$
STRING$
SPLIT$
TAB
TAN
TRIM$
TYPE$
UCASE$
UNESCAPE$
URLDECODE$
URLENCODE$
USING$
VAL
WRITEFILE
```

## Comment Introducer (recognized by core lexer)

```
REM
```

## Common Multi‑word Forms (aliases)
These are commonly written as two-word forms in code; the core lexer also recognizes the single-word variants shown above.

```
END IF
END FUNC
END FUNCTION
END SUB
END WHILE
SELECT CASE
CASE ELSE
```

If you want this in another format (CSV/JSON/regex), let me know and I can add alternative exports.
