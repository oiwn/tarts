# Tarts: Beautiful terminal screensavers in Rust - new release

---

# **Title:** Tarts: Beautiful terminal screensavers in Rust - v0.1.24

---

## **Post Body:**

Hey r/rust! I'd like to share my project **Tarts** - a collection of
terminal-based screensavers written in Rust. Just released v0.1.24 with some
exciting updates!

## 🎨 **What is Tarts?**

Tarts is a lightweight, fast collection of terminal screensavers that brings
visual effects to your terminal. Think of it as the Linux `cmatrix` but with a
dozen different effects and modern Rust implementation.

## ✨ **New in v0.1.24:**

- **Removed unmaintained dependencies** - Removed CLI parsing dep for even smaller binariy
- **New help system** - Run `tarts --help` to see all available effects with descriptions
- **Better CLI experience** - Added `--version` flag and improved error handling
- **Homebrew tap** - Easy installation on macOS

## 🎭 **Featured Effects:**

**Digital Rain** - Authentic Matrix-style digital rain with smooth animation and character flow

**Maze Generation** - Real-time maze generation with perfect algorithms

**3D Donut** - Classic 3D donut rotation with proper shading and perspective

**And 8 more effects:**
- Conway's Game of Life (it looks terrible, need to make it interesting)
- Boids flocking simulation (need to improve)
- 3D Cube rotation
- Fire simulation
- Plasma effects
- Pipe maze animation
- ASCII crabs

## 🚀 **Installation:**

### macOS (Recommended):
```bash
brew install oiwn/tap/tarts
```

### Anywhere via Cargo:
```bash
cargo install tarts
```

## 💻 **Usage:**

```bash
# Run any effect
tarts matrix
tarts maze
tarts donut

# See all effects
tarts --help
```

## 🔮 **What's Next:**

- Polish and optimize existing effects
- Add configuration system for customization
- More effect ideas and community contributions welcome!

**GitHub:** https://github.com/oiwn/tarts
