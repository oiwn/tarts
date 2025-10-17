#!/usr/bin/env ruby
# Homebrew Formula Template for Tarts (build-from-source)
# Place this file in your tap repository at: oiwn/homebrew-tarts/Formula/tarts.rb
# Then users can: `brew tap oiwn/tarts` and `brew install tarts`.

class Tarts < Formula
  desc "Terminal Arts - Screen savers and visual effects for terminal"
  homepage "https://github.com/oiwn/tarts"
  license "MIT"

  # Replace VERSION and SHA256 for each tagged release
  url "https://github.com/oiwn/tarts/archive/refs/tags/vVERSION.tar.gz"
  sha256 "REPLACE_WITH_SHA256"
  version "VERSION"

  depends_on "rust" => :build

  def install
    # Build and install from source; uses the lockfile and pins deps
    system "cargo", "install", "--locked", "--root", prefix.to_s, "--path", "."
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tarts --version")
  end
end

# How to update for a new release:
# 1) Set VERSION to the new tag (e.g., 0.1.22)
# 2) Fetch source tarball and compute SHA256:
#    curl -L -o tarts-v$VERSION.tar.gz \
#      https://github.com/oiwn/tarts/archive/refs/tags/v$VERSION.tar.gz
#    shasum -a 256 tarts-v$VERSION.tar.gz
# 3) Replace REPLACE_WITH_SHA256 with the hash.
# 4) Commit this formula to your tap repo (oiwn/homebrew-tarts) and push.
