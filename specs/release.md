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
   - Builds binaries for macOS (x86_64, arm64), Linux (not implemented yet)
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

## Current Status
- Ready to release v0.1.24 with pico-args removal and CLI improvements
- Homebrew tap currently at v0.1.23
- Release v0.1.24 is live: https://github.com/oiwn/tarts/releases/tag/v0.1.24
- Need to update Homebrew tap to v0.1.24

## v0.1.24 Tap Update - Immediate Action Needed

### Current Formula Details
- File: https://raw.githubusercontent.com/oiwn/homebrew-tap/refs/heads/main/tarts.rb
- Current version: v0.1.23
- Current SHA256: `707c10aa58a41cc8ec3e995db640411b73c0b21343d3ec3200faec0a49d19d38`

### Updated Formula Requirements
- New version: v0.1.24
- New URL: `https://github.com/oiwn/tarts/archive/refs/tags/v0.1.24.tar.gz`
- New SHA256: `__NEED_TO_CALCULATE__`

### Steps to Calculate SHA256
```bash
curl -sL https://github.com/oiwn/tarts/archive/refs/tags/v0.1.24.tar.gz | sha256sum
```

### Why Build from Source?
- Bypass Apple's Gatekeeper warnings on macOS
- Ensure compatibility with user's system architecture  
- Allow optimization for user's specific CPU
- Reduce maintenance burden (no need to build binaries for multiple platforms)

### Updated Formula Preview
```ruby
class Tarts < Formula
  desc "Terminal Arts - Screen savers and visual effects for terminal"
  homepage "https://github.com/oiwn/tarts"
  license "MIT"

  url "https://github.com/oiwn/tarts/archive/refs/tags/v0.1.24.tar.gz"
  sha256 "__NEW_SHA256_HERE__"  # Replace with actual SHA256
  version "0.1.24"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tarts --version")
  end
end
```
