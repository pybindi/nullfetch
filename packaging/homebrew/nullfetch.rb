class Nullfetch < Formula
  desc "Fast system fetch tool"
  homepage "https://github.com/pybindi/nullfetch"
  url "https://github.com/pybindi/nullfetch/archive/refs/tags/v3.0.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_SHA256"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    man1.install "nullfetch.1"
  end

  test do
    assert_match "nullfetch", shell_output("#{bin}/nullfetch --version")
  end
end
