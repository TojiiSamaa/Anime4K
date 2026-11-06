### Benchmark — clean source, 960x540 → 1920x1080, preset C, 3 images
`AMD EPYC 7763 64-Core Processor` · 4 threads · commit `23d7f23` · 2026-11-06T02:31:53Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 245 | 34.42 dB | 0.9622 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `23d7f23` | 1582 | 38.60 dB | 0.9711 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `23d7f23` | 315 | 38.62 dB | 0.9711 | 55.84 dB | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 138 | 32.49 dB | 0.9428 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._