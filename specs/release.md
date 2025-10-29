# Release Routine

## Goal
Automate and document the process for releasing new versions of tarts, including Homebrew tap updates.

## Pre-Release Checklist
- [ ] All tests passing on main branch
- [ ] Documentation updated (README.md, version numbers)
- [ ] Version bumped in Cargo.toml
- [ ] CHANGELOG.md updated with release notes
- [ ] GitHub workflow ready to build releases

## Release Process

### Step 1: Merge Release PR
```bash
# Ensure main branch is up to date
git checkout main
git pull origin main
```

### Step 2: Automated Release (GitHub Actions)
1. Push to main triggers `.github/workflows/release.yml`
2. GitHub Action:
   - Builds binaries for macOS (x86_64, arm64), Linux
   - Creates GitHub Release with changelog
   - Uploads release assets
   - Publishes to crates.io

### Step 3: Update Homebrew Tap (Manual)
```bash
# Clone homebrew-tap repository
git clone https://github.com/oiwn/homebrew-tap.git
cd homebrew-tap

# Update version and SHA256 in tarts.rb
# Using formula: sha256sum file
# Or: shasum -a 256 file

# Test formula locally
brew install --build-from-source ./tarts.rb
brew test tarts

# Commit and push
git add tarts.rb
git commit -m "tarts: update to <version_number>"
git push origin main
```

### Step 4: Verification
- [ ] GitHub Release created successfully
- [ ] Homebrew formula installs correctly
- [ ] cargo install tarts works
- [ ] All effects run without errors

## Homebrew Tap Update Details

### Automation Option (Future)
Could create GitHub Action that:
1. Watches for new releases on tarts repo
2. Auto-updates tarts.rb with new version/sha256
3. Runs tests and creates PR

### Manual Process (Current)
1. Download new binary from GitHub Release
2. Calculate SHA256: `shasum -a 256 tarts-macos-x86_64`
3. Update version in tarts.rb
4. Update URL and sha256 in tarts.rb
5. Test and commit

## Post-Release Tasks
- [ ] Update documentation if needed
- [ ] Monitor GitHub releases for issues
- [ ] Update version to next development version
- [ ] Announce on Reddit/social media

## Release Templates

### GitHub Release Template
```markdown
## Version X.Y.Z

### New Features
- Feature 1 description
- Feature 2 description

### Bug Fixes
- Bug fix description

### Installation
```bash
brew install oiwn/tap/tarts
cargo install tarts
```
```

### Homebrew Tap Update Script
```bash
#!/bin/bash
VERSION="0.1.24"  # get this from Cargo.toml
URL="https://github.com/oiwn/tarts/releases/download/v${VERSION}/tarts-macos-x86_64"
SHA256=$(curl -L -s "$URL" | shasum -a 256 | cut -d' ' -f1)

# Update tarts.rb with sed or similar
sed -i.bak "s/version \".*\"/version \"${VERSION}\"/" tarts.rb
sed -i.bak "s/sha256 \".*\"/sha256 \"${SHA256}\"/" tarts.rb
```
## Current Status
- Ready to release v0.1.24 with lag fixes
- Homebrew tap currently at v0.1.23
- Release workflow configured and tested
- Documentation updated
