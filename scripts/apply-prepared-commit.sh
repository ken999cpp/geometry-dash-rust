#!/usr/bin/env bash
# Import the prepared commit without replacing any working-tree files.
set -euo pipefail
cd -- "$(dirname -- "$0")/.."

task_bundle="${1:-$PWD/dist/geometry-dash-prepared.bundle}"
task_origin="${2:-https://github.com/ken999cpp/geometry-dash-rust.git}"
if [[ ! -f "$task_bundle" ]]; then
    printf 'Prepared Git bundle not found: %s\n' "$task_bundle" >&2
    exit 1
fi
if git rev-parse --verify HEAD >/dev/null 2>&1; then
    printf 'This repository already has commits. Import the bundle on a separate branch for review.\n' >&2
    exit 1
fi

bash scripts/setup-git.sh
git bundle verify "$task_bundle"
git fetch "$task_bundle" refs/heads/main
git update-ref refs/heads/main FETCH_HEAD
git symbolic-ref HEAD refs/heads/main
git reset --mixed HEAD
if git remote get-url origin >/dev/null 2>&1; then
    git remote set-url origin "$task_origin"
else
    git remote add origin "$task_origin"
fi
git log -1 --oneline
git status --short
