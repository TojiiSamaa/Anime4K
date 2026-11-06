# anime4k-rs

Conversion « à froid » de vidéos avec les shaders [Anime4K](https://github.com/bloc97/Anime4K) de bloc97 : on donne un fichier, un preset (A par défaut), et on récupère une vidéo déjà traitée, lisible partout (téléphone compris).

Pour la lecture en direct dans mpv/Plex, les shaders d'origine restent la bonne solution : c'est le lecteur qui les exécute sur la carte graphique.

## Deux moteurs

| Moteur | Quand | Comment |
|---|---|---|
| `libplacebo` (défaut conseillé avec GPU) | machine avec carte graphique | ffmpeg + libplacebo exécutent les `.glsl` d'origine (Vulkan), encodage matériel possible |
| `natif` | machine **sans** GPU | interpréteur Rust des shaders sur CPU (AVX2/FMA, tous les cœurs), sans dépendance |

Le moteur natif lit les poids directement dans les `.glsl` de la release officielle : aucun poids n'est recopié dans ce dépôt.

## Mesures (preset A, 1080p → 2160p)

| Machine | Moteur | Vitesse |
|---|---|---|
| 2× Xeon E5-2670 v3, 24 cœurs, sans GPU | libplacebo sur llvmpipe | 0,40 img/s |
| idem | **natif** | **0,86 img/s** |
| RTX 5080 | libplacebo | ~88 img/s (~58 img/s avec encodage HEVC NVENC) |

Fidélité du moteur natif face à libplacebo (mire RGB sans perte, 6 images) : **PSNR 54,5 dB, écart max 1,4/255** (différence de précision fp16 GPU / f32 CPU).

## Utilisation

```sh
# shaders officiels v4.0.1
curl -LO https://github.com/bloc97/Anime4K/releases/download/v4.0.1/Anime4K_v4.0.zip
unzip Anime4K_v4.0.zip -d shaders

cargo build --release
anime4k-rs episode.mkv                                  # preset A, 2160p, moteur natif
anime4k-rs episode.mkv --preset B+B --hauteur 1440
anime4k-rs episode.mkv --moteur libplacebo              # avec GPU
```

Options : `--preset A|B|C|A+A|B+B|C+A`, `--hauteur N`, `--sortie fichier.mkv`, `--moteur natif|libplacebo`, `--images N`, `--brut` (sortie gbrpf32le non compressée, pour les bancs), `--shaders <dossier>` (ou variable `ANIME4K_SHADERS`).

Banc de fidélité : `anime4k-rs comparer natif.raw ref.raw` (PSNR et écart max entre deux sorties `--brut`).

Le son, les sous-titres, les pièces jointes (polices) et les chapitres sont recopiés tels quels. Sortie par défaut : HEVC 10 bits (libx265, CRF 20).

`A4K_PROFIL=1` affiche le temps de chaque passe.

## Prérequis

- `ffmpeg` et `ffprobe` (avec libplacebo et Vulkan pour le moteur `libplacebo`)
- Rust 1.85+ (édition 2024)

## Licence

MIT (`LICENSE`). Les shaders et l'algorithme Anime4K sont de bloc97, sous licence MIT (`LICENSE-Anime4K`).
