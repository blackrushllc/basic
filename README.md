# Basic

## This is the Basic Programming Language - A subset of Basil🌿
> ### This is what first year students should learn.
> ### This is what hobbyists should learn.
> ### This is what professionals should learn.
> ### This is the only programming language you need.

>
> Invite link to Blackrush Slack (Never Expires)
>
> https://join.slack.com/t/blackrushworkspace/shared_invite/zt-3g33s1rxc-9wWmCfggBEzInblqjzsn1A
>
> Join the Blackrush Slack Community for daily builds, discussions, lols
>

This BASIC interpreter and compiler is a subset of Basil🌿

Basil🌿 is a Modern, Mod-able, AI-aware, Object Oriented (or not) BASIC language Bytecode Interpreter and **Cross-Platform
Compiler** with lots of Rad Mods such as AI, AWS, Zip, Crypt (Base64, PGP) CrossTerm, Inet (SMTP, FTP, Json, Curl, REST, etc),
SQL(MySQL/Postgres, RDS, Sqlite, ORM, etc), MIDI (Audio, DAW), and even a Totally Tubular "OK Prompt" CLI mode
(Jolt Cola not included)

>
> Complete Online Reference for Basil: https://yobasic.com/basil/reference.html
>
> Look at the /docs/ folder for guides, development notes, and more.
>


## Why first languages matter

Your first programming language shouldn’t be a puzzle box. It should:
- Lower cognitive load while you’re learning core ideas like variables, expressions, control flow, and functions.
- Offer clear, immediate feedback (short edit–run cycles, gentle error messages).
- Be consistent in how it uses syntax to express ideas.
- Build habits that transfer to the broader programming world.

Basil🌿 was designed against these criteria. It keeps the classic readability of BASIC, but adds an alternate “modern” surface syntax so that what you learn today still looks familiar later.

At the same time, Basil🌿 is powerful enough to build real projects, with a growing standard library and a modular “mod” system that to adds out-of-box functionality like AI, AWS, SQL databases, HTTP, SMTP, JSON, CSV, cryptography, audio/MIDI/DAW support, and more.

Basil🌿 is also made for the AI age, the first programming language designed for AI from the ground up.

Basic is a subset of Basil🌿 that focuses on the core language features that beginners need, without overwhelming them with advanced concepts.

Basic is essentially the same as Basil except that it omits advanced features like:
- AWS integration
- Database access
- Networking (HTTP, SMTP, CURL)
- Object-oriented programming
- Modules and packages
- Advanced standard library functions (e.g., JSON, CSV, cryptography)
- AI integration
- Audio/MIDI/DAW support
- WebAssembly support
- Distributed processing (Gearman-like DPROC)
- Game-capable graphics
- Asterisk Integration (VoIP)
- Advanced Screen UI (CrossTerm)
- Tons of example programs using advanced features
- And more...
---

### Core built-ins for upgrades and utilities

This build includes two always-available built-in functions intended to help tooling like upgrade.bas and other utilities:

- EXEPATH$()
  - Returns the absolute directory path of the currently running executable, or an empty string on failure.
  - Example:
    
    PRINT "EXEPATH = "; EXEPATH$()

- NET_DOWNLOAD_FILE%(url$, destPath$)
  - Downloads a file from a URL to the given destination path. Returns 0 on success, non-zero on error.
  - Return codes:
    - 0 = success
    - 1 = invalid/unsupported URL
    - 2 = HTTP error (non-2xx)
    - 3 = network/TLS/IO error during transfer (in this Basic build, HTTPS URLs return 3)
    - 4 = file write/filesystem error
    - 99 = unexpected internal error
  - The function is blocking, creates parent directories as needed, writes to a temporary file and then renames to avoid partial files.
  - Example:

    rc% = NET_DOWNLOAD_FILE%("http://example.com/", "tmp/example.html")
    PRINT "RC = "; rc%

Note: In this lean Basic build, the downloader uses a minimal in-process HTTP/1.1 client without external dependencies. HTTPS is not currently supported here and will return code 3. The full Basil distribution may provide HTTPS via optional features.

### Two ways to say the same thing (both valid in Basic/Basil🌿)
Classic BASIC style:

```
REM BOTH SYNTAXES ARE VALID:

REM Infinite loop with BREAK (will break at 3)
LET i = 0;
WHILE TRUE BEGIN
    LET i = i + 1;
    IF i == 3 THEN BEGIN // Block IF
        BREAK;
    END
    PRINT i;
END
```

Modern brace style (THEN is implied when you open a brace):

```
// Infinite loop with BREAK (will break at 3)
let i = 0;
while true {
    let i = i + 1;
    if i == 3 { // Block IF
        break;
    }
    print i;
}
```

You can mix and match styles in one program. Internally, both forms compile to the same structures and run the same way.

---


### Quick Try:

🌿 Running a basic program without rebuilding the VM:

```terminal
target/release/basic run examples/hello.bas
# or
target/debug/basic run examples/hello.bas
```

Building and deploying Basic to run CGI scripts on Linux:

```
cargo build --release
install -m 0755 target/release/basic /usr/lib/cgi-bin/basic.cgi
```


🌿 https://basilbasic.com - The website for Basic/Basil🌿



# The Basic Programming Language for Education

### Why Basic/Basil🌿 works as a first learning language
- Gentle, explicit control flow
    - `if ... then` and `if ... { ... }` are both accepted; `else/elseif` read naturally.
    - `while`, `for`, and `select case` are straightforward and visible.
- Clear block boundaries
    - You can choose `BEGIN ... END` or `{ ... }`. Either way, blocks are explicit and obvious.
- Low ceremony, fast feedback
    - Small surface area, immediate execution, simple I/O (`print`, `println`).
- Case‑insensitive keywords; readable by design
    - Beginners don’t lose momentum over capitalization or minor formatting.
- A bridge to mainstream languages
    - The brace form prepares students to read/write C‑family languages without abandoning BASIC’s clarity.

---


### How Basic🌿 addresses first‑year pain points
- Visible structure
    - Choose braces or `BEGIN/END`. Students can literally “see the block.”
- Predictable, explicit control flow
    - `if/elseif/else`, `while`, `for/next`, and `select case` have minimal hidden rules.
- One concept at a time
    - You can start with the classic style and later migrate to braces without relearning the language.
- Transferable skills
    - The modern style maps cleanly to C, C#, Java, JavaScript, and Go idioms.
- Friendly diagnostics
    - Errors mention both classic and modern forms (e.g., “Expected THEN or ‘{’ after IF condition.”), guiding students instead of stopping them.

---

### A suggested path for an intro course (e.g., COP‑1000)
1. 🌱 Week 1–2: Variables, arithmetic, `print`/`println`, simple `if/then`.
2. 🌱 Week 3: Loops (`while`, `for/next`), `break` and `continue`.
3. 🌱 Week 4: Functions (`func`, `return`), parameters, local scope.
4. 🌱 Week 5: Decisions at scale: `select case`; string operations.
5. 🌱 Week 6: Modernization—introduce the brace style in parallel; show side‑by‑side translations.
6. 🌱 Week 7+: Objects and modules as applicable; project work.

Students leave with working mental models and syntax that looks familiar across the industry.

---

### Quick syntax map: classic to modern
- IF
    - Classic: `IF cond THEN BEGIN ... END`
    - Modern:  `if cond { ... }`
- ELSE / ELSEIF
    - Classic: `ELSE BEGIN ... END` or single statement
    - Modern:  `} else if cond { ... } else { ... }`
- WHILE
    - Classic: `WHILE cond BEGIN ... END`
    - Modern:  `while cond { ... }`
- FOR / NEXT
    - Classic: `FOR i = 1 TO 10 ... NEXT i`
    - Modern:  same control header; body can use `{ ... }`
- SELECT CASE
    - Classic: `SELECT CASE x ... END [SELECT]`
    - Modern:  `select case x { ... }`

Both forms are always valid; pick one or mix as you learn.

---

### Education and Community

Basic abd Basil🌿 are open source projects and are actively developed by a community of volunteers, built with education and community in mind.

We have built Basic and Basil🌿 to be a great learning tool for beginners, while remaining robust and powerful for real-world use.
We are committed to making it easy for you to learn the Basic language and to contribute to the project.

---

### Summary

Basil🌿 restores the simplicity many of us loved in our first encounters with BASIC, while offering a modern, brace‑style
path that aligns with today’s mainstream languages. It’s small enough to learn quickly, expressive enough to build real
projects, and friendly enough to keep students in the game—so more learners finish the course confident, not frustrated.

### Resources

Basic Github Repository: https://github.com/blackrushllc/basic

Basil Github Repository: https://github.com/blackrushllc/basil

Complete Online Reference: https://basilbasic.com/basil/reference.html

Email: BlackrushDrive@Gmail.com

Everywhere: @BlackrushWorld

Basic/Basil are open source projects under MIT license, Copyright (c) 2026 Blackrush LLC, Tarpon Springs, Florida, USA.