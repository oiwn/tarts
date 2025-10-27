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
- [ ] Update README.md with current features and installation methods
- [ ] Add showcase GIFs/videos of effects for Reddit
- [ ] Document all available effects with examples
- [ ] Add installation instructions for Homebrew, Cargo, and manual

### 2. Code Quality
- [ ] Fix remaining Clippy warnings
- [ ] Run final tests to ensure all effects work correctly
- [ ] Verify binary builds work on target platforms

### 3. Release Assets
- [ ] Create release notes for v0.1.23
- [ ] Verify Homebrew formula installation works
- [ ] Test installation from tap: `brew install oiwn/tap/tarts`

### 4. Reddit Announcement Preparation
- [ ] Prepare engaging title and description
- [ ] Create visual assets (screenshots/GIFs)
- [ ] Write clear installation instructions
- [ ] Prepare to answer common questions

### 5. Final Checks
- [ ] Verify all effects start and run without errors
- [ ] Test on different terminal sizes
- [ ] Ensure graceful exit on Ctrl+C
- [ ] Check memory usage and performance

## Available Effects for Release
- matrix, life, maze, boids, cube, crab, donut, pipes, plasma, fire, terrain

## Installation Methods
1. **Homebrew**: `brew install oiwn/tap/tarts`
2. **Cargo**: `cargo install tarts`
3. **Manual**: Download binaries from GitHub releases

## Next Steps After Release
- Monitor Homebrew tap installations
- Gather user feedback from Reddit
- Plan next effect additions based on interest