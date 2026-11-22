#!/bin/sh
# Install a packaged binary, or the binary supplied by make install.
set -eu
binary=${1:-"$(dirname "$0")/declutter"}
bindir=${PREFIX:-"$HOME/.local"}/bin
mkdir -p "$bindir"
tmp=$(mktemp "$bindir/.declutter.XXXXXX")
trap 'rm -f "$tmp"' EXIT
install -m 755 "$binary" "$tmp"
mv -f "$tmp" "$bindir/declutter"
printf 'Installed %s/bin/declutter\n' "${PREFIX:-"$HOME/.local"}"
