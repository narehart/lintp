#!/usr/bin/env bash
#
# Fails when a NEW pair of similar Rust functions appears.
#
# similarity-rs runs at its default sensitivity here, because tuning the
# threshold up to silence known pairs also silences real copy-paste: two
# functions differing only in prefix/suffix score 91.78%, so any gate above
# that misses them. Instead every pair the tool reports is compared against a
# reviewed baseline. Pairs in the baseline pass; anything new fails.
#
# The baseline is a list to shrink, not a place to park duplication. Each
# entry should be a pair that shares a shape rather than an implementation —
# caller/callee, or two dispatchers. Real duplication gets fixed, not added.
set -uo pipefail

BASELINE="${BASELINE:-scripts/similarity-baseline.txt}"
TARGET="${1:-src}"

if ! command -v similarity-rs >/dev/null 2>&1; then
  echo "similarity-rs not found. Install it with: cargo install similarity-rs --locked" >&2
  exit 127
fi

# Strip line numbers: they shift with every edit and say nothing about whether
# two functions are duplicates.
normalize() {
  sed -nE 's|^ +([^ :]+):[0-9]+-[0-9]+ function ([A-Za-z0-9_]+) <-> ([^ :]+):[0-9]+-[0-9]+ function ([A-Za-z0-9_]+)$|\1::\2 <-> \3::\4|p' |
    sort -u
}

report=$(similarity-rs --min-lines 5 "$TARGET" 2>&1)
echo "$report"

current=$(printf '%s\n' "$report" | normalize)
baseline=$(grep -vE '^\s*(#|$)' "$BASELINE" | sort -u)

new_pairs=$(comm -23 <(printf '%s\n' "$current") <(printf '%s\n' "$baseline"))
fixed_pairs=$(comm -13 <(printf '%s\n' "$current") <(printf '%s\n' "$baseline"))

if [ -n "$fixed_pairs" ]; then
  echo
  echo "These baseline pairs no longer register — please remove them from $BASELINE:"
  printf '%s\n' "$fixed_pairs" | sed 's/^/  /'
fi

if [ -n "$new_pairs" ]; then
  echo
  echo "New duplicate logic (not in $BASELINE):"
  printf '%s\n' "$new_pairs" | sed 's/^/  /'
  echo
  echo "Factor out the shared part. Only add to the baseline if the two genuinely"
  echo "share a shape rather than an implementation, and say why in the file."
  exit 1
fi

echo
echo "No new duplicate logic."
