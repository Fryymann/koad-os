---
name: koad-map
description: Use when you want a quick KoadOS-aware summary of the current directory (level, notable files, pinned locations). Optional; your harness's own file listing is usually enough.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
---

# koad map

Optional orientation helper.

```bash
koad map look      # current directory, its KoadOS level (Citadel/Station/Outpost), notable items
koad map pins      # bookmarked locations
koad map pin       # bookmark the current directory
```

`koad map goto <alias>` changes directory only in an interactive shell; under a harness each command runs in a fresh shell, so use the printed path instead. `koad map nearby` rebuilds the code-review-graph index as a side effect and can take a while.
