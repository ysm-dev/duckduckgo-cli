#!/bin/sh
set -eu

repo="ysm-dev/duckduckgo-cli"
prefix=""
version=""

linux_libc() {
  if ldd --version 2>&1 | grep -qi musl; then
    echo "duckduckgo-cli: musl Linux release artifacts are not available yet; use cargo install duckduckgo-cli" >&2
    exit 1
  fi
  echo gnu
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --prefix) prefix="$2"; shift 2 ;;
    --version) version="$2"; shift 2 ;;
    *) echo "duckduckgo-cli: unknown installer option $1" >&2; exit 2 ;;
  esac
done

os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
case "$os:$arch" in
  darwin:arm64) triple="aarch64-apple-darwin" ;;
  darwin:x86_64) triple="x86_64-apple-darwin" ;;
  linux:x86_64) triple="x86_64-unknown-linux-$(linux_libc)" ;;
  linux:aarch64|linux:arm64) triple="aarch64-unknown-linux-$(linux_libc)" ;;
  *) echo "duckduckgo-cli: unsupported platform $os/$arch" >&2; exit 1 ;;
esac

if [ -z "$version" ]; then
  version=$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
fi
[ -n "$version" ] || { echo "duckduckgo-cli: could not determine release version" >&2; exit 1; }

if [ -z "$prefix" ]; then
  if [ -n "${DUCKDUCKGO_INSTALL_DIR:-}" ]; then prefix="$DUCKDUCKGO_INSTALL_DIR";
  elif [ -d "$HOME/.local/bin" ] && printf '%s' "$PATH" | grep -q "$HOME/.local/bin"; then prefix="$HOME/.local";
  else prefix="/usr/local"; fi
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM
base="duckduckgo-$version-$triple.tar.gz"
url="https://github.com/$repo/releases/download/$version/$base"
curl -fsSL "$url" -o "$tmp/$base"
curl -fsSL "$url.sha256" -o "$tmp/$base.sha256"
(cd "$tmp" && shasum -a 256 -c "$base.sha256")
tar -xzf "$tmp/$base" -C "$tmp"
archive_dir="$tmp/duckduckgo-$version-$triple"

mkdir_cmd=mkdir
install_cmd=install
if [ ! -w "$prefix" ]; then mkdir_cmd="sudo mkdir"; install_cmd="sudo install"; fi
$mkdir_cmd -p "$prefix/bin" "$prefix/share/man/man1" "$prefix/share/bash-completion/completions" "$prefix/share/zsh/site-functions" "$prefix/share/fish/vendor_completions.d"
$install_cmd -m 0755 "$archive_dir/bin/duckduckgo" "$prefix/bin/duckduckgo"
ln -sf duckduckgo "$prefix/bin/ddg" 2>/dev/null || sudo ln -sf duckduckgo "$prefix/bin/ddg"
[ -f "$archive_dir/share/man/man1/duckduckgo.1" ] && $install_cmd -m 0644 "$archive_dir/share/man/man1/duckduckgo.1" "$prefix/share/man/man1/duckduckgo.1"
[ -f "$archive_dir/share/completions/duckduckgo.bash" ] && $install_cmd -m 0644 "$archive_dir/share/completions/duckduckgo.bash" "$prefix/share/bash-completion/completions/duckduckgo"
[ -f "$archive_dir/share/completions/_duckduckgo" ] && $install_cmd -m 0644 "$archive_dir/share/completions/_duckduckgo" "$prefix/share/zsh/site-functions/_duckduckgo"
[ -f "$archive_dir/share/completions/duckduckgo.fish" ] && $install_cmd -m 0644 "$archive_dir/share/completions/duckduckgo.fish" "$prefix/share/fish/vendor_completions.d/duckduckgo.fish"

case ":$PATH:" in
  *":$prefix/bin:"*) ;;
  *) echo "Add $prefix/bin to PATH to run duckduckgo." ;;
esac
