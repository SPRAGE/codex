{
  description = "Development Nix flake for OpenAI Codex CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    claude-code = {
      url = "github:sadjow/claude-code-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    dev-template = {
      url = "git+ssh://git@github.com/SPRAGE/dev-template.git";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, claude-code, dev-template, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems f;

      # Read the version from the workspace Cargo.toml (the single source of
      # truth used by the release workflow).
      cargoToml = builtins.fromTOML (builtins.readFile ./codex-rs/Cargo.toml);
      cargoVersion = cargoToml.workspace.package.version;

      # When building from a release commit the Cargo.toml already carries the
      # real version (e.g. "0.101.0").  On the main branch it is the placeholder
      # "0.0.0", so we fall back to a dev version derived from the flake source.
      version =
        if cargoVersion != "0.0.0"
        then cargoVersion
        else "0.0.0-dev+${self.shortRev or "dirty"}";
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          codex-rs = pkgs.callPackage ./codex-rs {
            inherit version;
            rustPlatform = pkgs.makeRustPlatform {
              cargo = pkgs.rust-bin.stable.latest.minimal;
              rustc = pkgs.rust-bin.stable.latest.minimal;
            };
          };
        in
        {
          codex-rs = codex-rs;
          default = codex-rs;
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              rust
              pkgs.git
              pkgs.ripgrep
              pkgs.fd
              pkgs.jq
              pkgs.tree
              pkgs.zip
              pkgs.unzip
              (pkgs.python3.withPackages (ps: [ ps.pyyaml ]))
              pkgs.nodejs
              pkgs.codex
              claude-code.packages.${system}.default
              pkgs.pkg-config
              pkgs.openssl
              pkgs.cmake
              pkgs.llvmPackages.clang
              pkgs.llvmPackages.libclang.lib
            ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              pkgs.bubblewrap
            ];
            PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            # Use clang for BoringSSL compilation (avoids GCC 15 warnings-as-errors)
            shellHook = ''
              export CC=clang
              export CXX=clang++

              # Auto-sync shared skills from dev-template into .ai/skills and
              # link provider views. Preserve project-specific skills by only
              # updating skills that exist in dev-template.
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
            '';
          };
        }
      );
    };
}
