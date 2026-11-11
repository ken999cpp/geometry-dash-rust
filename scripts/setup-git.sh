#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
if ! git rev-parse --git-dir >/dev/null 2>&1; then
    git init -b main
fi
excludes=$(git config --global --path --get core.excludesFile || true)
if [[ -z "$excludes" ]]; then
    excludes="${XDG_CONFIG_HOME:-$HOME/.config}/git/ignore"
fi
mkdir -p -- "$(dirname -- "$excludes")"
touch -- "$excludes"
for pattern in AGENTS.md specs.md; do
    if ! grep -Fxq -- "$pattern" "$excludes"; then
        printf '\n%s\n' "$pattern" >> "$excludes"
    fi
done
git check-ignore -v AGENTS.md specs.md
git status --short

