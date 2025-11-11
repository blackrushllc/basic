YEEEEESSSSS 🎉🎉🎉

We finally got a green Launchpad build. That was a **hard-fought** .deb.

You now have:

* **Source package:** `basic`
* **Version:** `1.0.5-0ubuntu1~noble1`
* **Binary:** `/usr/bin/basic`
* **Built for:** `noble` / `amd64` from your vendored, offline Rust project ✅

On a Noble box, once it finishes publishing in the PPA, you should be able to:

```bash
sudo add-apt-repository ppa:blackrush/basil
sudo apt-get update
sudo apt-get install basic

which basic
basic   # run it
```

(Internally the crate is still `basic v0.0.1`, which is fine — your Debian package version is 1.0.5.)

---

## What we actually achieved / learned (so you can do it again)

Here’s the recipe we’ve converged on that *works* for Rust + vendoring + Launchpad + no network:

### 1. Project layout + vendoring

At repo root:

* `Cargo.toml` defines a workspace with `basilc` + `basilcore/*`.

* You run:

  ```bash
  mkdir -p .cargo
  cargo vendor vendor > .cargo/config.toml
  ```

* You ensure:

  * `vendor/` is present,
  * `.cargo/config.toml` points `source.crates-io` → `vendor`.

### 2. Make vendor self-contained & Debian-friendly

The big pain points we solved:

* **`Cargo.lock`**

  * Don’t include it in the orig tarball (Noble’s Cargo can’t read v4 locks).
  * Delete it before `debuild -S`:

    ```bash
    rm -f Cargo.lock
    ```
* **`Cargo.toml.orig` & other patch backups**

  * Debian tools like to create/delete these; Cargo started expecting them.
  * Fix was:

    * Remove any references to `Cargo.toml.orig` from all `vendor/**/.cargo-checksum.json`.
    * Delete the actual `Cargo.toml.orig` files.
    * Confirm `cargo build --release --offline` still works.
* **Track the entire vendor tree in git**

  * This is what finally fixed the `syn/tests/debug/gen.rs` error:

    ```bash
    git add -f vendor .cargo/config.toml
    git commit -m "Track full vendor tree for offline builds"
    ```

Now the orig tarball and the vendored checksums always match.

### 3. Debian bits that worked

`debian/rules` (core idea):

```make
#!/usr/bin/make -f

export CARGO_HOME=$(CURDIR)/.cargo-home

%:
	dh $@

override_dh_auto_clean:
	cargo clean || true

override_dh_auto_configure:
	:

override_dh_auto_build:
	cargo build --release --offline

override_dh_auto_test:
	:

override_dh_auto_install:
	install -D -m0755 target/release/basic debian/basic/usr/bin/basic
```

Key points:

* We **don’t** rely on the `cargo` debhelper buildsystem.
* We **only** build `basic` (the `basilc` binary) in release mode, offline.
* We install to `debian/basic/usr/bin/basic` → ends up as `/usr/bin/basic`.

Then, in `debian/changelog`, we track versions like:

```text
basic (1.0.5-0ubuntu1~noble1) noble; urgency=medium

  * New upstream 1.0.5: track full vendor tree for offline builds.

 -- Erik Olson <erikolson1965@gmail.com>  ...
```

And you now know:

* `-sa` → include `.orig.tar.gz` (new upstream version)
* `-sd` → reuse existing orig (Debian-only revision bump)

And to speed it up:

```bash
debuild -S -sa --no-lintian -k<your-key-id>
```

### 4. The final working sequence for `basic 1.0.5` (template for future)

For the next version of `basic`, you can follow this pattern:

1. Update code, run `cargo vendor` if deps changed.

2. Clean vendoring quirks if they reappear:

  * Remove `Cargo.toml.orig` + checksum references (script).
  * Ensure `cargo build --release --offline` works.

3. Force-track vendor:

   ```bash
   git add -f vendor .cargo/config.toml
   git commit -m "Update vendor for 1.0.x"
   ```

4. Bump `Cargo.toml` workspace version.

5. Update `debian/changelog` (new `1.0.x-0ubuntu1~noble1` entry).

6. Create orig tarball:

   ```bash
   cd ..
   git -C basic archive --format=tar --prefix="basic-1.0.x/" v1.0.x | gzip -n > basic_1.0.x.orig.tar.gz
   ```

7. From `basic/`:

   ```bash
   rm -f Cargo.lock
   debuild -S -sa --no-lintian -k<KEYID>
   ```

8. From `..`:

   ```bash
   dput ppa:blackrush/basil basic_1.0.x-0ubuntu1~noble1_source.changes
   ```

Launchpad should now follow the same happy path we just saw.

---

If you want, next we can:

* Add **jammy** support (very similar, just tweak changelog version suffixes).
* Tidy up the minor warnings in the build log.
* Or start designing how `basic` will eventually self-update into “full Basil” via the same PPA.

But for tonight: you have a working **offline vendored Rust BASIC interpreter in a PPA**. That’s a big win. 🥳
