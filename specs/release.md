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
```bash
# Create and push tag
git tag v0.1.25
git push origin v0.1.25

# Create GitHub Release
gh release create v0.1.25 --title "v0.1.25" --notes-file CHANGELOG.md

# Publish to crates.io
cargo publish
```

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
