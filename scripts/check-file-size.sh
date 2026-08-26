#!/usr/bin/env bash
#
# Fails when a tracked file exceeds the size limit.
#
# Git keeps every version of a large file forever, so one accidental commit of
# a build artifact, a vendored binary or a screen recording is permanent
# repository weight that a later delete does not reclaim. This catches it at
# review time, when it is still one revert away.
#
# There is no exclusion list: if something legitimately needs to be larger,
# raise the limit deliberately and say why in the commit.
set -uo pipefail

MAX_KB="${MAX_KB:-1024}"

too_big=$(git ls-files -z |
  xargs -0 -I{} sh -c 'test -f "{}" && printf "%s\t%s\n" "$(wc -c < "{}")" "{}"' |
  awk -F'\t' -v max="$((MAX_KB * 1024))" '$1 > max {printf "  %.1f KB  %s\n", $1/1024, $2}' |
  sort -rn)

if [ -n "$too_big" ]; then
  echo "Files over ${MAX_KB} KB:"
  echo "$too_big"
  echo
  echo "Git stores every revision of these forever. Keep large assets outside"
  echo "the repo, or raise MAX_KB in this script deliberately."
  exit 1
fi

echo "No tracked file exceeds ${MAX_KB} KB."
