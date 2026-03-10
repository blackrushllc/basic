# Basil vs. Basic Alignment Report

This report outlines the enhancements, bug fixes, and architectural improvements found in the Basil interpreter that are currently missing or outdated in the Basic project. This serves as a roadmap for bringing Basic into alignment with the core Basil interpreter.

## 1. Lexer Enhancements
Basil's lexer has been updated to support more modern string handling and identifier rules.
- **Triple-Quoted Strings:** Support for `"""long strings"""` that can span multiple lines and preserve formatting.
- **String Interpolation:** Implementation of `#{expression}` within strings for dynamic evaluation.
- **Expanded Identifiers:** Support for a broader range of characters in identifiers, aligning with modern naming conventions.

## 2. VM & Bytecode Improvements
Significant changes were made to the Virtual Machine to improve stability and error handling.
- **Stack Frame Safety:** Introduction of `gosub_base` and `handler_base` in the VM state. This prevents "RETURN without GOSUB" or "END PROC" errors from accidentally popping stack frames beyond the current function or subroutine boundary.
- **Source Mapping:** Enhanced bytecode metadata that maps instructions back to source lines/columns more accurately, resulting in better runtime error messages.
- **Opcode Optimization:** Basil has refined its opcode numbering and dispatch logic for better performance.

## 3. Parser & Syntax Updates
The Basil parser allows for more flexible and concise code.
- **Implicit LET:** Support for assignments without the `LET` keyword (e.g., `x = 10` instead of `LET x = 10`).
- **Flexible Block Syntax:**
    - Support for curly braces `{ ... }` as an alternative to `BEGIN...END`.
    - Support for implicit blocks in control structures.
- **DIM Desugaring:** More robust handling of array declarations, including multi-dimensional arrays and dynamic resizing logic.

## 4. Core Built-in Functions
Several key functions in Basil provide functionality that is currently missing or less capable in Basic.
- **`RENDER$(template$, context)`:** A powerful function for template rendering, used extensively in web contexts.
- **`SPLIT$(string$, delimiter$)`:** Returns a dynamic List object rather than a fixed-size array.
- **`VAL(string$)`:** Enhanced to handle integer conversions more gracefully and support different bases.
- **`STR$(value)`:** Refined implementation for better formatting of numbers and booleans.

## 5. Bytecode Format Differences
- **File Extension:** Basil uses `.basilx` while Basic uses `.basx`.
- **Header Metadata:** Basil's bytecode format includes versioning and feature flags that allow the VM to check for compatibility.

## Recommended Action Plan
1. **Port VM Safety Fixes:** Prioritize `gosub_base` and `handler_base` to fix stack corruption issues.
2. **Update Lexer:** Implement triple-quoted strings and interpolation.
3. **Enhance Parser:** Add support for implicit `LET` and brace-delimited blocks.
4. **Synchronize Built-ins:** Port the updated implementations of `SPLIT$`, `VAL`, and `STR$`.
