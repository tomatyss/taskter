class Taskter < Formula
  desc "Terminal Kanban board CLI tool"
  homepage "https://github.com/tomatyss/taskter"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/tomatyss/taskter/releases/download/v#{version}/taskter-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "<sha256>"
    end
    if Hardware::CPU.arm?
      url "https://github.com/tomatyss/taskter/releases/download/v#{version}/taskter-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "<sha256>"
    end
  end

  on_linux do
    url "https://github.com/tomatyss/taskter/releases/download/v#{version}/taskter-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "<sha256>"
  end

  def install
    bin.install "taskter"
  end
end
