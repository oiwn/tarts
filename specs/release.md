# Release Routine

## Pre-Release Checklist

- [ ] All tests passing on main branch
- [ ] Pre-commit checks pass (`prek run --all`: fmt, clippy, tests, typos, gitleaks)
- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated with release notes
- [ ] `README.md` up to date
- [ ] `cargo build --release` succeeds

## Release Process

### Step 1: Merge to main

```bash
git checkout main
git pull origin main
git merge dev
git push origin main
```

### Step 2: Tag and Release

Tag-driven: the tag push does everything. `release.yml` builds the binaries and creates the GitHub release with assets; `publish.yml` verifies the tag matches `Cargo.toml` and publishes to crates.io.

```bash
# version in Cargo.toml must already equal the tag (publish.yml enforces it)
git tag v0.1.27
git push origin v0.1.27
```

Manual `gh release create` / `cargo publish` are no longer part of the flow.

### Step 3: Update Homebrew Tap

```bash
cd /path/to/homebrew-tap
just update-tarts 0.1.25

# Verify locally
brew install --build-from-source ./tarts.rb
brew test tarts

# Commit and push
git add tarts.rb
git commit -m "tarts: update to 0.1.25"
git push origin main
```

### Step 4: Verification

- [ ] GitHub Release created with assets
- [ ] `brew install oiwn/tap/tarts` works
- [ ] `cargo install tarts` works
- [ ] All effects run without errors

## Post-Release

- [ ] Update `specs/ctx.md` with next iteration task
- [ ] Announce on Reddit/social media
