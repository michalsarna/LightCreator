#!/usr/bin/env bash
# Build the GRBL simulator (grbl-sim) from the real GRBL 1.1h firmware sources.
#
#   tools/grbl-sim/build.sh [output-dir]      default: target/grbl-sim
#
# The result is `<output-dir>/grbl/grbl/sim/grbl_sim.exe` (the `.exe` suffix is grbl-sim's own naming, it is a
# normal Linux / macOS binary). It speaks the exact GRBL serial protocol on stdin / stdout, so LightCreator can be
# tested against the real firmware code without an Arduino; see docs/grbl-sim.md.
set -euo pipefail

GRBL_REF="${GRBL_REF:-v1.1h.20190825}"   # gnea/grbl tag
SIM_REF="${SIM_REF:-master}"             # grbl/grbl-sim branch or commit

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
out="${1:-$root/target/grbl-sim}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

if [ ! -d "$out/grbl/.git" ]; then
    git -c advice.detachedHead=false clone --quiet --depth 1 --branch "$GRBL_REF" https://github.com/gnea/grbl.git "$out/grbl"
fi
sim="$out/grbl/grbl/sim"
if [ ! -d "$sim/.git" ]; then
    git clone --quiet https://github.com/grbl/grbl-sim.git "$sim"
    git -C "$sim" -c advice.detachedHead=false checkout --quiet "$SIM_REF"
fi

case "$(uname -s)" in
    Darwin) platform=OSX ;;
    *) platform=LINUX ;;
esac

# -fcommon: grbl-sim defines globals in headers, which GCC >= 10 / recent clang reject by default
# ("multiple definition of `wdt'").
make -C "$sim" --no-print-directory new PLATFORM="$platform" FLAGS="-g -O2 -fcommon -w" >/dev/null

echo "$sim/grbl_sim.exe"
