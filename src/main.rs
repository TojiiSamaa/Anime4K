// anime4k-rs : conversion « à froid » avec les shaders Anime4K (bloc97, MIT).
// Moteur natif CPU par défaut ; `--moteur libplacebo` = référence (ffmpeg + libplacebo).
mod moteur;

use std::{env, io::{Read, Write}, path::Path, process::{Command, Stdio}, time::Instant};

/// Dossier des .glsl de la release officielle : `ANIME4K_SHADERS`, sinon `./shaders`.
fn shaders() -> String { env::var("ANIME4K_SHADERS").unwrap_or_else(|_| "shaders".into()) }

/// Chaînes HQ officielles d'Anime4K v4 (modes A/B/C et doubles).
fn preset(nom: &str) -> Option<&'static [&'static str]> {
    Some(match nom {
        "A" => &["Clamp_Highlights", "Restore_CNN_VL", "Upscale_CNN_x2_VL", "AutoDownscalePre_x2", "AutoDownscalePre_x4", "Upscale_CNN_x2_M"],
        "B" => &["Clamp_Highlights", "Restore_CNN_Soft_VL", "Upscale_CNN_x2_VL", "AutoDownscalePre_x2", "AutoDownscalePre_x4", "Upscale_CNN_x2_M"],
        "C" => &["Clamp_Highlights", "Upscale_Denoise_CNN_x2_VL", "AutoDownscalePre_x2", "AutoDownscalePre_x4", "Upscale_CNN_x2_M"],
        "A+A" => &["Clamp_Highlights", "Restore_CNN_VL", "Upscale_CNN_x2_VL", "Restore_CNN_M", "AutoDownscalePre_x2", "AutoDownscalePre_x4", "Upscale_CNN_x2_M"],
        "B+B" => &["Clamp_Highlights", "Restore_CNN_Soft_VL", "Upscale_CNN_x2_VL", "AutoDownscalePre_x2", "AutoDownscalePre_x4", "Restore_CNN_Soft_M", "Upscale_CNN_x2_M"],
        "C+A" => &["Clamp_Highlights", "Upscale_Denoise_CNN_x2_VL", "AutoDownscalePre_x2", "AutoDownscalePre_x4", "Restore_CNN_M", "Upscale_CNN_x2_M"],
        _ => return None,
    })
}

fn quitter(msg: &str) -> ! { eprintln!("{msg}"); std::process::exit(1) }
fn usage() -> ! {
    quitter("usage :\n  anime4k-rs <entrée> [--preset A|B|C|A+A|B+B|C+A] [--hauteur 2160] [--sortie f.mkv] [--moteur natif|libplacebo] [--images N] [--brut] [--shaders <dossier>]\n  anime4k-rs comparer <a.raw> <b.raw>   (gbrpf32le, même taille)")
}

/// (largeur, hauteur, images/s) du premier flux vidéo.
fn sonder(f: &str) -> (usize, usize, String) {
    let o = Command::new("ffprobe").args(["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height,r_frame_rate", "-of", "csv=p=0", f])
        .output().unwrap_or_else(|e| quitter(&format!("ffprobe : {e}")));
    let s = String::from_utf8_lossy(&o.stdout);
    let v: Vec<&str> = s.trim().split(',').collect();
    if v.len() < 3 { quitter(&format!("{f} : pas de flux vidéo")) }
    (v[0].parse().unwrap(), v[1].parse().unwrap(), v[2].to_string())
}

fn comparer(a: &str, b: &str) {
    let (a, b) = (std::fs::read(a).unwrap(), std::fs::read(b).unwrap());
    if a.len() != b.len() { quitter(&format!("tailles différentes : {} / {}", a.len(), b.len())) }
    let (mut se, mut max) = (0f64, 0f32);
    let n = a.len() / 4;
    for (x, y) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let d = (f32::from_le_bytes(x.try_into().unwrap()).clamp(0.0, 1.0) - f32::from_le_bytes(y.try_into().unwrap()).clamp(0.0, 1.0)).abs();
        se += (d as f64).powi(2);
        max = max.max(d);
    }
    let psnr = 10.0 * (1.0 / (se / n as f64)).log10();
    println!("PSNR {psnr:.2} dB · écart max {max:.5} ({:.2}/255)", max * 255.0);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("comparer") {
        return comparer(args.get(1).unwrap_or_else(|| usage()), args.get(2).unwrap_or_else(|| usage()));
    }
    let entree = args.first().filter(|a| !a.starts_with("--")).unwrap_or_else(|| usage()).clone();
    let opt = |cle: &str| args.windows(2).find(|w| w[0] == cle).map(|w| w[1].clone());
    let brut = args.iter().any(|a| a == "--brut");
    let nom = opt("--preset").unwrap_or_else(|| "A".into());
    let dossier = opt("--shaders").unwrap_or_else(shaders);
    let moteur = opt("--moteur").unwrap_or_else(|| "natif".into());
    let images = opt("--images");
    let chaine = preset(&nom).unwrap_or_else(|| quitter(&format!("preset inconnu : {nom}")));
    let sources: Vec<String> = chaine.iter().map(|s| {
        let f = format!("{dossier}/Anime4K_{s}.glsl");
        std::fs::read_to_string(&f).unwrap_or_else(|e| quitter(&format!("{f} : {e}")))
    }).collect();

    let (w, h, fps) = sonder(&entree);
    let oh: usize = opt("--hauteur").and_then(|v| v.parse().ok()).unwrap_or(2160);
    let ow = ((w * oh) as f64 / h as f64 / 2.0).round() as usize * 2;
    let sortie = opt("--sortie").unwrap_or_else(|| {
        let p = Path::new(&entree);
        format!("{}/{}.anime4k-{}.{}", p.parent().map_or(".".into(), |d| d.display().to_string()),
            p.file_stem().unwrap().to_string_lossy(), nom.replace('+', "_"), if brut { "raw" } else { "mkv" })
    });
    let limite: Vec<String> = images.iter().flat_map(|n| ["-frames:v".to_string(), n.clone()]).collect();
    let t0 = Instant::now();

    if moteur == "libplacebo" {
        let glsl = env::temp_dir().join(format!("anime4k-{}.glsl", std::process::id()));
        std::fs::write(&glsl, sources.join("\n")).unwrap();
        // hwdownload refuse le float : référence brute via RGBA 16 bits puis conversion.
        let (fmt, fin) = if brut { ("rgba64le", ",format=gbrpf32le") } else { ("yuv420p10le", "") };
        let vf = format!("hwupload,libplacebo=w={ow}:h={oh}:custom_shader_path={}:format={fmt},hwdownload,format={fmt}{fin}", glsl.display());
        let mut c = Command::new("ffmpeg");
        c.args(["-hide_banner", "-loglevel", "error", "-init_hw_device", "vulkan", "-i", &entree, "-vf", &vf]).args(&limite);
        if brut { c.args(["-f", "rawvideo", "-y", &sortie]); } else { c.args(encodage()).args(["-y", &sortie]); }
        let ok = c.status().map(|s| s.success()).unwrap_or(false);
        let _ = std::fs::remove_file(&glsl);
        if !ok { quitter("échec ffmpeg/libplacebo") }
        return println!("{sortie} ({:.1} s)", t0.elapsed().as_secs_f64());
    }

    let m = moteur::Moteur::charger(&sources);
    // shortcut: matrice couleur lue dans le flux (repli BT.601 de swscale si non balisé), à forcer si un fichier HD non balisé sort décoloré.
    let mut dec = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i", &entree, "-map", "0:v:0"]).args(&limite)
        .args(["-vf", "scale=in_color_matrix=auto", "-f", "rawvideo", "-pix_fmt", "gbrpf32le", "-"])
        .stdout(Stdio::piped()).spawn().unwrap_or_else(|e| quitter(&format!("ffmpeg : {e}")));
    let mut lecteur = dec.stdout.take().unwrap();

    let (tx, rx) = std::sync::mpsc::sync_channel::<(usize, usize, Vec<u8>)>(2);
    let enc = {
        let (entree, sortie, fps) = (entree.clone(), sortie.clone(), fps.clone());
        std::thread::spawn(move || {
            let mut premier = rx.recv().ok();
            let Some((fw, fh, _)) = premier.as_ref().map(|p| (p.0, p.1, ())) else { return };
            if brut {
                let mut f = std::fs::File::create(&sortie).unwrap();
                while let Some((_, _, d)) = premier.take().or_else(|| rx.recv().ok()) { f.write_all(&d).unwrap() }
                return;
            }
            // Si la chaîne n'atteint pas exactement la taille demandée, ffmpeg finit en spline36 (comme libplacebo).
            let vf = format!("scale={ow}:{oh}:flags=spline:out_color_matrix=bt709:out_range=tv,format=yuv420p10le");
            let mut e = Command::new("ffmpeg")
                .args(["-hide_banner", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "gbrpf32le", "-s", &format!("{fw}x{fh}"), "-framerate", &fps, "-i", "-",
                    "-i", &entree, "-map", "0:v", "-map", "1:a?", "-map", "1:s?", "-map", "1:t?", "-map_metadata", "1", "-map_chapters", "1", "-vf", &vf])
                .args(encodage()).args(["-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-y", &sortie])
                .stdin(Stdio::piped()).spawn().unwrap();
            let mut stdin = e.stdin.take().unwrap();
            while let Some((_, _, d)) = premier.take().or_else(|| rx.recv().ok()) {
                if stdin.write_all(&d).is_err() { break }
            }
            drop(stdin);
            if !e.wait().map(|s| s.success()).unwrap_or(false) { quitter("échec de l'encodage") }
        })
    };

    let plan = w * h;
    let mut brute = vec![0u8; plan * 3 * 4];
    let mut n = 0usize;
    while lecteur.read_exact(&mut brute).is_ok() {
        let mut img = moteur::Tex::new(w, h);
        // gbrp : plans G, B, R -> canaux R, G, B, A=1
        moteur::par_lignes(&mut img, |y, ligne| {
            for (p, c) in [(0, 1), (1, 2), (2, 0)] {
                let src = &brute[(p * plan + y * w) * 4..][..w * 4];
                for (d, s) in ligne[c * w..][..w].iter_mut().zip(src.chunks_exact(4)) { *d = f32::from_le_bytes(s.try_into().unwrap()) }
            }
            ligne[3 * w..].fill(1.0);
        });
        let r = m.traiter(img, (ow, oh));
        let mut out = vec![0u8; r.w * r.h * 12];
        let lignes_par = r.h.div_ceil(std::thread::available_parallelism().map_or(1, |n| n.get()));
        std::thread::scope(|s| {
            for (plan_sortie, c) in out.chunks_mut(r.w * r.h * 4).zip([1, 2, 0]) {
                for (k, bloc) in plan_sortie.chunks_mut(lignes_par * r.w * 4).enumerate() {
                    let r = &r;
                    s.spawn(move || for (j, ligne) in bloc.chunks_mut(r.w * 4).enumerate() {
                        let src = &r.d[((k * lignes_par + j) * 4 + c) * r.w..][..r.w];
                        for (d, v) in ligne.chunks_exact_mut(4).zip(src) { d.copy_from_slice(&v.to_le_bytes()) }
                    });
                }
            }
        });
        if tx.send((r.w, r.h, out)).is_err() { break }
        n += 1;
        eprint!("\r{n} images · {:.2} img/s", n as f64 / t0.elapsed().as_secs_f64());
    }
    drop(tx);
    let _ = dec.wait();
    enc.join().unwrap();
    eprintln!();
    println!("{sortie} ({n} images en {:.1} s)", t0.elapsed().as_secs_f64());
}

fn encodage() -> [&'static str; 12] {
    ["-c:v", "libx265", "-preset", "medium", "-crf", "20", "-pix_fmt", "yuv420p10le", "-c:a", "copy", "-c:s", "copy"]
}

#[cfg(test)]
mod tests {
    #[test]
    fn presets_connus_et_analysables() {
        for p in ["A", "B", "C", "A+A", "B+B", "C+A"] {
            let src: Vec<String> = super::preset(p).unwrap().iter()
                .map(|s| std::fs::read_to_string(format!("{}/Anime4K_{s}.glsl", super::shaders())).expect("shader absent (ANIME4K_SHADERS ?)")).collect();
            super::moteur::Moteur::charger(&src); // panique sur toute ligne non reconnue
        }
        assert!(super::preset("Z").is_none());
    }
}
