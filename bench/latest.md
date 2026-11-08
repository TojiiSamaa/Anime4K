### Benchmark — clean source, 960x540 → 1920x1080, preset C, 3 images
`AMD EPYC 9V74 80-Core Processor` · 4 threads · commit `3957751` · 2026-11-08T04:44:00Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 211 | 34.42 dB | 0.9622 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `3957751` | 1265 | 38.60 dB | 0.9711 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `3957751` | 166 | 38.58 dB | 0.9712 | 67.06 dB | 6/6 runs ok · deterministic |
| anime4k-rs (native, size S) | `3957751` | 58 | 37.11 dB | 0.9652 | — | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 113 | 32.49 dB | 0.9428 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._