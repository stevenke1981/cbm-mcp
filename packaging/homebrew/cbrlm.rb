class Cbrlm < Formula
  desc "Codebase RLM Memory MCP — Rust knowledge graph server for AI coding agents"
  homepage "https://github.com/stevenke1981/cbm-mcp"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/stevenke1981/cbm-mcp/releases/download/v#{version}/cbm-mcp-macos-arm64.tar.gz"
      sha256 "UPDATE_FROM_RELEASE_SHA256SUMS"
    end
    on_intel do
      url "https://github.com/stevenke1981/cbm-mcp/releases/download/v#{version}/cbm-mcp-macos-x64.tar.gz"
      sha256 "UPDATE_FROM_RELEASE_SHA256SUMS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/stevenke1981/cbm-mcp/releases/download/v#{version}/cbm-mcp-linux-arm64.tar.gz"
      sha256 "UPDATE_FROM_RELEASE_SHA256SUMS"
    end
    on_intel do
      url "https://github.com/stevenke1981/cbm-mcp/releases/download/v#{version}/cbm-mcp-linux-x64.tar.gz"
      sha256 "UPDATE_FROM_RELEASE_SHA256SUMS"
    end
  end

  def install
    bin.install "codebase-memory-mcp"
  end

  def post_install
    ohai "Run 'codebase-memory-mcp install --yes --all' to configure MCP agents"
  end

  livecheck do
    url :stable
    strategy :github_latest
  end

  def caveats
    <<~EOS
      Run `codebase-memory-mcp install --yes --all` to register the MCP server with coding agents.
      Optional graph UI: `codebase-memory-mcp --ui --port 9749`
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/codebase-memory-mcp --version")
  end
end