# Current Task: Release Preparation

## Goal
Prepare the `tarts` project for public release and Reddit announcement. The project is now ready with multiple effects and Homebrew tap integration.

## Status
- ✅ Multiple effects implemented and working
- ✅ Homebrew tap created at: https://raw.githubusercontent.com/oiwn/homebrew-tap/refs/heads/main/tarts.rb
- ✅ Release workflow builds macOS binaries (x86_64, arm64)
- ✅ CI mostly working (only Clippy failing)

## Release Preparation Tasks

### 1. Documentation Updates
- [x] Update README.md with current features and installation methods
- [x] Add showcase GIFs/videos of effects for Reddit
- [x] Document all available effects with examples
- [x] Add installation instructions for Homebrew, Cargo, and manual

### 2. Code Quality
- [x] Fix remaining Clippy warnings
- [x] Run final tests to ensure all effects work correctly
- [x] Verify binary builds work on target platforms

### 3. Release Assets
- [ ] Create release notes for v0.1.23
- [x] Verify Homebrew formula installation works
^^^ already verified
- [ ] Test installation from tap: `brew install oiwn/tap/tarts`

### 4. Reddit Announcement Preparation
- [ ] Prepare engaging title and description
- [x] Create visual assets (screenshots/GIFs)
- [x] Write clear installation instructions

### 5. Final Checks
- [x] Verify all effects start and run without errors
- [x] Test on different terminal sizes
- [ ] Ensure graceful exit on Ctrl+C
^^^ effects are not quit immediately by "q" or ctrl-c sometimes there is few second lag! need to figure

## Available Effects for Release
- matrix, life, maze, boids, cube, crab, donut, pipes, plasma, fire, terrain

## Installation Methods
1. **Homebrew**: `brew install oiwn/tap/tarts`
2. **Cargo**: `cargo install tarts`
3. **Manual**: Download binaries from GitHub releases

# NOTES

README.md should be user friendly. Not too much nerd related

## Next Steps After Release
- Monitor Homebrew tap installations
^^^ possible?
- Gather user feedback from Reddit
- Plan next effect additions based on interest
