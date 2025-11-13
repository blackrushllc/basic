## Junie Prompt — Add `EXEPATH$()` and `NET_DOWNLOAD_FILE%` Built-ins

You are working in a Rust-based BASIC interpreter project.
There are **two sibling projects** that are almost identical:

* **Basil** (full-featured): interpreter + compiler + web server.
* **Basic** (stripped-down “naked” BASIC): interpreter only.

This prompt is designed to work in either repo.
When I run it in one repo, treat *that* repo as the active project and ignore the other.

### Goal

Add **two new core built-in functions** to the BASIC language, always available (no feature flags):

1. `EXEPATH$()`
   Returns the **directory path of the currently running executable** as a string.

2. `NET_DOWNLOAD_FILE%(url$, destPath$)`
   Downloads a file from an HTTP(S) URL to a given destination path.
   Returns `0` on success, non-zero on error.
   The function should **not** throw a BASIC error on network failure; the caller checks the return code.

We already have a working `SHELL` command in the language; you may assume it exists but you **do not** need to modify it.

These functions will be used by a BASIC script called `upgrade.bas` (and other utilities) to download new versions of the `basic` / `basilc` / `bcc` / `basil-serve` binaries.

---

## Functional Requirements

### 1. `EXEPATH$()`

**Signature at the BASIC level:**

```basic
path$ = EXEPATH$()
```

Behavior:

* Returns the **absolute directory path** where the current interpreter executable lives.

    * On Linux/macOS: directory of the `basic` / `basilc` binary (`/usr/bin`, `/usr/local/bin`, etc.).
    * On Windows: directory of `basic.exe` / `basilc.exe` (e.g., `C:\Program Files\Basil\`).
* If the executable path cannot be determined for some reason, return an empty string `""`.
* Do **not** raise a BASIC error; failure -> `""`.

Implementation hint (Rust):
Use `std::env::current_exe()` and then take the parent directory as a string.

### 2. `NET_DOWNLOAD_FILE%(url$, destPath$)`

**Signature at the BASIC level:**

```basic
rc% = NET_DOWNLOAD_FILE%(url$, destPath$)
```

Parameters:

* `url$` – Full HTTP/HTTPS URL to download from.
* `destPath$` – Full path to the target file on disk.

Behavior:

* Perform a **blocking** HTTP(S) download.
* Create parent directories of `destPath$` if needed (like `mkdir -p`).
* Overwrite `destPath$` if it already exists.
* Return an integer status code:

    * `0` – success (file downloaded and written completely).
    * Non-zero – failure. Suggested mapping (document in code comments):

        * `1` – invalid or unsupported URL / parse error
        * `2` – HTTP error (non-2xx status)
        * `3` – network / TLS / IO error during transfer
        * `4` – file write / filesystem error
        * `99` – unexpected internal error

Constraints:

* **Must not** rely on external tools like `wget` or `curl` from BASIC; implement in Rust using an HTTP client library.
* Use a **blocking** API (no async runtime changes).
* Must work on **Linux, macOS, and Windows**.
* Timeouts should be sane defaults; no need for configuration for now.
* On partial failure, do **not** leave a corrupted partial file if you can reasonably avoid it (e.g., download to a temp file and then rename).

Note about this Basic repo build: to preserve offline/vendored builds, the initial implementation of NET_DOWNLOAD_FILE% uses a minimal in-process HTTP/1.1 client without external dependencies. It supports http:// URLs (including chunked transfer and Content-Length) and returns status code 3 for https:// URLs (TLS/network unsupported in this lean build). The full Basil distribution may enable HTTPS via optional features in the future.

---

## Implementation Plan

> Important: This project already has a structured architecture for tokens, parser, VM/runtime, and built-in functions. Please follow the existing patterns used for other built-ins (e.g., string functions, ENV$, etc.), not ad-hoc hooks.

### Step 1: Add a small HTTP client dependency

In the **interpreter core crate** where runtime built-ins live (likely something like `basilcore/common` or `basilcore/vm`, depending on how the project is structured), update `Cargo.toml` to add a minimal blocking HTTP client.

Use **reqwest** with a blocking + rustls TLS config (or whatever is consistent with the project’s existing dependency style):

```toml
[dependencies]
reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls"] }
```

If the project already depends on `reqwest` or has conventions around HTTP clients, follow those instead, avoiding duplicates.

Make sure any changes you make are compatible with Debian vendoring / offline builds (i.e., standard crates.io dependencies only, no git dependencies).

### Step 2: Wire `EXEPATH$()` into the language

Find the parts of the interpreter where built-in functions are:

* Declared or enumerated (e.g., some enum like `BuiltinFn`, or a function registry).
* Parsed into AST nodes or function call nodes.
* Executed at runtime in the VM.

Then:

1. Add a new built-in **function symbol** for `EXEPATH` (no arguments, returns string).

    * Follow the naming and typing conventions already used: some projects encode `$` in the name, others separate the type metadata. Use whatever Basil/Basic already do (look at how something like `ENV$` or other functions that return strings are represented).

2. Update the parser / resolver so that a call like `EXEPATH$()` is recognized and type-checked as returning a string.

3. Implement the runtime behavior:

    * In the VM (or standard library module where built-in functions are implemented), add a handler that:

        * Calls `std::env::current_exe()`.
        * If successful, takes `.parent()` and returns its string path (using OS-appropriate separator, but consistent with the rest of the project).
        * If anything fails, return an empty string `""`.

4. Add a simple unit/integration test:

    * Run a short program that prints `EXEPATH$()` and assert it’s non-empty or at least that the function does not crash.
    * Don’t hard-code exact paths, just assert that the function exists and returns a string.

### Step 3: Wire `NET_DOWNLOAD_FILE%` into the language

Similarly:

1. Add a new built-in function symbol for `NET_DOWNLOAD_FILE` that:

    * Takes **two string arguments**: `url$, destPath$`.
    * Returns an integer.

2. Update parser / resolver:

    * Ensure calls like `NET_DOWNLOAD_FILE%(url$, destPath$)` are accepted and typed correctly.

3. Implement a helper in Rust, e.g. in a dedicated module like `basilcore/common/net.rs`:

   ```rust
   pub fn net_download_file(url: &str, dest_path: &str) -> i32 {
       use std::fs;
       use std::io::Write;
       use std::path::Path;

       // Basic validation.
       let parsed = match reqwest::Url::parse(url) {
           Ok(u) => u,
           Err(_) => return 1, // invalid URL
       };

       let dest = Path::new(dest_path);

       if let Some(parent) = dest.parent() {
           if let Err(_) = fs::create_dir_all(parent) {
               return 4; // filesystem error
           }
       }

       let resp = match reqwest::blocking::get(parsed) {
           Ok(r) => r,
           Err(_) => return 3, // network-level error
       };

       if !resp.status().is_success() {
           return 2; // HTTP error
       }

       let bytes = match resp.bytes() {
           Ok(b) => b,
           Err(_) => return 3,
       };

       // Optional: write to temp file then rename to avoid partial corruption
       let tmp_path = dest.with_extension("download.tmp");

       if let Ok(mut f) = fs::File::create(&tmp_path) {
           if let Err(_) = f.write_all(&bytes) {
               let _ = fs::remove_file(&tmp_path);
               return 4; // write error
           }
       } else {
           return 4;
       }

       if let Err(_) = fs::rename(&tmp_path, dest) {
           let _ = fs::remove_file(&tmp_path);
           return 4;
       }

       0
   }
   ```

   (You can refine error mapping to match your style.)

4. Call this helper from the VM’s built-in dispatch:

    * Extract the two string arguments from the BASIC call stack / VM representation.
    * Call `net_download_file(url, destPath)` and push the returned `i32` back as a BASIC integer.

5. Add a basic test:

    * Do **not** rely on external internet in normal tests. Instead, you can:

        * Gate network tests behind a feature flag (`net-tests`) and document that they are optional.
        * Or at least add a doc-test/example showing manual usage:

          ```basic
          rc% = NET_DOWNLOAD_FILE%("https://example.com", "example.html")
          PRINT "RC ="; rc%
          ```

---

## Step 4: Keep behavior consistent between Basic and Basil

You will run this prompt in **each repo**:

* When you’re in the **Basil** repo, implement the functions in the Basil codebase.
* When you’re in the **Basic** repo, implement the same functions in its codebase.

Requirements:

* The BASIC-level behavior and API must be **identical** in both interpreters.
* Any tests or examples you add (e.g.,

  ```basic
  PRINT EXEPATH$()
  PRINT NET_DOWNLOAD_FILE%("https://example.com", "test.bin")
  ```

  ) should work identically in both projects.

---

## Step 5: Documentation

Add or update documentation:

* Wherever built-in functions are documented (reference manual, help text, README, etc.), add entries for:

    * `EXEPATH$()`

        * Returns the directory path of the running executable (empty string on failure).

    * `NET_DOWNLOAD_FILE%(url$, destPath$)`

        * Downloads a file from HTTP/HTTPS to `destPath$`, returns 0 on success, non-zero on error.
        * Note: This may involve network access and can take time.

Include at least one BASIC example snippet in the docs that matches what upgrade scripts will do.

---

## Acceptance Criteria

* The code compiles (`cargo build` / `cargo test`) in the current repo.

* A trivial BASIC program can run:

  ```basic
  PRINT "EXEPATH = "; EXEPATH$()
  rc% = NET_DOWNLOAD_FILE%("https://example.com", "test-download.bin")
  PRINT "RC ="; rc%
  ```

  without crashing (you can run this manually; tests do not have to hit the real network).

* The new built-ins are visible and callable in both:

    * The **Basic** interpreter project.
    * The **Basil** interpreter project.

Please now implement this plan in the current repo (Basic or Basil, depending on where I run this prompt), following existing architectural patterns in the interpreter.
