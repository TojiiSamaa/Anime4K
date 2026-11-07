### Benchmark — blurred + H.264 source, 960x540 → 1920x1080, preset A, 3 images
`AMD EPYC 9V45 96-Core Processor` · 4 threads · commit `46500e5` · 2026-11-07T03:40:55Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 137 | 29.37 dB | 0.8627 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `46500e5` | 1829 | 30.84 dB | 0.8736 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `46500e5` | 189 | 30.84 dB | 0.8736 | 65.03 dB | 6/6 runs ok · deterministic |
| anime4k-rs (native, size S) | `46500e5` | 84 | 30.41 dB | 0.8666 | — | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 89 | 30.02 dB | 0.8560 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._