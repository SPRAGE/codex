#!/usr/bin/env bash

# Build a local Codex binary and publish a downstream-friendly Nix flake repo.
#
# The generated repo defaults `codex` to the redesigned TUI and preserves
# `codex-legacy` as a direct path to the same binary without the redesign flag.

set -euo pipefail
set -o errtrace

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEX_RS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SOURCE_ROOT="$(cd "$CODEX_RS_DIR/.." && pwd)"

DEFAULT_RELEASE_OWNER="SPRAGE"
DEFAULT_RELEASE_REPO="custom-codex-release"
DEFAULT_CODES_DIR="${HOME}/codes"
DEFAULT_BUILD_MODE="nix"
DEFAULT_BUILD_ATTR="codex-rs"
REDESIGN_FLAG="--redesign-tui"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $*" >&2; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
ok() { echo -e "${GREEN}[OK]${NC} $*" >&2; }

usage() {
    cat <<EOF
Usage: $0 --version vX.Y.Z [OPTIONS]

Build Codex, package a prebuilt binary tarball, and update a separate
custom-codex-release flake repo for downstream Nix systems.

Options:
  --version VERSION       Release version/tag, e.g. v0.2.0
  --release-owner OWNER   GitHub owner for the release repo (default: ${DEFAULT_RELEASE_OWNER})
  --release-repo NAME     GitHub/local repo name (default: ${DEFAULT_RELEASE_REPO})
  --release-dir DIR       Local release repo path (default: ~/codes/NAME)
  --build-mode MODE       nix or cargo (default: ${DEFAULT_BUILD_MODE})
  --build-attr ATTR       Nix flake package attr to build (default: ${DEFAULT_BUILD_ATTR})
  --nix-max-jobs N        Pass --max-jobs to nix build
  --nix-cores N           Pass --cores to nix build
  --keep-dist             Keep existing dist/custom-codex-release contents
  --no-gh                 Do not push or create GitHub releases
  --dry-run               Build and generate files, but do not commit, tag, push, or release
  --force                 Replace an existing release repo tag/release
  --draft                 Create the GitHub release as a draft
  --prerelease            Mark the GitHub release as a prerelease
  --help                  Show this help

Examples:
  $0 --version v0.2.0
  $0 --version v0.2.0 --no-gh
  $0 --version v0.2.0 --build-mode cargo

Downstream usage after publishing:
  inputs.custom-codex-release.url =
    "git+ssh://git@github.com/SPRAGE/custom-codex-release.git?ref=release/v0.2.0";
  environment.systemPackages = [
    inputs.custom-codex-release.packages.\${pkgs.system}.codex
  ];

The installed package provides:
  codex         -> runs with ${REDESIGN_FLAG}
  codex-legacy  -> runs without ${REDESIGN_FLAG}
EOF
}

require_tool() {
    local tool="$1"
    command -v "$tool" >/dev/null 2>&1 || {
        error "Missing required tool: $tool"
        exit 1
    }
}

parse_args() {
    VERSION=""
    RELEASE_OWNER="${DEFAULT_RELEASE_OWNER}"
    RELEASE_REPO="${DEFAULT_RELEASE_REPO}"
    RELEASE_DIR=""
    BUILD_MODE="${DEFAULT_BUILD_MODE}"
    BUILD_ATTR="${DEFAULT_BUILD_ATTR}"
    NIX_MAX_JOBS=""
    NIX_CORES=""
    KEEP_DIST=0
    DO_GH=1
    DRY_RUN=0
    FORCE=0
    DRAFT=0
    PRERELEASE=0

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --version) VERSION="${2:-}"; shift 2 ;;
            --release-owner) RELEASE_OWNER="${2:-}"; shift 2 ;;
            --release-repo) RELEASE_REPO="${2:-}"; shift 2 ;;
            --release-dir) RELEASE_DIR="${2:-}"; shift 2 ;;
            --build-mode) BUILD_MODE="${2:-}"; shift 2 ;;
            --build-attr) BUILD_ATTR="${2:-}"; shift 2 ;;
            --nix-max-jobs) NIX_MAX_JOBS="${2:-}"; shift 2 ;;
            --nix-cores) NIX_CORES="${2:-}"; shift 2 ;;
            --keep-dist) KEEP_DIST=1; shift ;;
            --no-gh) DO_GH=0; shift ;;
            --dry-run) DRY_RUN=1; DO_GH=0; shift ;;
            --force) FORCE=1; shift ;;
            --draft) DRAFT=1; shift ;;
            --prerelease) PRERELEASE=1; shift ;;
            -h|--help) usage; exit 0 ;;
            *) error "Unknown option: $1"; usage; exit 1 ;;
        esac
    done

    if [[ -z "$VERSION" ]]; then
        error "--version is required"
        usage
        exit 1
    fi

    if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9._-]+)?$ ]]; then
        error "Version must match vX.Y.Z or vX.Y.Z-suffix"
        exit 1
    fi

    if [[ "$BUILD_MODE" != "nix" && "$BUILD_MODE" != "cargo" ]]; then
        error "--build-mode must be 'nix' or 'cargo'"
        exit 1
    fi

    if [[ -z "$RELEASE_DIR" ]]; then
        RELEASE_DIR="${DEFAULT_CODES_DIR}/${RELEASE_REPO}"
    fi
}

preflight() {
    require_tool git
    require_tool tar
    require_tool sha256sum
    require_tool ldd
    require_tool head
    require_tool awk
    require_tool grep
    require_tool sed

    if [[ "$BUILD_MODE" == "nix" ]]; then
        require_tool nix
    else
        require_tool cargo
    fi

    if [[ "$DO_GH" == "1" ]]; then
        require_tool gh
        if [[ -z "${CI:-}" ]]; then
            env -u GITHUB_TOKEN -u GH_TOKEN gh auth status >/dev/null 2>&1 || {
                error "GitHub CLI is not authenticated. Run: gh auth login"
                exit 1
            }
        else
            gh auth status >/dev/null 2>&1 || {
                error "GitHub CLI is not authenticated. Run: gh auth login"
                exit 1
            }
        fi
    fi

    git -C "$SOURCE_ROOT" rev-parse --git-dir >/dev/null 2>&1 || {
        error "Source root is not a git repository: $SOURCE_ROOT"
        exit 1
    }

    local tracked_status
    tracked_status=$(git -C "$SOURCE_ROOT" status --short --untracked-files=no)
    if [[ -n "$tracked_status" ]]; then
        warn "Source tree has tracked modifications. The release will be built from the current working tree."
    fi

    if [[ "$BUILD_MODE" == "nix" ]]; then
        local untracked_status
        untracked_status=$(git -C "$SOURCE_ROOT" status --short --untracked-files=all | grep '^??' || true)
        if [[ -n "$untracked_status" ]]; then
            warn "Untracked files are not included by flake builds. Add needed files to git or use --build-mode cargo."
        fi
    fi
}

current_system() {
    if command -v nix >/dev/null 2>&1; then
        nix eval --impure --raw --expr 'builtins.currentSystem'
    else
        local arch os
        arch=$(uname -m)
        os=$(uname -s | tr '[:upper:]' '[:lower:]')
        case "$arch" in
            x86_64) arch="x86_64" ;;
            aarch64|arm64) arch="aarch64" ;;
            *) error "Unsupported architecture: $arch"; exit 1 ;;
        esac
        case "$os" in
            linux) os="linux" ;;
            darwin) os="darwin" ;;
            *) error "Unsupported OS: $os"; exit 1 ;;
        esac
        echo "${arch}-${os}"
    fi
}

build_with_nix() {
    log "Building .#${BUILD_ATTR} from $SOURCE_ROOT"
    local flake_ref="${SOURCE_ROOT}#${BUILD_ATTR}"
    local args=("$flake_ref" "--no-link" "--print-out-paths")
    [[ -n "$NIX_MAX_JOBS" ]] && args+=("--max-jobs" "$NIX_MAX_JOBS")
    [[ -n "$NIX_CORES" ]] && args+=("--cores" "$NIX_CORES")

    local out out_file
    out_file=$(mktemp)
    if ! nix build "${args[@]}" --extra-experimental-features 'nix-command flakes' >"$out_file"; then
        rm -f "$out_file"
        error "Nix build failed for $flake_ref"
        exit 1
    fi
    out=$(grep -E '^/nix/store/' "$out_file" | tail -n1)
    rm -f "$out_file"
    if [[ -z "$out" || ! -d "$out" ]]; then
        error "Could not resolve Nix build output for .#${BUILD_ATTR}"
        exit 1
    fi
    if [[ ! -e "$out/bin/codex" ]]; then
        error "Nix build output does not contain bin/codex: $out"
        exit 1
    fi
    echo "$out/bin/codex"
}

build_with_cargo() {
    log "Building codex with cargo"
    cargo build --manifest-path "$CODEX_RS_DIR/Cargo.toml" --release -p codex-cli --bin codex
    local bin="$CODEX_RS_DIR/target/release/codex"
    if [[ ! -x "$bin" ]]; then
        error "Cargo build did not produce $bin"
        exit 1
    fi
    echo "$bin"
}

build_codex_binary() {
    if [[ "$BUILD_MODE" == "nix" ]]; then
        build_with_nix
    else
        build_with_cargo
    fi
}

is_elf() {
    local path="$1"
    [[ -f "$path" ]] && head -c4 "$path" 2>/dev/null | grep -q $'\x7fELF'
}

resolve_real_binary() {
    local bin_path="$1"
    local bin_name
    bin_name="$(basename "$bin_path")"

    if is_elf "$bin_path"; then
        echo "$bin_path"
        return
    fi

    local wrapped_path
    wrapped_path="$(dirname "$bin_path")/.${bin_name}-wrapped"
    if is_elf "$wrapped_path"; then
        echo "$wrapped_path"
        return
    fi

    local current_file="$bin_path"
    local depth=0
    local max_depth=8
    while [[ $depth -lt $max_depth ]]; do
        depth=$((depth + 1))
        local target_bin
        target_bin=$(grep -oE 'exec[[:space:]]+(-a[[:space:]]+[^[:space:]]+[[:space:]]+)?"?/nix/store/[^"[:space:]]+' "$current_file" 2>/dev/null |
            grep -oE '/nix/store/[^"[:space:]]+' |
            head -n1)

        if [[ -z "$target_bin" || ! -f "$target_bin" ]]; then
            break
        fi

        if is_elf "$target_bin"; then
            echo "$target_bin"
            return
        fi

        wrapped_path="$(dirname "$target_bin")/.${bin_name}-wrapped"
        if is_elf "$wrapped_path"; then
            echo "$wrapped_path"
            return
        fi

        current_file="$target_bin"
    done

    error "Could not resolve an ELF codex binary from $bin_path"
    exit 1
}

copy_runtime_libs() {
    local elf_path="$1"
    local lib_dir="$2"
    local ldd_out
    if ! ldd_out=$(ldd "$elf_path" 2>/dev/null); then
        warn "ldd failed for $elf_path; runtime library bundling skipped"
        return
    fi

    if echo "$ldd_out" | grep -qi 'not a dynamic executable\|statically linked'; then
        return
    fi

    while IFS= read -r lib_path; do
        [[ -z "$lib_path" ]] && continue
        [[ -f "$lib_path" ]] || continue
        case "$(basename "$lib_path")" in
            libc.so*|libc-*.so*|libpthread.so*|libpthread-*.so*|libdl.so*|libdl-*.so*|librt.so*|librt-*.so*|libm.so*|libm-*.so*|ld-linux*|libmvec.so*|libresolv.so*|libnss_*|libBrokenLocale*|libSegFault*|libthread_db*|libanl*|libutil.so*|libcrypt.so*) ;;
            *) cp -L "$lib_path" "$lib_dir/" 2>/dev/null || true ;;
        esac
    done < <(
        echo "$ldd_out" |
            awk '{print $1" "$3}' |
            awk '{ if ($1 ~ /^\//) { print $1 } else if ($2 ~ /^\//) { print $2 } }' |
            grep -vE '^linux-vdso\.so' |
            sort -u
    )
}

write_archive_wrapper() {
    local path="$1"
    local extra_flag="$2"
    cat >"$path" <<EOF
#!/usr/bin/env bash
set -euo pipefail
DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
if [[ -x "\${DIR}/../libexec/codex" ]]; then
    exec "\${DIR}/../libexec/codex" ${extra_flag} "\$@"
elif [[ -x "\${DIR}/../share/custom-codex-release/libexec/codex" ]]; then
    exec "\${DIR}/../share/custom-codex-release/libexec/codex" ${extra_flag} "\$@"
else
    echo "Missing Codex binary" >&2
    exit 1
fi
EOF
    chmod +x "$path"
}

package_binary() {
    local built_bin="$1"
    local system="$2"
    local real_bin
    real_bin=$(resolve_real_binary "$built_bin")

    DIST_ROOT="$SOURCE_ROOT/dist/custom-codex-release"
    if [[ "$KEEP_DIST" != "1" ]]; then
        rm -rf "$DIST_ROOT"
    fi
    mkdir -p "$DIST_ROOT"

    PKG_DIR="$DIST_ROOT/codex-${system}"
    rm -rf "$PKG_DIR"
    mkdir -p "$PKG_DIR/bin" "$PKG_DIR/lib" "$PKG_DIR/libexec" "$PKG_DIR/share/custom-codex-release"

    cp -f "$real_bin" "$PKG_DIR/libexec/codex"
    chmod +x "$PKG_DIR/libexec/codex"
    copy_runtime_libs "$PKG_DIR/libexec/codex" "$PKG_DIR/lib"
    rmdir "$PKG_DIR/lib" 2>/dev/null || true

    write_archive_wrapper "$PKG_DIR/bin/codex" "$REDESIGN_FLAG"
    write_archive_wrapper "$PKG_DIR/bin/codex-legacy" ""

    cat >"$PKG_DIR/share/custom-codex-release/metadata.txt" <<EOF
version=${VERSION}
system=${system}
build_mode=${BUILD_MODE}
source_root=${SOURCE_ROOT}
source_rev=$(git -C "$SOURCE_ROOT" rev-parse HEAD)
source_dirty=$([[ -n "$(git -C "$SOURCE_ROOT" status --short)" ]] && echo true || echo false)
redesign_flag=${REDESIGN_FLAG}
EOF

    cat >"$PKG_DIR/install.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
PREFIX="${PREFIX:-/usr/local}"
echo "Installing custom Codex to ${PREFIX}/bin"
sudo mkdir -p "${PREFIX}/bin" "${PREFIX}/share/custom-codex-release/libexec" "${PREFIX}/share/custom-codex-release/lib"
sudo cp bin/codex bin/codex-legacy "${PREFIX}/bin/"
sudo chmod +x "${PREFIX}/bin/codex" "${PREFIX}/bin/codex-legacy"
sudo cp libexec/codex "${PREFIX}/share/custom-codex-release/libexec/"
sudo chmod +x "${PREFIX}/share/custom-codex-release/libexec/codex"
if [[ -d lib ]]; then
    sudo cp -r lib/* "${PREFIX}/share/custom-codex-release/lib/" 2>/dev/null || true
fi
echo "Done."
EOF
    chmod +x "$PKG_DIR/install.sh"

    TAR_PATH="$DIST_ROOT/codex-${system}.tar.gz"
    tar -czf "$TAR_PATH" -C "$DIST_ROOT" "codex-${system}"
    sha256sum "$TAR_PATH" >"${TAR_PATH}.sha256"
    TAR_SHA256="$(cut -d' ' -f1 "${TAR_PATH}.sha256")"

    ok "Packaged $TAR_PATH"
    log "SHA256: $TAR_SHA256"
}

ensure_release_repo() {
    if [[ -d "$RELEASE_DIR/.git" ]]; then
        log "Using release repo: $RELEASE_DIR"
        return
    fi

    mkdir -p "$(dirname "$RELEASE_DIR")"

    if [[ "$DO_GH" == "1" ]] && gh repo view "${RELEASE_OWNER}/${RELEASE_REPO}" >/dev/null 2>&1; then
        log "Cloning ${RELEASE_OWNER}/${RELEASE_REPO} into $RELEASE_DIR"
        gh repo clone "${RELEASE_OWNER}/${RELEASE_REPO}" "$RELEASE_DIR"
        return
    fi

    log "Creating local release repo at $RELEASE_DIR"
    mkdir -p "$RELEASE_DIR"
    git -C "$RELEASE_DIR" init -b main >/dev/null

    if [[ "$DO_GH" == "1" ]]; then
        log "Creating GitHub repo ${RELEASE_OWNER}/${RELEASE_REPO}"
        gh repo create "${RELEASE_OWNER}/${RELEASE_REPO}" --private --source "$RELEASE_DIR" --remote origin
    fi
}

existing_release_checks() {
    if [[ "$DRY_RUN" == "1" ]]; then
        return
    fi

    if git -C "$RELEASE_DIR" tag -l | grep -qx "$VERSION"; then
        if [[ "$FORCE" == "1" ]]; then
            warn "Deleting existing local tag $VERSION in release repo"
            git -C "$RELEASE_DIR" tag -d "$VERSION" >/dev/null
        else
            error "Local release repo tag $VERSION already exists. Use --force to replace it."
            exit 1
        fi
    fi

    if [[ "$DO_GH" == "1" ]]; then
        if git -C "$RELEASE_DIR" ls-remote --tags origin | grep -q "refs/tags/${VERSION}$"; then
            if [[ "$FORCE" == "1" ]]; then
                warn "Deleting remote tag $VERSION in ${RELEASE_OWNER}/${RELEASE_REPO}"
                git -C "$RELEASE_DIR" push origin ":refs/tags/${VERSION}"
            else
                error "Remote release repo tag $VERSION already exists. Use --force to replace it."
                exit 1
            fi
        fi

        if gh release view "$VERSION" --repo "${RELEASE_OWNER}/${RELEASE_REPO}" >/dev/null 2>&1; then
            if [[ "$FORCE" == "1" ]]; then
                warn "Deleting existing GitHub release $VERSION in ${RELEASE_OWNER}/${RELEASE_REPO}"
                gh release delete "$VERSION" --repo "${RELEASE_OWNER}/${RELEASE_REPO}" --yes
            else
                error "GitHub release $VERSION already exists in ${RELEASE_OWNER}/${RELEASE_REPO}. Use --force to replace it."
                exit 1
            fi
        fi
    fi
}

emit_release_flake() {
    local system="$1"
    local sha="$2"
    local source_rev
    source_rev=$(git -C "$SOURCE_ROOT" rev-parse HEAD)

    cp -f "$TAR_PATH" "$RELEASE_DIR/codex-${system}.tar.gz"
    cp -f "${TAR_PATH}.sha256" "$RELEASE_DIR/codex-${system}.tar.gz.sha256"

    cat >"$RELEASE_DIR/flake.nix" <<EOF
{
  description = "Custom Codex release with prebuilt redesigned TUI binary";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    (flake-utils.lib.eachSystem [ "${system}" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;

        version = "${VERSION}";
        tarballSha256BySystem = {
          "${system}" = "${sha}";
        };
        tarballBySystem = {
          "${system}" = ./codex-${system}.tar.gz;
        };
        tarball =
          tarballBySystem.\${system} or
            (throw "No custom Codex prebuilt binary for \${system} in \${version}");

        runtimeDeps = with pkgs; [
          stdenv.cc.cc.lib
          openssl
          zlib
          zstd
          xz
          bzip2
        ] ++ lib.optionals stdenv.isLinux [
          libcap
        ];

        customCodex = pkgs.stdenv.mkDerivation {
          pname = "custom-codex";
          inherit version;

          src = tarball;

          nativeBuildInputs = [
            pkgs.autoPatchelfHook
            pkgs.makeWrapper
          ];
          buildInputs = runtimeDeps;

          dontStrip = true;
          autoPatchelfIgnoreMissingDeps = true;

          unpackPhase = ''
            tar xzf \$src
            cd codex-\${system}
          '';

          installPhase = ''
            mkdir -p \$out/bin \$out/lib \$out/libexec \$out/share/custom-codex-release

            if [ -f libexec/codex ]; then
              cp libexec/codex \$out/libexec/codex
            elif [ -f bin/codex ] && head -c4 bin/codex 2>/dev/null | grep -q \$'\\x7fELF'; then
              cp bin/codex \$out/libexec/codex
            else
              echo "ERROR: release tarball does not contain an ELF Codex binary" >&2
              exit 1
            fi
            chmod +x \$out/libexec/codex

            if [ -d lib ]; then
              for f in lib/*; do
                case "\$(basename "\$f")" in
                  libc.so*|libc-*.so*|libpthread.so*|libpthread-*.so*|libdl.so*|libdl-*.so*|librt.so*|librt-*.so*|libm.so*|libm-*.so*|ld-linux*|libmvec.so*|libresolv.so*|libnss_*|libBrokenLocale*|libSegFault*|libthread_db*|libanl*|libutil.so*|libcrypt.so*) ;;
                  *) cp -L "\$f" \$out/lib/ 2>/dev/null || true ;;
                esac
              done
            fi

            cp -r share/custom-codex-release/* \$out/share/custom-codex-release/ 2>/dev/null || true
          '';

          postFixup = ''
            if [ -d \$out/lib ]; then
              patchelf --add-rpath \$out/lib \$out/libexec/codex 2>/dev/null || true
            fi
            makeWrapper \$out/libexec/codex \$out/bin/codex \\
              --add-flags "${REDESIGN_FLAG}"
            makeWrapper \$out/libexec/codex \$out/bin/codex-legacy
          '';

          meta = with lib; {
            description = "Custom Codex binary with redesigned TUI enabled by default";
            homepage = "https://github.com/${RELEASE_OWNER}/${RELEASE_REPO}";
            license = licenses.asl20;
            mainProgram = "codex";
            platforms = platforms.linux;
          };
        };
      in {
        packages = {
          default = customCodex;
          codex = customCodex;
          custom-codex = customCodex;
          codex-legacy = customCodex;
        };

        checks."tarball-sha256" = pkgs.runCommand "custom-codex-tarball-sha256" { } ''
          actual=\$(\${pkgs.coreutils}/bin/sha256sum \${tarball} | \${pkgs.coreutils}/bin/cut -d' ' -f1)
          expected="${sha}"
          if [ "\$actual" != "\$expected" ]; then
            echo "sha256 mismatch for \${tarball}" >&2
            echo "expected: \$expected" >&2
            echo "actual:   \$actual" >&2
            exit 1
          fi
          touch \$out
        '';

        apps = {
          default = {
            type = "app";
            program = "\${customCodex}/bin/codex";
          };
          codex = {
            type = "app";
            program = "\${customCodex}/bin/codex";
          };
          codex-legacy = {
            type = "app";
            program = "\${customCodex}/bin/codex-legacy";
          };
        };
      }
    )) // {
      overlays.default = final: prev: {
        codex = self.packages.\${final.system}.codex;
        custom-codex = self.packages.\${final.system}.custom-codex;
        codex-legacy = self.packages.\${final.system}.codex-legacy;
      };

      nixosModules = {
        default = import ./nix/modules/custom-codex.nix { inherit self; };
        custom-codex = import ./nix/modules/custom-codex.nix { inherit self; };
      };
    };
}
EOF

    mkdir -p "$RELEASE_DIR/nix/modules"
    cat >"$RELEASE_DIR/nix/modules/custom-codex.nix" <<'EOF'
{ self }:

{ config, lib, pkgs, ... }:

let
  cfg = config.programs.custom-codex;
  system = pkgs.stdenv.hostPlatform.system;
in
{
  options.programs.custom-codex = {
    enable = lib.mkEnableOption "custom Codex with the redesigned TUI enabled by default";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${system}.codex;
      defaultText = lib.literalExpression "custom-codex-release.packages.${pkgs.system}.codex";
      description = ''
        Package that provides the custom Codex commands. The default package
        installs `codex` with the redesigned TUI enabled and `codex-legacy`
        without the redesign flag.
      '';
    };

    setDefaultEditor = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Whether to set EDITOR to the redesigned Codex command for all users.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    environment.variables = lib.mkIf cfg.setDefaultEditor {
      EDITOR = "codex";
    };
  };
}
EOF

    cat >"$RELEASE_DIR/README.md" <<EOF
# Custom Codex Release

Version: ${VERSION}
Source commit: ${source_rev}
Published systems:

- ${system}: \`${sha}\`

This private flake distributes a prebuilt Codex binary for downstream Nix
systems. The tarball is committed into this repository and consumed via
\`git+ssh\`, matching the private release style used by the trading repo.

The \`codex\` command starts with \`${REDESIGN_FLAG}\` by default. The
\`codex-legacy\` command is preserved for the legacy TUI.

## NixOS Module Usage

Use \`git+ssh\`, not \`github:\`, so Nix fetches the private repository through
SSH and can see the committed tarball.

\`\`\`nix
{
  inputs.custom-codex-release.url =
    "git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=release/${VERSION}";

  outputs = { nixpkgs, custom-codex-release, ... }: {
    nixosConfigurations.host = nixpkgs.lib.nixosSystem {
      system = "${system}";
      modules = [
        custom-codex-release.nixosModules.default
        {
          programs.custom-codex.enable = true;
        }
      ];
    };
  };
}
\`\`\`

The module installs the package into \`environment.systemPackages\`. That
package provides both commands:

- \`codex\`: redesigned TUI by default.
- \`codex-legacy\`: legacy TUI, without \`${REDESIGN_FLAG}\`.

### Module Options

\`\`\`nix
programs.custom-codex = {
  enable = true;
  package = custom-codex-release.packages.\${pkgs.system}.codex;
  setDefaultEditor = false;
};
\`\`\`

## Package Usage Without Module

\`\`\`nix
{
  inputs.custom-codex-release.url =
    "git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=release/${VERSION}";

  outputs = { nixpkgs, custom-codex-release, ... }: {
    nixosConfigurations.host = nixpkgs.lib.nixosSystem {
      system = "${system}";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [
            custom-codex-release.packages.\${pkgs.system}.codex
          ];
        })
      ];
    };
  };
}
\`\`\`

## Direct Run

\`\`\`sh
nix run "git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=release/${VERSION}"
nix run "git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=release/${VERSION}#codex-legacy"
\`\`\`

## Floating Latest Input

For machines where you intentionally want the newest published binary:

\`\`\`nix
inputs.custom-codex-release.url =
  "git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=latest";
\`\`\`

## Flake Outputs

- \`packages.<system>.codex\`: redesign UI by default, plus \`codex-legacy\`.
- \`packages.<system>.custom-codex\`: alias of \`codex\`.
- \`packages.<system>.codex-legacy\`: compatibility alias; the package still
  installs both \`codex\` and \`codex-legacy\`.
- \`nixosModules.default\`: installs the custom Codex package through
  \`programs.custom-codex.enable\`.
- \`nixosModules.custom-codex\`: alias of the default module.

Regenerate this repo from the Codex checkout with:

\`\`\`sh
codex-rs/scripts/create-custom-codex-release.sh --version ${VERSION}
\`\`\`
EOF

    cat >"$RELEASE_DIR/.gitignore" <<'EOF'
result
result-*
.direnv/
.envrc
EOF

    ok "Generated release flake in $RELEASE_DIR"
}

commit_release_repo() {
    if [[ "$DRY_RUN" == "1" ]]; then
        log "Dry run: skipping release repo commit/tag"
        return
    fi

    git -C "$RELEASE_DIR" add \
        flake.nix \
        README.md \
        .gitignore \
        "codex-${SYSTEM}.tar.gz" \
        "codex-${SYSTEM}.tar.gz.sha256" \
        nix/modules/custom-codex.nix

    if [[ -z "$(git -C "$RELEASE_DIR" status --short)" ]]; then
        warn "Release repo has no file changes to commit"
    else
        git -C "$RELEASE_DIR" commit -m "Release ${VERSION}"
    fi

    git -C "$RELEASE_DIR" tag -a "$VERSION" -m "Release ${VERSION}"
    git -C "$RELEASE_DIR" branch -f "release/${VERSION}" HEAD
    git -C "$RELEASE_DIR" branch -f latest HEAD

    if [[ "$DO_GH" == "1" ]]; then
        git -C "$RELEASE_DIR" push origin main
        git -C "$RELEASE_DIR" push origin "release/${VERSION}" latest
        git -C "$RELEASE_DIR" push origin "$VERSION"
    else
        log "Skipping release repo push (--no-gh or --dry-run)"
    fi
}

publish_github_release() {
    if [[ "$DO_GH" != "1" ]]; then
        log "Skipping GitHub release (--no-gh or --dry-run)"
        return
    fi

    local notes_file
    notes_file=$(mktemp)
    cat >"$notes_file" <<EOF
# Custom Codex ${VERSION}

Prebuilt Codex binary for downstream Nix systems.

- System: ${SYSTEM}
- SHA256: ${TAR_SHA256}
- Source commit: $(git -C "$SOURCE_ROOT" rev-parse HEAD)
- Downstream input: git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=release/${VERSION}
- \`codex\` runs with \`${REDESIGN_FLAG}\`
- \`codex-legacy\` is preserved without \`${REDESIGN_FLAG}\`
EOF

    local release_args=(
        "$VERSION"
        "$TAR_PATH"
        "${TAR_PATH}.sha256"
        "--repo" "${RELEASE_OWNER}/${RELEASE_REPO}"
        "--title" "Custom Codex ${VERSION}"
        "--notes-file" "$notes_file"
    )
    [[ "$DRAFT" == "1" ]] && release_args+=("--draft")
    [[ "$PRERELEASE" == "1" ]] && release_args+=("--prerelease")

    if [[ -z "${CI:-}" ]]; then
        env -u GITHUB_TOKEN -u GH_TOKEN gh release create "${release_args[@]}"
    else
        gh release create "${release_args[@]}"
    fi

    rm -f "$notes_file"
    ok "Published GitHub release ${RELEASE_OWNER}/${RELEASE_REPO}@${VERSION}"
}

main() {
    parse_args "$@"
    trap 'error "Interrupted"; exit 130' INT TERM
    preflight

    SYSTEM="$(current_system)"
    log "Target system: $SYSTEM"

    local built_bin
    built_bin="$(build_codex_binary)"
    package_binary "$built_bin" "$SYSTEM"

    ensure_release_repo
    existing_release_checks
    emit_release_flake "$SYSTEM" "$TAR_SHA256"
    commit_release_repo
    publish_github_release

    ok "Done"
    echo
    echo "Release repo: $RELEASE_DIR"
    echo "Tarball: $TAR_PATH"
    echo "Downstream input: git+ssh://git@github.com/${RELEASE_OWNER}/${RELEASE_REPO}.git?ref=release/${VERSION}"
}

main "$@"
