Ahhh, that explains *everything*. 🙃

If `debuild` is making the `.orig` files disappear from your working tree, that means:

> **dpkg-source / quilt is treating `*.orig` as patch backup files and cleaning them up.**

So even if we ship them in the orig tarball, **Debian actively deletes them in the unpacked source tree**. That’s why Launchpad *still* can’t find `vendor/serde/Cargo.toml.orig` at build time.

So the conclusion is:

> We **cannot** rely on `Cargo.toml.orig` being present at build time.
> Instead, we must make the vendored checksums not care about them at all.

In other words: we need to make `.cargo-checksum.json` stop referencing those `Cargo.toml.orig` files, then safely delete those files everywhere.

Once Cargo no longer expects them, it won’t complain when Debian silently nukes them.

Let’s do that. This is simpler and more robust than fighting dpkg-source.

---

## Plan: Strip `Cargo.toml.orig` from the checksum metadata

High-level steps:

1. Regenerate `vendor/` normally.
2. Run a small Python script to remove `...Cargo.toml.orig` entries from all `.cargo-checksum.json` files.
3. Delete all `Cargo.toml.orig` files.
4. Confirm `cargo build --release --offline` still works.
5. Commit this cleaned vendor tree.
6. Bump to a new upstream version (say **1.0.3** or **1.0.4**), make a new orig tarball, verify it has **no** `.orig` files.
7. `debuild -S` + `dput` again.

I’ll assume we move to **1.0.3** now (we can skip reusing 1.0.2 to keep things clean).

---

## 1. Regenerate vendor cleanly (optional but recommended)

If you’ve already got a fresh vendor tree for 1.0.2 that builds offline, you *can* reuse it. But to be safe, let’s regenerate:

```bash
cd /var/www/basic  # repo root

rm -rf vendor
rm -f .cargo/config.toml

mkdir -p .cargo
cargo vendor vendor > .cargo/config.toml
```

Check that things look normal:

```bash
find vendor -name 'Cargo.toml.orig' | head
# you should see lots of ...Cargo.toml.orig again
```

---

## 2. Strip `Cargo.toml.orig` from `.cargo-checksum.json`

Now we surgically edit the checksum metadata so Cargo stops expecting those backup files.

Run this from the repo root:

```bash
python3 - << 'PY'
import json
from pathlib import Path

root = Path("vendor")

for path in root.rglob(".cargo-checksum.json"):
    data = json.loads(path.read_text())
    files = data.get("files", {})
    changed = False

    for key in list(files.keys()):
        if key.endswith("Cargo.toml.orig"):
            del files[key]
            changed = True

    if changed:
        data["files"] = files
        path.write_text(json.dumps(data, indent=4, sort_keys=True))
        print("Updated", path)
PY
```

This:

* Walks every `vendor/**/.cargo-checksum.json`.
* Removes any `files` entry whose key ends with `"Cargo.toml.orig"`.
* Writes the cleaned JSON back out.

---

## 3. Delete the actual `Cargo.toml.orig` files

Now that Cargo no longer expects them, we can safely remove them:

```bash
find vendor -name 'Cargo.toml.orig' -delete
```

Sanity check:

```bash
find vendor -name 'Cargo.toml.orig' | wc -l
# should be 0

grep -R "Cargo.toml.orig" vendor || echo "No Cargo.toml.orig references in vendor (good)"
```

---

## 4. Confirm offline build still works

This is the key test: if Cargo is happy, then the checksum metadata is self-consistent.

```bash
cargo clean
cargo build --release --offline
```

If this **succeeds**, then:

* Cargo accepted the modified `.cargo-checksum.json`.
* It isn’t trying to look for `Cargo.toml.orig` files anymore.
* We’re now immune to Debian silently nuking `*.orig`.

If it fails with a checksum error for some other file, we’d adjust the script, but in practice this works fine.

---

## 5. Commit the cleaned vendor tree

```bash
git add vendor .cargo/config.toml
git status
# should show modified .cargo-checksum.json files, no deleted vendor files

git commit -m "Clean vendored crates: drop Cargo.toml.orig and checksum references"
```

Make sure:

```bash
find vendor -name 'Cargo.toml.orig'   # no output
grep -R "Cargo.toml.orig" vendor      # no output
```

---

## 6. Bump to a new upstream version (say 1.0.3 or 1.0.4)

Let’s pick **1.0.3** for clarity.

Edit `Cargo.toml`:

```toml
[workspace.package]
edition = "2021"
version = "1.0.3"
```

Then:

```bash
git add Cargo.toml
git commit -m "Bump upstream version to 1.0.3"
git tag -a v1.0.3 -m "basic 1.0.3"
```

Update `debian/changelog`:

```bash
dch -v 1.0.3-0ubuntu1~noble1 -D noble "New upstream 1.0.3: vendor cleanup, remove Cargo.toml.orig and checksums."
git add debian/changelog
git commit -m "Debian: 1.0.3-0ubuntu1~noble1 for Noble"
```

Check:

```bash
git status  # should be clean
```

---

## 7. Create a fresh orig tarball and verify no `.orig` files

From the **parent** directory of `basic`:

```bash
cd /var/www   # parent of basic

git -C basic archive --format=tar --prefix="basic-1.0.3/" v1.0.3 | gzip -n > basic_1.0.3.orig.tar.gz
```

Now make absolutely sure nothing `.orig` snuck into the tarball:

```bash
tar tzf basic_1.0.3.orig.tar.gz | grep 'Cargo.toml.orig' || echo "No Cargo.toml.orig in orig (good)"
tar tzf basic_1.0.3.orig.tar.gz | grep '\.cargo/config.toml'
tar tzf basic_1.0.3.orig.tar.gz | grep 'vendor/' | head
```

You **want**:

* The “No Cargo.toml.orig in orig (good)” line.
* `vendor/` and `.cargo/config.toml` present.

---

## 8. Build the signed source package

Back in the repo:

```bash
cd basic

# Make sure there's no Cargo.lock before source build
rm -f Cargo.lock

debuild -S -sa -k2EF79540903769CEC035AF87D5E337C9282AE080
```

You should get:

* No “local changes detected” errors.
* A GPG passphrase prompt.
* Files in `../`:

  * `basic_1.0.3-0ubuntu1~noble1.dsc`
  * `basic_1.0.3.orig.tar.gz`
  * `basic_1.0.3-0ubuntu1~noble1.debian.tar.xz`
  * `basic_1.0.3-0ubuntu1~noble1_source.changes`

---

## 9. Upload to the PPA

From the parent directory:

```bash
cd ..
dput ppa:blackrush/basil basic_1.0.3-0ubuntu1~noble1_source.changes
```

Now on Launchpad:

* dpkg-source will still happily delete any `*.orig` it finds, **but there are none**.
* `cargo build --release --offline` will use your vendored crates and the edited `.cargo-checksum.json` files, which no longer mention `Cargo.toml.orig`.
* So the serde checksum error (and all its cousins) should finally be gone.

---

So yeah, your observation that `debuild` is deleting `.orig` files is spot-on — that’s dpkg/quilt behavior, not you going crazy.

The workaround is exactly what we just laid out: **make the Rust vendoring setup not rely on those backup files at all.** Once you’ve run through these steps and uploaded 1.0.3, let’s look at the new Launchpad build log and see what (if anything) it complains about next.
