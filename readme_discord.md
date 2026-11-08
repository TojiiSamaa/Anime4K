![anime4k-rs](https://raw.githubusercontent.com/TojiiSamaa/anime4k-rust/main/assets/discord-apercu.png)
anime4k-rs applique les shaders Anime4K (bloc97) une seule fois à tout un fichier vidéo, pour que l'épisode upscalé se lise partout, même sur un téléphone, sans shader à la lecture.
Les réseaux de neurones ne sont pas modifiés : mêmes couches, mêmes poids, lus directement dans les fichiers .glsl officiels.
Deux moteurs au choix : un interpréteur en Rust pur sur CPU, ou ffmpeg + libplacebo en Vulkan sur GPU, avec choix de la carte via --gpu.
Audio, sous-titres, polices et chapitres sont recopiés, et un banc de mesure mis à jour par la CI compare vitesse et qualité aux autres outils.
