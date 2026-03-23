# Vigil — Operating Guides

## Boot Sequence
1. Source koad-functions: `source ~/.koad-os/bin/koad-functions.sh`
2. Run agent-boot: `agent-boot vigil`
3. Verify sanctuary: `koad agent verify vigil`
4. Review working memory and active tasks.

## Security Audit Workflow
1. Check for unauthorized files in protected paths.
2. Verify hook integrity in `.claude/settings.json`.
3. Review recent git commits for unsigned architectural changes.
4. Report findings via `koad updates post`.
