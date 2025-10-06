# Homebrew Formula Template for Tarts
# This template should be placed in a tap repository at: homebrew-taps/Formula/tarts.rb

class Tarts < Formula
  desc "Terminal Arts - Screen savers and visual effects for terminal"
  homepage "https://github.com/oiwn/tui-screen-savers-rs"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/oiwn/tui-screen-savers-rs/releases/download/v0.1.18/tarts-v0.1.18-aarch64-apple-darwin.tar.gz"
      sha256 ""  # TODO: Replace with actual SHA256 after release
    else
      url "https://github.com/oiwn/tui-screen-savers-rs/releases/download/v0.1.18/tarts-v0.1.18-x86_64-apple-darwin.tar.gz"
      sha256 ""  # TODO: Replace with actual SHA256 after release
    end
  end

  def install
    bin.install "tarts"
  end

  test do
    system "#{bin}/tarts", "--version"
  end
end