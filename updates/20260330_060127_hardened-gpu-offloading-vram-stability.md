+++
id        = "upd_20260330_060127_hardened-gpu-offloading-vram-stability"
timestamp = "2026-03-30T06:01:27.237254615+00:00"
author    = "tyr"
level     = "citadel"
category  = "infra"
summary   = "HARDENED: GPU Offloading & VRAM Stability"
+++

Enforced 'num_gpu: 99' in OllamaClient. Configured OLLAMA_NUM_PARALLEL=1 in systemd. Added 'koad doctor --gpu' for live VRAM delta verification. Exported LD_LIBRARY_PATH in agent-boot.
