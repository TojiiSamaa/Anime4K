### Benchmark — clean source, 960x540 → 1920x1080, preset C, 3 images
`AMD EPYC 9V45 96-Core Processor` · 4 threads · commit `46500e5` · 2026-11-07T03:39:28Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 178 | 34.42 dB | 0.9622 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `46500e5` | 1026 | 38.60 dB | 0.9711 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `46500e5` | 123 | 38.58 dB | 0.9712 | 67.06 dB | 6/6 runs ok · deterministic |
| anime4k-rs (native, size S) | `46500e5` | 48 | 37.11 dB | 0.9652 | — | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 94 | 32.49 dB | 0.9428 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._