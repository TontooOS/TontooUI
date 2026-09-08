#!/usr/bin/env bash
# Element Review loop for TontooUI (WSL/Linux).
#
# Builds the review app once, then relaunches it after every decision:
# Ja/Skip/Zurueck continue automatically, Nein/Done/close stop the loop.
# Decisions live in temp/review/ so a restart resumes at the same element.
#
# Usage (one command from Windows):
#   wsl -d archlinux -- bash -lc "cd /mnt/c/Users/arlo1/Documents/TontooLibs/TontooUI && bash review.sh"

set -u
cd "$(dirname "$0")"

mkdir -p temp/review/approved temp/review/rejected
if [ ! -f temp/review/state.json ]; then
  printf '{"index":0,"done":false}' > temp/review/state.json
fi

cargo build --example review || exit 1

while true; do
  before=$(cat temp/review/state.json)
  ./target/debug/examples/review
  code=$?
  after=$(cat temp/review/state.json)
  if grep -q '"done":true' temp/review/state.json; then
    echo "Review fertig!"
    break
  fi
  if [ "$code" -eq 4 ]; then
    echo "Nein markiert - gestoppt. Fix sagen, dann weiter."
    break
  fi
  if [ "$after" = "$before" ]; then
    echo "Ohne Entscheidung geschlossen - gestoppt."
    break
  fi
done
