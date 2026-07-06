{
  description = "Development Nix flake for OpenAI Codex CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    claude-code = {
      # SECURITY: Pin to a specific rev for production use
      # url = "github:sadjow/claude-code-nix/<rev>";
      url = "github:sadjow/claude-code-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    custom-codex-release.url = "git+ssh://git@github.com/SPRAGE/custom-codex-release.git?ref=latest";
    dev-template = {
      url = "github:SPRAGE/dev-template";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, claude-code, custom-codex-release, dev-template, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust = pkgs.rust-bin.fromRustupToolchainFile ./codex-rs/rust-toolchain.toml;
        python = pkgs.python3.withPackages (ps: [ ps.pyyaml ]);
        bazel = pkgs.writeShellScriptBin "bazel" ''
          exec ${pkgs.bazelisk}/bin/bazelisk "$@"
        '';
        codexPackage = custom-codex-release.packages.${system}.codex;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            # Rust workspace tooling.
            rust
            pkgs.cargo-nextest

            # Repo build, format, and packaging helpers.
            bazel
            pkgs.dotslash
            pkgs.just
            pkgs.pkg-config
            pkgs.openssl
            pkgs.cmake
            pkgs.llvmPackages.clang
            pkgs.llvmPackages.libclang.lib
            pkgs.zip
            pkgs.unzip

            # Node, TypeScript SDK, and repo formatting.
            pkgs.nodejs
            pkgs.pnpm

            # Python SDK/scripts.
            python
            pkgs.uv

            # General repository exploration and AI workflow tools.
            pkgs.git
            pkgs.ripgrep
            pkgs.fd
            pkgs.jq
            pkgs.tree
            claude-code.packages.${system}.default
            codexPackage
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.bubblewrap
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

          shellHook = ''
            export CC=clang
            export CXX=clang++

            # Auto-sync shared skills from dev-template into .ai/skills and link provider views.
            _sync_agent_skills() {
              _src="$1"
              _dst="$2"
              _label="$3"
              if [ -d "$_src" ]; then
                if [ -e "$_dst" ] && [ ! -d "$_dst" ]; then
                  echo "skipped $_label sync because it is not a directory"
                  return
                fi
                mkdir -p "$_dst"
                _n=0
                for _d in "$_src"/*/; do
                  [ -d "$_d" ] || continue
                  _s=$(basename "$_d")
                  if [ ! -d "$_dst/$_s" ] || ! diff -rq "$_src/$_s" "$_dst/$_s" >/dev/null 2>&1; then
                    rm -rf "$_dst/$_s"
                    cp -rL "$_src/$_s" "$_dst/$_s"
                    chmod -R u+w "$_dst/$_s"
                    _n=$((_n + 1))
                  fi
                done
                [ "$_n" -gt 0 ] && echo "synced $_n skill(s) to $_label from dev-template"
              fi
            }

            _link_agent_skills() {
              _dst="$1"
              _label="$2"
              _shared="$PWD/.ai/skills"
              mkdir -p "$(dirname "$_dst")"
              if [ -L "$_dst" ]; then
                [ "$(readlink "$_dst")" = "../.ai/skills" ] || { rm -f "$_dst"; ln -s ../.ai/skills "$_dst"; echo "relinked $_label to .ai/skills"; }
              elif [ -d "$_dst" ]; then
                _can_convert=1
                for _d in "$_dst"/*/; do
                  [ -d "$_d" ] || continue
                  _s=$(basename "$_d")
                  if [ ! -d "$_shared/$_s" ]; then
                    cp -rL "$_d" "$_shared/$_s"
                    chmod -R u+w "$_shared/$_s"
                    echo "migrated $_label/$_s to .ai/skills"
                  elif ! diff -rq "$_d" "$_shared/$_s" >/dev/null 2>&1; then
                    echo "skipped $_label link because $_s differs from .ai/skills/$_s"
                    _can_convert=0
                  fi
                done
                if [ "$_can_convert" -eq 1 ]; then
                  rm -rf "$_dst"
                  ln -s ../.ai/skills "$_dst"
                  echo "linked $_label to .ai/skills"
                fi
              elif [ ! -e "$_dst" ]; then
                ln -s ../.ai/skills "$_dst"
                echo "linked $_label to .ai/skills"
              fi
            }

            _skills_src="${dev-template}/template/.ai/skills"
            _sync_agent_skills "$_skills_src" "$PWD/.ai/skills" ".ai/skills"
            _link_agent_skills "$PWD/.agents/skills" ".agents/skills"
            _link_agent_skills "$PWD/.claude/skills" ".claude/skills"
            _link_agent_skills "$PWD/.codex/skills" ".codex/skills"

            _agents_readme="${dev-template}/template/.agents/README.md"
            if [ -f "$_agents_readme" ] && [ ! -f "$PWD/.agents/README.md" ]; then
              mkdir -p "$PWD/.agents"
              cp -L "$_agents_readme" "$PWD/.agents/README.md"
              chmod u+w "$PWD/.agents/README.md"
              echo "synced .agents/README.md from dev-template"
            fi

            _codex_readme="${dev-template}/template/.codex/README.md"
            if [ -f "$_codex_readme" ] && [ ! -f "$PWD/.codex/README.md" ]; then
              mkdir -p "$PWD/.codex"
              cp -L "$_codex_readme" "$PWD/.codex/README.md"
              chmod u+w "$PWD/.codex/README.md"
              echo "synced .codex/README.md from dev-template"
            fi

            _codex_config="${dev-template}/template/.codex/config.toml"
            if [ -f "$_codex_config" ] && [ ! -f "$PWD/.codex/config.toml" ]; then
              mkdir -p "$PWD/.codex"
              cp -L "$_codex_config" "$PWD/.codex/config.toml"
              chmod u+w "$PWD/.codex/config.toml"
              echo "synced .codex/config.toml from dev-template"
            fi

            _codex_agents="${dev-template}/template/.codex/agents"
            if [ -d "$_codex_agents" ]; then
              mkdir -p "$PWD/.codex/agents"
              for _agent in "$_codex_agents"/*.toml; do
                [ -f "$_agent" ] || continue
                _agent_name=$(basename "$_agent")
                if [ ! -f "$PWD/.codex/agents/$_agent_name" ]; then
                  cp -L "$_agent" "$PWD/.codex/agents/$_agent_name"
                  chmod u+w "$PWD/.codex/agents/$_agent_name"
                  echo "synced .codex/agents/$_agent_name from dev-template"
                fi
              done
            fi

            # Fix hook permissions (nix flake init strips execute bit)
            if [ -d "$PWD/.claude/hooks" ]; then
              chmod +x "$PWD/.claude/hooks"/*.sh 2>/dev/null || true
            fi

            echo "OpenAI Codex CLI dev shell ready"
            echo "Rust: $(rustc --version)"
            echo "Node: $(node --version)"
            echo "pnpm: $(pnpm --version)"
          '';
        };
      }
    );
}
