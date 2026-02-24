# BASIC Keywords

This document lists selected core built-in functions supported by this Basic build. For the full Basil reference, see https://yobasic.com/basil/reference.html

## Core Built-in Functions (recent additions)

- STR$(x)
  - Coerces a single argument to a string using standard formatting.
- VAL(s$)
  - Parses a trimmed string argument as a number. Tries integer first, then floating-point. If parsing fails, returns 0.0.
- SPLIT$(src$[, delim$])
  - Splits a string into an array of strings using `delim$` (default `,`). If `delim$` is empty, returns an array containing the entire string. If the delimiter is not found, returns a one-element array with the original string.
