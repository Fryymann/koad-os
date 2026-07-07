# KoadOS Installer & Updater — Operating Document

**Owner:** Clyde · **Last reviewed:** 2026-07-06 · **Applies to:** v3.2.0+

Single entrypoint: `./install.sh` at the repo root. Two modes, one path flag.

```
./install.sh --install [--home PATH]   # fresh Citadel setup (default mode)
./install.sh --update                  # upgrade every locally installed Citadel
```

## Target directory resolution (`--install`)

Precedence: `--home PATH` → exported `KOAD_HOME` → exported `KOADOS_HOME` → `~/.koad-os`.

⚠️ **Sourced koad shells export `KOADOS_HOME` pointing at the live instance.** Running
`--install` from such a shell without `--home` targets the live install. Two guards (added
2026-07-06 after a config-clobber incident):

- Non-interactive (`no TTY`) `--install` **aborts with exit 1** when the target directory
  already exists. Re-run interactively, or pass `--home <fresh-path>` for side-by-side.
- Interactive runs prompt before deleting/overwriting an existing installation.

## Fresh install flow (`--install`)

1. **Prerequisites** — cargo, docker, protoc, sqlite3, redis-server, python3, pipx,
   docker-compose. All checked *before* any filesystem mutation; missing prereqs abort.
   Bootstrap them with `./koad-setup.sh`.
   - The compose check is *functional* (`docker-compose version`), not `command -v`:
     WSL ships a shim that exists on PATH but errors when Docker Desktop integration is off.
2. **Interactive identity** — prompts for Citadel Name (default `Sanctuary`) and Captain
   agent (default `Tyr`); non-interactive runs take the defaults.
3. **Build** — `cargo build --release`, binaries copied to `$KOAD_HOME/bin`.
4. **Assets** — `config/`, `scripts/`, `plugin/skills/` copied under `$KOAD_HOME`;
   `.env` created from `.env.template` (existing `.env` preserved); redis conf rendered.
5. **Infrastructure** — `docker-compose up -d --build` (CASS, Redis, Qdrant).
6. **Init** — `koad-init.sh` seeds identity/db. Agent identities stay local and
   git-ignored (`config/identities/*.toml`); create more with `koad agent new`.
7. **systemd** — service templates rendered from `config/systemd/`; enabling/starting
   uses `sudo -n` and degrades to warnings without passwordless sudo.

## Update flow (`--update`)

Iterates every detected install (e.g. `~/.citadel-jupiter`, `~/.koad-os`) and refreshes:
release **binaries**, `koad-functions.sh`, `agent-boot.sh`, `scripts/`, `skills/`.
It does **not** touch `config/`, `.env`, identities, or databases.

⚠️ **Silent-restart gotcha:** the update restarts services with `sudo -n systemctl …`,
which fails silently without passwordless sudo — binaries on disk update while the
RUNNING process keeps executing old code from memory. Always follow an update with an
explicit restart **in a real terminal** (the prompt needs a TTY):

```bash
sudo systemctl restart koad-cass.service koad-citadel.service
```

Verify the swap took:

```bash
readlink -f /proc/$(pgrep -f koad-cass | head -1)/exe   # expect $KOAD_HOME/bin/koad-cass
stat -c %y $KOAD_HOME/bin/koad-cass                     # mtime = build time
source $KOAD_HOME/bin/koad-functions.sh && koad system status
```

## Docker on WSL2 (Windows hosts)

- Docker Desktop must be **running on Windows** and the distro enabled under
  *Settings → Resources → WSL integration*; otherwise `docker ps` fails and the
  PATH-visible `docker-compose` shim errors (the preflight now catches this).
- Cold `cargo build --release` of the workspace plus container builds is memory-hungry.
  Give WSL headroom in `%UserProfile%\.wslconfig` (8 GB+ recommended):

  ```ini
  [wsl2]
  memory=12GB
  processors=6
  swap=8GB
  ```

- Container builder images track Rust **>= 1.90** (Edition-2024 dependencies); do not
  pin older toolchains in Dockerfiles.

## Sandboxed install testing (fleet verification)

Test the installer without touching a live instance:

```bash
git clone --branch main --depth 1 <origin-url> /tmp/fleet-test/src
cd /tmp/fleet-test/src
KOAD_HOME=/tmp/fleet-test/koad-home ./install.sh --install < /dev/null
```

Non-interactive + fresh `--home`/`KOAD_HOME` proceeds through preflight; on a machine
with a live install and no override it must abort before mutating anything (exit 1).
Verified 2026-07-06 on Jupiter for both cases. Full happy-path verification (through the
Docker boot and `koad-init`) still requires a host with working Docker — pending external
fleet machine.

## Uninstall

```bash
./scripts/uninstall.sh $KOAD_HOME --force
```

Removes the install directory; systemd unit removal needs sudo. Databases under
`$KOAD_HOME/data/` are destroyed with it — back up `cass.db` first if the memory matters.
