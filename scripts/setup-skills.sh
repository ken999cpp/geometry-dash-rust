#!/usr/bin/env bash
# Run in a normal terminal: the Codex sandbox cannot write the global skills folder.
set -euo pipefail

task_dry_run=false
if [[ "${1:-}" == --dry-run ]]; then
    task_dry_run=true
elif [[ $# -gt 0 ]]; then
    printf 'Usage: bash scripts/setup-skills.sh [--dry-run]\n' >&2
    exit 2
fi

task_codex_dir="${CODEX_HOME:-$HOME/.codex}"
task_skills_dir="$task_codex_dir/skills"
task_installer="$task_skills_dir/.system/skill-installer/scripts/install-skill-from-github.py"
if [[ ! -f "$task_installer" ]]; then
    printf 'Codex skill installer not found: %s\n' "$task_installer" >&2
    exit 1
fi

for task_skill in screenshot gh-fix-ci; do
    if [[ -d "$task_skills_dir/$task_skill" ]]; then
        printf 'Already installed: %s\n' "$task_skill"
        continue
    fi
    task_command=(python3 "$task_installer" --repo openai/skills
        --path "skills/.curated/$task_skill" --dest "$task_skills_dir")
    if "$task_dry_run"; then
        printf '%q ' "${task_command[@]}"
        printf '\n'
    else
        "${task_command[@]}"
    fi
done

if ! "$task_dry_run"; then
    printf 'Installed skills will be available on your next Codex turn.\n'
fi
