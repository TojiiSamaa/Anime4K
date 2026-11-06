### Benchmark — 960x540 → 1920x1080, preset A, 3 images
`AMD EPYC 9V74 80-Core Processor` · 4 threads · commit `fd3fa22` · 2026-11-06T02:15:44Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 211 | 34.42 dB | 0.9622 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `fd3fa22` | 2117 | 33.68 dB | 0.9462 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `fd3fa22` | 379 | 33.70 dB | 0.9460 | 56.05 dB | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 115 | 32.49 dB | 0.9428 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._