### Benchmark — clean source, 960x540 → 1920x1080, preset C, 3 images
`INTEL(R) XEON(R) PLATINUM 8573C` · 4 threads · commit `67d3ca1` · 2026-11-06T05:05:57Z

| Implementation | Version | ms / image | PSNR vs ground truth | SSIM | Fidelity vs GLSL | Reliability |
|---|---|---:|---:|---:|---:|---|
| bicubic (ffmpeg) | `-` | 258 | 34.42 dB | 0.9622 | — | 6/6 runs ok · deterministic |
| Anime4K GLSL via libplacebo | `67d3ca1` | 1538 | 38.60 dB | 0.9711 | — | 6/6 runs ok · deterministic |
| anime4k-rs (native) | `67d3ca1` | 180 | 38.58 dB | 0.9712 | 67.06 dB | 6/6 runs ok · deterministic |
| anime4k-rs (native, size S) | `67d3ca1` | 68 | 37.11 dB | 0.9652 | — | 6/6 runs ok · deterministic |
| andraantariksa/Anime4K-rs | `v0.2` | 153 | 32.49 dB | 0.9428 | — | 6/6 runs ok · deterministic |

_Ground truth: 1080p crops of Pepper&Carrot (David Revoy, CC BY 4.0) downscaled 2× then upscaled back. PSNR/SSIM reward closeness, not perceived sharpness. Fidelity = PSNR between our native engine and the original shaders run by libplacebo (higher = closer)._