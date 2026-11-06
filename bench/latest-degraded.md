### Benchmark — blurred + H.264 source, 960x540 → 1920x1080, preset A, 3 images
`INTEL(R) XEON(R) PLATINUM 8573C` · 4 threads · commit `67d3ca1` · 2026-11-06T05:07:59Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 198 | 29.37 dB | 0.8627 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `67d3ca1` | 2596 | 30.84 dB | 0.8736 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `67d3ca1` | 286 | 30.84 dB | 0.8736 | 65.03 dB | 6/6 runs ok · deterministic |
| anime4k-rs (native, size S) | `67d3ca1` | 95 | 30.41 dB | 0.8666 | — | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 144 | 30.02 dB | 0.8560 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._