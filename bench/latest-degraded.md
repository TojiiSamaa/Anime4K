### Benchmark — blurred + H.264 source, 960x540 → 1920x1080, preset A, 3 images
`AMD EPYC 9V45 96-Core Processor` · 4 threads · commit `9d3ec42` · 2026-11-06T03:42:33Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 127 | 29.37 dB | 0.8627 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `9d3ec42` | 1624 | 30.84 dB | 0.8736 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `9d3ec42` | 165 | 30.84 dB | 0.8736 | 65.03 dB | 6/6 runs ok · deterministic |
| anime4k-rs (native, size S) | `9d3ec42` | 88 | 30.41 dB | 0.8666 | — | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 81 | 30.02 dB | 0.8560 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._