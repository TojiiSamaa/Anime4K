//! Moteur CPU natif : interprète les shaders mpv d'Anime4K (bloc97, MIT) sans GPU.
//! Les poids sont lus dans les .glsl d'origine à l'exécution ; rien n'est recopié ici.
use std::collections::HashMap;

/// Texture 4 canaux, disposition [y][c][x] (une ligne = 4 plans contigus).
#[derive(Clone)]
pub struct Tex { pub w: usize, pub h: usize, pub d: Vec<f32> }

impl Tex {
    pub fn new(w: usize, h: usize) -> Self { Tex { w, h, d: vec![0.0; w * h * 4] } }
    #[inline] fn at(&self, c: usize, x: usize, y: usize) -> f32 { self.d[(y * 4 + c) * self.w + x] }
    #[inline] fn plan(&self, c: usize, y: usize) -> &[f32] { &self.d[(y * 4 + c) * self.w..][..self.w] }
}

struct Entree { tex: String, neg: bool, relu: bool }
struct Terme { i: usize, dx: i32, dy: i32, m: [f32; 16] }
struct Conv { entrees: Vec<Entree>, termes: Vec<Terme>, biais: [f32; 4], residu: bool }
enum Genre { Conv(Conv), D2S { src: Vec<String>, residu: bool }, StatsH, StatsV, Clamp, Copie }

struct Passe { desc: String, hook: String, save: Option<String>, w: Option<String>, h: Option<String>, when: Option<String>, genre: Genre }

pub struct Moteur { passes: Vec<Passe> }

fn nombres(s: &str) -> Vec<f32> {
    s.split(',').map(|n| n.trim().parse().unwrap_or_else(|_| panic!("nombre illisible : {n}"))).collect()
}
/// Contenu entre `debut(` et la parenthèse fermante correspondante.
fn entre<'a>(s: &'a str, debut: &str) -> &'a str {
    let i = s.find(debut).unwrap_or_else(|| panic!("{debut} absent de : {s}")) + debut.len();
    let mut prof = 1;
    for (k, ch) in s[i..].char_indices() {
        match ch { '(' => prof += 1, ')' => { prof -= 1; if prof == 0 { return &s[i..i + k] } } _ => {} }
    }
    panic!("parenthèse non fermée : {s}")
}

fn analyser_conv(desc: &str, corps: &[&str]) -> Conv {
    let mut c = Conv { entrees: vec![], termes: vec![], biais: [0.0; 4], residu: false };
    let mut defs: HashMap<usize, Entree> = HashMap::new();
    for l in corps.iter().map(|l| l.trim()) {
        if l.starts_with("#define g") {
            // #define go_N(x_off, y_off) (EXPR)  |  #define g_N (EXPR)
            let n: usize = l.split(['_', '(', ' ']).nth(2).and_then(|n| n.parse().ok()).unwrap_or_else(|| panic!("{desc} : {l}"));
            let expr = &l[l.find(") (").map_or_else(|| l.find(" (").unwrap() + 1, |p| p + 2)..];
            let p = expr.find("_texOff(").or_else(|| expr.find("_tex(")).unwrap_or_else(|| panic!("{desc} : {l}"));
            let tex = expr[..p].rsplit('(').next().unwrap().trim_start_matches('-').to_string();
            defs.insert(n, Entree { tex, neg: expr.contains("max(-("), relu: expr.starts_with("(max(") || expr.starts_with("max(") });
        } else if l.contains("mat4(") {
            let m = nombres(entre(l, "mat4("));
            assert_eq!(m.len(), 16, "{desc} : mat4 incomplète");
            let apres = &l[l.rfind(") * ").unwrap_or_else(|| panic!("{desc} : {l}")) + 4..];
            let (i, dx, dy) = if let Some(r) = apres.strip_prefix("go_") {
                let n: usize = r[..r.find('(').unwrap()].parse().unwrap();
                let o = nombres(entre(r, "("));
                (n, o[0] as i32, o[1] as i32)
            } else if let Some(r) = apres.strip_prefix("g_") {
                (r.trim_end_matches(';').parse().unwrap_or_else(|_| panic!("{desc} : {l}")), 0, 0)
            } else { panic!("{desc} : terme inconnu {l}") };
            c.termes.push(Terme { i, dx, dy, m: m.try_into().unwrap() });
        } else if l.starts_with("result += vec4(") {
            let b = nombres(entre(l, "vec4("));
            for k in 0..4 { c.biais[k] += b[k] }
        } else if l == "return result + MAIN_tex(MAIN_pos);" { c.residu = true }
        else if !(l.is_empty() || l == "return result;" || l == "vec4 hook() {" || l == "}" || l.starts_with("//")) {
            panic!("{desc} : ligne non reconnue « {l} »")
        }
    }
    let n = defs.len();
    c.entrees = (0..n).map(|k| defs.remove(&k).unwrap_or_else(|| panic!("{desc} : entrée {k} manquante"))).collect();
    c
}

impl Moteur {
    pub fn charger(sources: &[String]) -> Moteur {
        let mut passes = vec![];
        for src in sources {
            let lignes: Vec<&str> = src.lines().collect();
            let debuts: Vec<usize> = (0..lignes.len()).filter(|&i| lignes[i].starts_with("//!DESC")).collect();
            for (k, &d) in debuts.iter().enumerate() {
                let fin = debuts.get(k + 1).copied().unwrap_or(lignes.len());
                let bloc = &lignes[d..fin];
                let tete = |cle: &str| bloc.iter().find_map(|l| l.strip_prefix(cle)).map(|v| v.trim().to_string());
                let binds: Vec<String> = bloc.iter().filter_map(|l| l.strip_prefix("//!BIND ")).map(|s| s.trim().to_string()).collect();
                let corps: Vec<&str> = bloc.iter().copied().filter(|l| !l.starts_with("//!")).collect();
                let texte = corps.join("\n");
                let desc = tete("//!DESC ").unwrap();
                let genre = if desc.contains("Depth-to-Space") {
                    Genre::D2S { src: binds.iter().filter(|b| *b != "MAIN").cloned().collect(), residu: texte.contains("+ MAIN_tex(MAIN_pos)") }
                } else if texte.contains("new_luma") { Genre::Clamp }
                else if texte.contains("MAIN_texOff(vec2(i - KERNELHALFSIZE, 0))") { Genre::StatsH }
                else if texte.contains("STATSMAX_texOff(vec2(0, i - KERNELHALFSIZE))") { Genre::StatsV }
                else if texte.contains("return HOOKED_tex(HOOKED_pos);") && !texte.contains("mat4") { Genre::Copie }
                else { Genre::Conv(analyser_conv(&desc, &corps)) };
                passes.push(Passe { hook: tete("//!HOOK ").unwrap(), save: tete("//!SAVE "), w: tete("//!WIDTH "), h: tete("//!HEIGHT "), when: tete("//!WHEN "), desc, genre });
            }
        }
        Moteur { passes }
    }

    /// Applique la chaîne. `main` = RGB dans les canaux 0..3 ; renvoie l'image finale (avant mise à l'échelle de sortie).
    pub fn traiter(&self, main: Tex, sortie: (usize, usize)) -> Tex {
        let natif = (main.w, main.h);
        let mut t: HashMap<String, Tex> = HashMap::new();
        t.insert("MAIN".into(), main);
        for point in ["MAIN", "PREKERNEL"] {
            for p in self.passes.iter().filter(|p| p.hook == point) {
                let taille = |nom: &str, t: &HashMap<String, Tex>| -> (usize, usize) {
                    match nom { "NATIVE" => natif, "OUTPUT" => sortie, "HOOKED" => { let m = &t["MAIN"]; (m.w, m.h) }
                        n => { let m = t.get(n).unwrap_or_else(|| panic!("{} : texture {n} inconnue", p.desc)); (m.w, m.h) } }
                };
                let rpn = |e: &str| rpn(e, |n, axe| { let s = taille(n, &t); (if axe == 'w' { s.0 } else { s.1 }) as f64 });
                if let Some(w) = &p.when { if rpn(w) == 0.0 { continue } }
                let chrono = std::time::Instant::now();
                let hooked = &t["MAIN"];
                let (w, h) =(p.w.as_deref().map_or(hooked.w, |e| rpn(e) as usize), p.h.as_deref().map_or(hooked.h, |e| rpn(e) as usize));
                let res = match &p.genre {
                    Genre::Conv(c) => conv(c, &t, w, h),
                    Genre::D2S { src, residu } => d2s(src, *residu, &t, w, h),
                    Genre::StatsH => stats(hooked, true),
                    Genre::StatsV => stats(&t["STATSMAX"], false),
                    Genre::Clamp => clamp(hooked, &t["STATSMAX"]),
                    Genre::Copie => reechantillonner(hooked, w, h),
                };
                if std::env::var_os("A4K_PROFIL").is_some() { eprintln!("{:>8.1} ms  {}x{}  {}", chrono.elapsed().as_secs_f64() * 1e3, w, h, p.desc) }
                t.insert(p.save.clone().unwrap_or_else(|| "MAIN".into()), res);
            }
        }
        t.remove("MAIN").unwrap()
    }
}

/// Expressions des en-têtes mpv (notation polonaise inversée : `OUTPUT.w MAIN.w / 1.2 >`).
fn rpn(e: &str, taille: impl Fn(&str, char) -> f64) -> f64 {
    let mut p: Vec<f64> = vec![];
    for tok in e.split_whitespace() {
        let v = match tok {
            "+" | "-" | "*" | "/" | ">" | "<" | "=" => {
                let (b, a) = (p.pop().unwrap(), p.pop().unwrap());
                match tok { "+" => a + b, "-" => a - b, "*" => a * b, "/" => a / b, ">" => (a > b) as u8 as f64, "<" => (a < b) as u8 as f64, _ => (a == b) as u8 as f64 }
            }
            "!" => (p.pop().unwrap() == 0.0) as u8 as f64,
            t => t.parse().unwrap_or_else(|_| {
                let (nom, axe) = t.rsplit_once('.').unwrap_or_else(|| panic!("jeton RPN inconnu : {t}"));
                taille(nom, axe.chars().next().unwrap())
            }),
        };
        p.push(v);
    }
    p.pop().unwrap()
}

/// Répartit les lignes de `out` entre les cœurs par paquets pris à la demande (vol de travail
/// façon rayon, std seulement) : un cœur ralenti par un voisin ne retarde plus toute la passe.
pub fn par_lignes(out: &mut Tex, f: impl Fn(usize, &mut [f32]) + Sync) {
    const PAQUET: usize = 8;
    let pas = out.w * 4;
    let n = std::thread::available_parallelism().map_or(1, |n| n.get());
    let file = std::sync::Mutex::new(out.d.chunks_mut(PAQUET * pas).enumerate());
    std::thread::scope(|s| {
        for _ in 0..n.min(out.h.div_ceil(PAQUET)) {
            s.spawn(|| loop {
                let Some((k, bloc)) = file.lock().unwrap().next() else { break };
                for (j, ligne) in bloc.chunks_mut(pas).enumerate() { f(k * PAQUET + j, ligne) }
            });
        }
    });
}

/// Pixels calculés par itération du noyau (3 vecteurs AVX2 de 8).
const BLOC: usize = 24;
const TUILE: usize = BLOC * 10;
const LARG: usize = TUILE + 2;

fn conv(c: &Conv, t: &HashMap<String, Tex>, w: usize, h: usize) -> Tex {
    let ins: Vec<(&Tex, &Entree)> = c.entrees.iter().map(|e| (&t[&e.tex], e)).collect();
    for (x, e) in &ins { assert!(x.w == w && x.h == h, "convolution sur {} de taille différente", e.tex) }
    // Seules les lignes (entrée, dy) réellement lues sont copiées : une seule pour les 1x1.
    let mut emplacement = vec![usize::MAX; ins.len() * 3];
    let mut lues = vec![];
    for tm in &c.termes {
        let k = tm.i * 3 + (tm.dy + 1) as usize;
        if emplacement[k] == usize::MAX { emplacement[k] = lues.len(); lues.push((tm.i, (tm.dy + 1) as usize)) }
    }
    // Termes précompilés : (décalage du plan canal 0 dans le tampon + dx, poids).
    let termes: Vec<(usize, &[f32; 16])> = c.termes.iter()
        .map(|tm| ((emplacement[tm.i * 3 + (tm.dy + 1) as usize] * 4 * LARG) + (1 + tm.dx) as usize, &tm.m)).collect();
    let main = c.residu.then(|| { let m = &t["MAIN"]; (m, Taps::new(m.w, w)) });
    let mut out = Tex::new(w, h);
    par_lignes(&mut out, |y, ligne| {
        let mut tampon = vec![0f32; lues.len() * 4 * LARG + BLOC];
        let mut acc = [0f32; 4 * TUILE];
        let mut x0 = 0;
        while x0 < w {
            let l = TUILE.min(w - x0);
            for (s, &(i, dy)) in lues.iter().enumerate() {
                let (tex, e) = ins[i];
                let yy = (y as isize + dy as isize - 1).clamp(0, h as isize - 1) as usize;
                for ch in 0..4 {
                    let src = tex.plan(ch, yy);
                    let dst = &mut tampon[(s * 4 + ch) * LARG..][..LARG];
                    // bords répliqués : x0-1 à gauche, dernier pixel à droite
                    dst[0] = src[x0.saturating_sub(1)];
                    let n = (LARG - 1).min(w - x0);
                    dst[1..1 + n].copy_from_slice(&src[x0..x0 + n]);
                    dst[1 + n..].fill(src[w - 1]);
                    // un seul passage signe + ReLU, une boucle par cas pour rester vectorisé
                    match (e.neg, e.relu) {
                        (false, false) => {}
                        (false, true) => dst.iter_mut().for_each(|v| *v = v.max(0.0)),
                        (true, false) => dst.iter_mut().for_each(|v| *v = -*v),
                        (true, true) => dst.iter_mut().for_each(|v| *v = (-*v).max(0.0)),
                    }
                }
            }
            noyau(&termes, &c.biais, &tampon, &mut acc);
            for r in 0..4 { ligne[r * w + x0..][..l].copy_from_slice(&acc[r * TUILE..][..l]) }
            x0 += TUILE;
        }
        if let Some((m, tx)) = &main { for r in 0..4 { bil_ligne(m, r, y, h, tx, &mut ligne[r * w..][..w], true) } }
    });
    out
}

/// acc[r][x] = biais[r] + Σ termes Σ canaux m[ch*4+r] * tampon[ch][x+décalage], sur une tuile.
fn noyau(termes: &[(usize, &[f32; 16])], biais: &[f32; 4], tampon: &[f32], acc: &mut [f32; 4 * TUILE]) {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx512f") {
        return unsafe { noyau_avx512(termes, biais, tampon, acc) };
    }
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        return unsafe { noyau_avx2(termes, biais, tampon, acc) };
    }
    for r in 0..4 { acc[r * TUILE..][..TUILE].fill(biais[r]) }
    for &(o, m) in termes {
        for ch in 0..4 {
            let v = &tampon[o + ch * LARG..][..TUILE];
            for r in 0..4 {
                let wgt = m[ch * 4 + r];
                for (a, x) in acc[r * TUILE..][..TUILE].iter_mut().zip(v) { *a = wgt.mul_add(*x, *a) }
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2,fma")]
unsafe fn noyau_avx2(termes: &[(usize, &[f32; 16])], biais: &[f32; 4], tampon: &[f32], acc: &mut [f32; 4 * TUILE]) {
    use std::arch::x86_64::*;
    let p = tampon.as_ptr();
    for xs in (0..TUILE).step_by(BLOC) {
        let mut a = [[_mm256_setzero_ps(); 3]; 4];
        for r in 0..4 { for j in 0..3 { a[r][j] = _mm256_set1_ps(biais[r]) } }
        for &(o, m) in termes {
            for ch in 0..4 {
                // SAFETY : o + 3*LARG + TUILE + BLOC <= tampon.len() (marge BLOC allouée).
                let q = unsafe { p.add(o + ch * LARG + xs) };
                let v = unsafe { [_mm256_loadu_ps(q), _mm256_loadu_ps(q.add(8)), _mm256_loadu_ps(q.add(16))] };
                for r in 0..4 {
                    let wgt = _mm256_set1_ps(m[ch * 4 + r]);
                    for j in 0..3 { a[r][j] = _mm256_fmadd_ps(wgt, v[j], a[r][j]) }
                }
            }
        }
        for r in 0..4 { for j in 0..3 { unsafe { _mm256_storeu_ps(acc.as_mut_ptr().add(r * TUILE + xs + j * 8), a[r][j]) } } }
    }
}

/// Même calcul que `noyau_avx2`, 48 pixels par itération (3 vecteurs de 16), même ordre des FMA : sortie identique au bit près.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn noyau_avx512(termes: &[(usize, &[f32; 16])], biais: &[f32; 4], tampon: &[f32], acc: &mut [f32; 4 * TUILE]) {
    use std::arch::x86_64::*;
    const B: usize = 48;
    const _: () = assert!(TUILE % B == 0);
    let p = tampon.as_ptr();
    for xs in (0..TUILE).step_by(B) {
        let mut a = [[_mm512_setzero_ps(); 3]; 4];
        for r in 0..4 { for j in 0..3 { a[r][j] = _mm512_set1_ps(biais[r]) } }
        for &(o, m) in termes {
            for ch in 0..4 {
                // SAFETY : o + 3*LARG + TUILE + BLOC <= tampon.len() (marge BLOC allouée), xs + 48 <= TUILE.
                let q = unsafe { p.add(o + ch * LARG + xs) };
                let v = unsafe { [_mm512_loadu_ps(q), _mm512_loadu_ps(q.add(16)), _mm512_loadu_ps(q.add(32))] };
                for r in 0..4 {
                    let wgt = _mm512_set1_ps(m[ch * 4 + r]);
                    for j in 0..3 { a[r][j] = _mm512_fmadd_ps(wgt, v[j], a[r][j]) }
                }
            }
        }
        for r in 0..4 { for j in 0..3 { unsafe { _mm512_storeu_ps(acc.as_mut_ptr().add(r * TUILE + xs + j * 16), a[r][j]) } } }
    }
}

/// Coefficients bilinéaires façon GPU (centres de texels, bords répliqués) d'un axe src -> dst.
struct Taps { i0: Vec<usize>, i1: Vec<usize>, f: Vec<f32> }
impl Taps {
    fn new(src: usize, dst: usize) -> Taps {
        let mut t = Taps { i0: vec![], i1: vec![], f: vec![] };
        for x in 0..dst {
            let u = ((x as f32 + 0.5) * src as f32 / dst as f32 - 0.5).max(0.0);
            let i0 = (u as usize).min(src - 1);
            t.i0.push(i0);
            t.i1.push((i0 + 1).min(src - 1));
            t.f.push(u - i0 as f32);
        }
        t
    }
}

/// Ligne y (sur une hauteur h) du canal c de t rééchantillonnée en bilinéaire ; ajoutée à dst ou écrite.
fn bil_ligne(t: &Tex, c: usize, y: usize, h: usize, tx: &Taps, dst: &mut [f32], ajouter: bool) {
    if t.w == dst.len() && t.h == h {
        let src = t.plan(c, y);
        if ajouter { dst.iter_mut().zip(src).for_each(|(d, s)| *d += s) } else { dst.copy_from_slice(src) }
        return;
    }
    let v = ((y as f32 + 0.5) * t.h as f32 / h as f32 - 0.5).max(0.0);
    let y0 = (v as usize).min(t.h - 1);
    let (r0, r1, fv) = (t.plan(c, y0), t.plan(c, (y0 + 1).min(t.h - 1)), v - y0 as f32);
    for (x, d) in dst.iter_mut().enumerate() {
        let (a, b, fu) = (tx.i0[x], tx.i1[x], tx.f[x]);
        let l0 = r0[a] + (r0[b] - r0[a]) * fu;
        let l1 = r1[a] + (r1[b] - r1[a]) * fu;
        let s = l0 + (l1 - l0) * fv;
        if ajouter { *d += s } else { *d = s }
    }
}

fn d2s(src: &[String], residu: bool, t: &HashMap<String, Tex>, w: usize, h: usize) -> Tex {
    let s: Vec<&Tex> = src.iter().map(|n| &t[n]).collect();
    let main = &t["MAIN"];
    let tx = Taps::new(main.w, w);
    let mut out = Tex::new(w, h);
    par_lignes(&mut out, |y, ligne| {
        for c in 0..4 {
            let plan = s[c.min(s.len() - 1)];
            let (k0, ys) = ((y % 2) * 2, y / 2);
            let (p0, p1) = (plan.plan(k0, ys), plan.plan(k0 + 1, ys));
            let dst = &mut ligne[c * w..][..w];
            for (x, d) in dst.iter_mut().enumerate() { *d = if x % 2 == 0 { p0[x / 2] } else { p1[x / 2] } }
            if residu { bil_ligne(main, c, y, h, &tx, dst, true) }
        }
    });
    out
}

const LUMA: [f32; 3] = [0.299, 0.587, 0.114];

fn stats(src: &Tex, horizontal: bool) -> Tex {
    let (w, h) = (src.w, src.h);
    let mut out = Tex::new(w, h);
    par_lignes(&mut out, |y, ligne| {
        for x in 0..w {
            let mut g = 0f32;
            for i in -2isize..=2 {
                g = g.max(if horizontal {
                    let xx = (x as isize + i).clamp(0, w as isize - 1) as usize;
                    (0..3).map(|c| LUMA[c] * src.at(c, xx, y)).sum()
                } else { src.at(0, x, (y as isize + i).clamp(0, h as isize - 1) as usize) });
            }
            ligne[x] = g;
        }
    });
    out
}

fn clamp(img: &Tex, stat: &Tex) -> Tex {
    let (w, h) = (img.w, img.h);
    let tx = Taps::new(stat.w, w);
    let mut out = Tex::new(w, h);
    par_lignes(&mut out, |y, ligne| {
        let mut s = vec![0f32; w];
        bil_ligne(stat, 0, y, h, &tx, &mut s, false);
        let (r, g, b) = (img.plan(0, y), img.plan(1, y), img.plan(2, y));
        for x in 0..w {
            let luma = LUMA[0] * r[x] + LUMA[1] * g[x] + LUMA[2] * b[x];
            s[x] = luma - luma.min(s[x]);
        }
        for c in 0..4 { for (x, d) in ligne[c * w..][..w].iter_mut().enumerate() { *d = img.at(c, x, y) - s[x] } }
    });
    out
}

pub fn reechantillonner(src: &Tex, w: usize, h: usize) -> Tex {
    let tx = Taps::new(src.w, w);
    let mut out = Tex::new(w, h);
    par_lignes(&mut out, |y, ligne| for c in 0..4 { bil_ligne(src, c, y, h, &tx, &mut ligne[c * w..][..w], false) });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rpn_entetes_mpv() {
        let t = |n: &str, a: char| match (n, a) { ("OUTPUT", 'w') => 3840.0, ("MAIN", 'w') => 1920.0, (_, _) => 1080.0 };
        assert_eq!(rpn("OUTPUT.w MAIN.w / 1.200 >", t), 1.0);
        assert_eq!(rpn("MAIN.w 2 *", t), 3840.0);
        assert_eq!(rpn("OUTPUT.w MAIN.w / 2.0 < !", t), 1.0);
    }
    #[test]
    fn conv_identite_et_relu() {
        // 1 entrée négative activée, matrice identité, biais 0.5 : sortie = max(-x,0) + 0.5
        let src = "//!DESC t\n//!HOOK MAIN\n//!BIND MAIN\n//!SAVE MAIN\n#define go_0(x_off, y_off) (max(-(MAIN_texOff(vec2(x_off, y_off))), 0.0))\nvec4 hook() {\n    vec4 result = mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0) * go_0(0.0, 0.0);\n    result += vec4(0.5, 0.5, 0.5, 0.5);\n    return result;\n}\n";
        let m = Moteur::charger(&[src.to_string()]);
        let mut img = Tex::new(3, 2);
        img.d.iter_mut().enumerate().for_each(|(i, v)| *v = if i % 2 == 0 { -1.0 } else { 1.0 });
        let r = m.traiter(img.clone(), (3, 2));
        for (a, b) in img.d.iter().zip(&r.d) { assert_eq!(*b, (-a).max(0.0) + 0.5) }
    }
}
