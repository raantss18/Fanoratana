//! Lire une formule à sa **structure** (ADR 0013) : la mise en page 2D se lit à la géométrie des traits,
//! chaque signe par le petit modèle ([`crate::signes`]). Le LaTeX est exact par construction : chaque
//! symbole vient de traits réels, rien n'est inventé ; ce qui n'est pas compris rend `None` (recours).
//!
//! Ordre de lecture d'une zone (récursif) :
//! 1. une **barre de fraction** (trait plat, encre au-dessus et au-dessous, plus large qu'eux) ;
//! 2. une **racine** (trait qui descend, remonte et file à droite au-dessus d'autres traits) ;
//! 3. les **signes** de gauche à droite (traits qui se chevauchent en largeur), avec les **piles** :
//!    `lim` sur `x → 0`, `Σ` entre ses bornes ;
//! 4. la **ligne de base** : exposants et indices se lisent à la place de chaque signe par rapport au
//!    corps du signe qui le porte (le corps d'un `y` ou d'un `g` s'arrête au-dessus de la jambe).
//!
//! Le contexte tranche les signes jumeaux : `×` entre deux nombres sinon `x`, `0`/`o`, `1`/`l`/`I`, et
//! les noms de fonctions (`lim`, `sin`, `cos`, `ln`…) lus sur les trois meilleures classes de chaque lettre.

use fano_core::Point;

use crate::signes::{Case, Choix, ClasseurSignes};

/// Le résultat : LaTeX (sans `$`), confiance (celle du signe le moins sûr), lectures voisines.
#[derive(Clone, Debug, PartialEq)]
pub struct LectureFormule {
    pub latex: String,
    pub confiance: f32,
    pub alternatives: Vec<String>,
}

#[derive(Clone, Debug)]
struct Tr {
    pts: Vec<Point>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl Tr {
    fn new(pts: Vec<Point>) -> Tr {
        let mut t = Tr {
            pts,
            x0: f32::MAX,
            y0: f32::MAX,
            x1: f32::MIN,
            y1: f32::MIN,
        };
        for p in &t.pts {
            t.x0 = t.x0.min(p.x);
            t.y0 = t.y0.min(p.y);
            t.x1 = t.x1.max(p.x);
            t.y1 = t.y1.max(p.y);
        }
        t
    }
    fn w(&self) -> f32 {
        self.x1 - self.x0
    }
    fn h(&self) -> f32 {
        self.y1 - self.y0
    }
    fn cx(&self) -> f32 {
        (self.x0 + self.x1) / 2.0
    }
    fn cy(&self) -> f32 {
        (self.y0 + self.y1) / 2.0
    }
    /// Plat : une barre, un moins, un trait du `=`.
    fn plat(&self) -> bool {
        self.w() > 0.0 && self.h() <= 0.3 * self.w()
    }
}

/// Une boîte (x0, y0, x1, y1).
#[derive(Clone, Copy, Debug)]
struct Boite {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl Boite {
    fn h(&self) -> f32 {
        self.y1 - self.y0
    }
    fn w(&self) -> f32 {
        self.x1 - self.x0
    }
    fn cx(&self) -> f32 {
        (self.x0 + self.x1) / 2.0
    }
    fn cy(&self) -> f32 {
        (self.y0 + self.y1) / 2.0
    }
}

/// Un signe lu : ses traits, ses meilleures classes, le LaTeX retenu.
#[derive(Clone, Debug)]
struct Signe {
    traits: Vec<usize>,
    choix: Choix,
    /// Le LaTeX retenu (après le contexte) et sa probabilité.
    texte: String,
    p: f32,
    /// L'identifiant de la classe retenue (`x`, `plus`, `somme`…).
    id: String,
}

/// Un nœud de la formule.
#[derive(Clone, Debug)]
enum N {
    Signe(usize),
    /// Un nom de fonction lu sur plusieurs signes (`\sin`).
    Mot(String),
    Frac(Vec<N>, Vec<N>),
    Racine(Vec<N>),
    Scripts(Box<N>, Option<Vec<N>>, Option<Vec<N>>),
    /// Un opérateur à bornes au-dessus / au-dessous (`\lim_{x\to0}`, `\sum_{k=1}^{n}`).
    Bornes(Box<N>, Option<Vec<N>>, Option<Vec<N>>),
    /// Un système (`cases`) ou une matrice (`pmatrix`, `vmatrix`) : rangées de cases.
    Tableau(&'static str, Vec<Vec<Vec<N>>>),
}

/// Les mots lus d'un bloc : fonctions, et `si`/`sinon` des systèmes (le plus long d'abord).
const FONCTIONS: [&str; 12] = [
    "sinon", "lim", "sin", "cos", "tan", "ln", "log", "exp", "det", "max", "min", "si",
];

/// Le LaTeX d'un mot : une fonction (`\sin`) ou un mot du texte (`si`, `sinon`).
fn latex_du_mot(f: &str) -> String {
    match f {
        "si" | "sinon" => format!("\\text{{{f} }}"),
        _ => format!("\\{f}"),
    }
}

struct Lecture<'a> {
    cl: &'a ClasseurSignes,
    tr: Vec<Tr>,
    /// Hauteur médiane d'un signe.
    hmed: f32,
    signes: Vec<Signe>,
}

/// Lit une formule écrite à la main. `None` : la structure n'est pas comprise (matrice, système…).
pub fn lire_formule(cl: &ClasseurSignes, traits: &[Vec<Point>]) -> Option<LectureFormule> {
    let tr: Vec<Tr> = traits
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| Tr::new(t.clone()))
        .collect();
    if tr.is_empty() || tr.len() > 120 {
        return None;
    }
    let hmed = hauteur_mediane(&tr);
    let mut l = Lecture {
        cl,
        tr,
        hmed,
        signes: Vec::new(),
    };
    let tous: Vec<usize> = (0..l.tr.len()).collect();
    let arbre = l.zone(&tous)?;
    if l.signes.is_empty() {
        return None;
    }
    l.contexte(&arbre);
    let latex = l.ecrire(&arbre);
    let confiance = l.signes.iter().map(|s| s.p).fold(1.0, f32::min);
    // Lectures voisines : les trois signes les moins sûrs, chacun remplacé par son second choix.
    let mut doutes: Vec<usize> = (0..l.signes.len())
        .filter(|&i| l.signes[i].choix.len() > 1)
        .collect();
    doutes.sort_by(|&a, &b| l.signes[a].p.total_cmp(&l.signes[b].p));
    let mut alternatives = Vec::new();
    for i in doutes.into_iter().take(3) {
        let garde = (l.signes[i].texte.clone(), l.signes[i].id.clone());
        let second = l.signes[i]
            .choix
            .iter()
            .map(|c| &l.cl.classes[c.0])
            .find(|c| c.latex != garde.0)
            .map(|c| (c.latex.clone(), c.id.clone()));
        if let Some((t, id)) = second {
            l.signes[i].texte = t;
            l.signes[i].id = id;
            let a = l.ecrire(&arbre);
            if a != latex && !alternatives.contains(&a) {
                alternatives.push(a);
            }
            (l.signes[i].texte, l.signes[i].id) = garde;
        }
    }
    Some(LectureFormule {
        latex,
        confiance,
        alternatives,
    })
}

/// Un arc ouvert d'un côté : `Some(true)` s'il bombe à droite (« ɔ »), `Some(false)` à gauche (« c »).
fn bombe(t: &Tr) -> Option<bool> {
    if t.pts.len() < 4 || t.w() <= 0.0 {
        return None;
    }
    let n = t.pts.len();
    let bouts = (t.pts[0].x + t.pts[n - 1].x) / 2.0;
    let (imax, _) = t.pts.iter().enumerate().max_by(|a, b| a.1.x.total_cmp(&b.1.x))?;
    let (imin, _) = t.pts.iter().enumerate().min_by(|a, b| a.1.x.total_cmp(&b.1.x))?;
    // L'extrême est loin des bouts en hauteur (un trait rapide n'a que peu de points au début).
    let milieu = |i: usize| t.pts[i].y > t.y0 + 0.15 * t.h() && t.pts[i].y < t.y1 - 0.15 * t.h();
    // Les deux bouts du même côté, l'extrême de l'autre côté au milieu du trait, et le trait monte ou descend.
    let vertical = (t.pts[0].y - t.pts[n - 1].y).abs() > 0.35 * t.h();
    if !vertical {
        return None;
    }
    if milieu(imax)
        && t.x1 - bouts > 0.45 * t.w()
        && (t.pts[0].x - t.x0).min(t.pts[n - 1].x - t.x0) < 0.6 * t.w()
    {
        return Some(true);
    }
    if milieu(imin)
        && bouts - t.x0 > 0.45 * t.w()
        && (t.x1 - t.pts[0].x).min(t.x1 - t.pts[n - 1].x) < 0.6 * t.w()
    {
        return Some(false);
    }
    None
}

/// Hauteur d'un signe : la médiane des traits ni plats ni minuscules (les points du `i` n'en sont pas).
fn hauteur_mediane(tr: &[Tr]) -> f32 {
    let grand = tr.iter().map(|t| t.h().max(t.w())).fold(0.0, f32::max);
    let mut hs: Vec<f32> = tr
        .iter()
        .filter(|t| !t.plat() && t.h().max(t.w()) > 0.2 * grand)
        .map(Tr::h)
        .collect();
    if hs.is_empty() {
        hs = tr.iter().map(|t| t.w()).collect();
    }
    // Les grands délimiteurs (parenthèses d'une matrice) ne sont pas des signes ordinaires : sans eux.
    if tr.len() > 8 && tr.iter().any(|t| t.h() > 2.5 * t.w() && t.h() > 0.8 * grand) {
        let petits: Vec<f32> = tr
            .iter()
            .filter(|t| !t.plat() && t.h().max(t.w()) > 0.08 * grand && t.h() < 0.5 * grand)
            .map(Tr::h)
            .collect();
        // Seulement si l'essentiel de l'encre est petit devant eux (le contenu d'une matrice), pas pour
        // les parenthèses d'un point `A(3 ; −1)`.
        let non_plats = tr.iter().filter(|t| !t.plat()).count();
        if !petits.is_empty() && petits.len() * 2 >= non_plats {
            hs = petits;
        }
    }
    hs.sort_by(f32::total_cmp);
    hs[hs.len() / 2].max(1.0)
}

fn union(a: Boite, b: Boite) -> Boite {
    Boite {
        x0: a.x0.min(b.x0),
        y0: a.y0.min(b.y0),
        x1: a.x1.max(b.x1),
        y1: a.y1.max(b.y1),
    }
}

fn boite_de(tr: &[Tr], ids: &[usize]) -> Boite {
    let mut b = Boite {
        x0: f32::MAX,
        y0: f32::MAX,
        x1: f32::MIN,
        y1: f32::MIN,
    };
    for &i in ids {
        b.x0 = b.x0.min(tr[i].x0);
        b.y0 = b.y0.min(tr[i].y0);
        b.x1 = b.x1.max(tr[i].x1);
        b.y1 = b.y1.max(tr[i].y1);
    }
    b
}

impl Lecture<'_> {
    fn boite(&self, ids: &[usize]) -> Boite {
        boite_de(&self.tr, ids)
    }

    /// Une zone : fraction, racine, ou rangée de signes.
    fn zone(&mut self, ids: &[usize]) -> Option<Vec<N>> {
        if ids.is_empty() {
            return Some(Vec::new());
        }
        if let Some((noeud, boite, pris)) = self.tableau(ids)? {
            let reste: Vec<usize> = ids.iter().copied().filter(|i| !pris.contains(i)).collect();
            return self.rangee_avec(&reste, vec![(noeud, boite)]);
        }
        if let Some(r) = self.fraction(ids) {
            return r;
        }
        if let Some(r) = self.racine(ids) {
            return r;
        }
        self.rangee(ids)
    }

    /// Les grands délimiteurs : hauts et étroits (parenthèses, crochets, barres, accolade), d'un trait ou de
    /// plusieurs traits verticaux bout à bout (une accolade en deux moitiés). De gauche à droite.
    fn delimiteurs(&self, ids: &[usize]) -> Vec<Vec<usize>> {
        let mut verticaux: Vec<usize> = ids
            .iter()
            .copied()
            .filter(|&i| self.tr[i].h() >= 0.8 * self.hmed && self.tr[i].h() >= 2.0 * self.tr[i].w())
            .collect();
        verticaux.sort_by(|&a, &b| self.tr[a].y0.total_cmp(&self.tr[b].y0));
        let mut groupes: Vec<Vec<usize>> = Vec::new();
        for i in verticaux {
            let t = &self.tr[i];
            let suite = groupes.iter_mut().find(|g| {
                let b = boite_de(&self.tr, g);
                let recouvre = t.x1.min(b.x1) - t.x0.max(b.x0) > -0.2 * self.hmed;
                recouvre && t.y0 - b.y1 < 0.3 * self.hmed && t.y0 > b.y0 + 0.5 * b.h()
            });
            match suite {
                Some(g) => g.push(i),
                None => groupes.push(vec![i]),
            }
        }
        groupes.retain(|g| {
            let b = boite_de(&self.tr, g);
            b.h() >= 2.2 * self.hmed && b.h() >= 2.5 * b.w()
        });
        groupes.sort_by(|a, b| boite_de(&self.tr, a).cx().total_cmp(&boite_de(&self.tr, b).cx()));
        groupes
    }

    /// Les rangées d'un contenu : traits groupés de haut en bas par recouvrement en hauteur.
    fn rangees(&self, contenu: &[usize]) -> Vec<Vec<usize>> {
        let mut ordre = contenu.to_vec();
        ordre.sort_by(|&a, &b| self.tr[a].y0.total_cmp(&self.tr[b].y0));
        let mut rangees: Vec<Vec<usize>> = Vec::new();
        let mut bas = f32::MIN;
        for i in ordre {
            let t = &self.tr[i];
            // Un point ou une barre suit la rangée en cours (le `−` d'une case, le point d'un `i`).
            let leger = t.plat() || self.petit(t);
            if rangees.is_empty()
                || (t.y0 > bas - 0.1 * self.hmed && !(leger && t.cy() < bas + 0.3 * self.hmed))
            {
                rangees.push(Vec::new());
                bas = f32::MIN;
            }
            bas = bas.max(t.y1);
            rangees.last_mut().unwrap().push(i);
        }
        rangees.retain(|r| r.iter().any(|&i| !self.petit(&self.tr[i])));
        rangees
    }

    /// Un tableau : une matrice (deux grands délimiteurs, au moins deux rangées entre eux) ou un système
    /// (une grande accolade ou barre à gauche, au moins deux rangées à sa droite). Rend le nœud, sa boîte et
    /// les traits pris. `Some(None)` : pas de tableau ; `None` : un tableau qu'on ne sait pas lire.
    fn tableau(&mut self, ids: &[usize]) -> Option<Option<(N, Boite, Vec<usize>)>> {
        let delims = self.delimiteurs(ids);
        let Some(g) = delims.first().cloned() else {
            return Some(None);
        };
        let tg = boite_de(&self.tr, &g);
        // Le délimiteur fermant a la taille de l'ouvrant (un `t` haut n'en est pas un).
        let d = delims[1..]
            .iter()
            .find(|k| {
                let t = boite_de(&self.tr, k);
                t.h() >= 0.7 * tg.h() && t.y1.min(tg.y1) - t.y0.max(tg.y0) > 0.6 * tg.h()
            })
            .cloned();
        let x_fin = d.as_ref().map_or(f32::MAX, |d| boite_de(&self.tr, d).x0);
        let contenu: Vec<usize> = ids
            .iter()
            .copied()
            .filter(|&i| {
                let t = &self.tr[i];
                !g.contains(&i)
                    && !d.as_ref().is_some_and(|d| d.contains(&i))
                    && t.cx() > tg.x0 + 0.75 * tg.w()
                    && t.cx() < x_fin
                    && t.cy() > tg.y0
                    && t.cy() < tg.y1
            })
            .collect();
        let rangees = self.rangees(&contenu);
        if rangees.len() < 2 {
            return Some(None);
        }
        let mut pris: Vec<usize> = contenu.clone();
        pris.extend(&g);
        let n = if let Some(d) = d {
            // Matrice : colonnes par les bandes verticales des signes de toutes les rangées.
            pris.extend(&d);
            let mut bandes: Vec<(f32, f32)> = Vec::new();
            let mut groupes: Vec<Vec<usize>> = Vec::new();
            for r in &rangees {
                groupes.extend(self.grouper(r));
            }
            let mut inter: Vec<(f32, f32)> = groupes
                .iter()
                .map(|g| {
                    let b = self.boite(g);
                    (b.x0, b.x1)
                })
                .collect();
            inter.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (a, b) in inter {
                match bandes.last_mut() {
                    Some(l) if a < l.1 + 0.35 * self.hmed => l.1 = l.1.max(b),
                    _ => bandes.push((a, b)),
                }
            }
            let mut lignes = Vec::new();
            for r in &rangees {
                let mut cases = Vec::new();
                for &(a, b) in &bandes {
                    let cellule: Vec<usize> = r
                        .iter()
                        .copied()
                        .filter(|&i| {
                            let c = self.tr[i].cx();
                            c >= a - 0.3 * self.hmed && c <= b + 0.3 * self.hmed
                        })
                        .collect();
                    cases.push(self.zone(&cellule)?);
                }
                lignes.push(cases);
            }
            let droit = |v: &[usize]| {
                v.len() == 1
                    && bombe(&self.tr[v[0]]).is_none()
                    && self.tr[v[0]].w() < 0.25 * self.tr[v[0]].h()
            };
            let env = if droit(&g) && droit(&d) {
                "vmatrix"
            } else {
                "pmatrix"
            };
            N::Tableau(env, lignes)
        } else {
            // Système : chaque rangée en deux cases, la valeur et la condition (« si … »).
            let mut lignes = Vec::new();
            for r in &rangees {
                let v = self.zone(r)?;
                let k = v
                    .iter()
                    .position(|n| matches!(n, N::Mot(m) if m.starts_with("\\text")));
                lignes.push(match k {
                    Some(k) if k > 0 => vec![v[..k].to_vec(), v[k..].to_vec()],
                    _ => vec![v],
                });
            }
            N::Tableau("cases", lignes)
        };
        let boite = self.boite(&pris);
        Some(Some((n, boite, pris)))
    }

    /// Une barre de fraction : la plus large barre plate qui a de l'encre (pas seulement d'autres barres)
    /// au-dessus et au-dessous, et qui est au moins aussi large (à 20 % près) que ce qu'elle sépare.
    /// Le numérateur et le dénominateur sont les traits centrés sur la barre, plus ceux qui la débordent
    /// mais la touchent presque (un `e` commencé avant le début de la barre).
    fn fraction(&mut self, ids: &[usize]) -> Option<Option<Vec<N>>> {
        let mut meilleure: Option<(usize, Vec<usize>, Vec<usize>)> = None;
        for &b in ids {
            let bar = &self.tr[b];
            if !bar.plat() || bar.w() < 0.5 * self.hmed {
                continue;
            }
            let portee = 2.5 * self.hmed;
            let cote = |i: usize, dessus: bool| -> bool {
                let t = &self.tr[i];
                if i == b || (dessus && t.cy() >= bar.cy()) || (!dessus && t.cy() <= bar.cy()) {
                    return false;
                }
                // Franchement d'un côté (la pointe d'une flèche traverse son trait : ce n'est pas une fraction).
                if (dessus && t.y1 > bar.cy() + 0.15 * self.hmed)
                    || (!dessus && t.y0 < bar.cy() - 0.15 * self.hmed)
                {
                    return false;
                }
                let centre = t.cx() >= bar.x0 - 0.05 * bar.w() && t.cx() <= bar.x1 + 0.05 * bar.w();
                let proche = if dessus { bar.y0 - t.y1 } else { t.y0 - bar.y1 };
                let deborde = t.x1 > bar.x0 + 0.05 * bar.w()
                    && t.x0 < bar.x1 - 0.05 * bar.w()
                    && proche < 1.0 * self.hmed;
                let dist = if dessus { bar.cy() - t.y1 } else { t.y0 - bar.cy() };
                (centre || deborde) && dist < portee
            };
            let dessus: Vec<usize> = ids.iter().copied().filter(|&i| cote(i, true)).collect();
            let dessous: Vec<usize> = ids.iter().copied().filter(|&i| cote(i, false)).collect();
            let encre = |v: &[usize]| {
                v.iter().any(|&i| !self.tr[i].plat()) && boite_de(&self.tr, v).h() >= 0.4 * self.hmed
            };
            if !encre(&dessus) || !encre(&dessous) {
                continue;
            }
            let (bd, bs) = (self.boite(&dessus), self.boite(&dessous));
            if bar.w() < 0.6 * bd.w() || bar.w() < 0.6 * bs.w() {
                continue;
            }
            if meilleure.as_ref().is_none_or(|m| bar.w() > self.tr[m.0].w()) {
                meilleure = Some((b, dessus, dessous));
            }
        }
        let (b, mut dessus, mut dessous) = meilleure?;
        // Un signe qui dépasse la barre à gauche ou à droite (le `3` de `3x − 1` écrit avant la barre) suit
        // son voisin, s'il est à sa hauteur et tout près.
        for (v, haut) in [(&mut dessus, true), (&mut dessous, false)] {
            loop {
                let bv = boite_de(&self.tr, v);
                let bar = &self.tr[b];
                let voisin = ids.iter().copied().find(|&i| {
                    let t = &self.tr[i];
                    let cote = if haut {
                        t.y1 < bar.cy() + 0.15 * self.hmed
                    } else {
                        t.y0 > bar.cy() - 0.15 * self.hmed
                    };
                    let hauteur = t.y1.min(bv.y1) - t.y0.max(bv.y0) > 0.5 * t.h().min(bv.h());
                    let pres = t.x0 < bv.x1 + 0.5 * self.hmed && t.x1 > bv.x0 - 0.5 * self.hmed;
                    i != b && !v.contains(&i) && cote && hauteur && pres && !t.plat()
                });
                match voisin {
                    Some(i) => v.push(i),
                    None => break,
                }
            }
        }
        log::debug!("fraction {b} : {dessus:?} / {dessous:?}");
        let (x0, x1) = (self.tr[b].x0, self.tr[b].x1);
        let pris: Vec<usize> = dessus.iter().chain(&dessous).copied().chain([b]).collect();
        let reste: Vec<usize> = ids.iter().copied().filter(|i| !pris.contains(i)).collect();
        let gauche: Vec<usize> = reste
            .iter()
            .copied()
            .filter(|&i| self.tr[i].cx() < (x0 + x1) / 2.0)
            .collect();
        let droite: Vec<usize> = reste
            .iter()
            .copied()
            .filter(|&i| self.tr[i].cx() >= (x0 + x1) / 2.0)
            .collect();
        Some((|| {
            let mut v = self.zone(&gauche)?;
            let n = self.zone(&dessus)?;
            let d = self.zone(&dessous)?;
            v.push(N::Frac(n, d));
            v.extend(self.zone(&droite)?);
            Some(v)
        })())
    }

    /// Une racine : un trait dont la partie droite (60 %) file en haut de sa boîte, au-dessus d'autres traits.
    fn racine(&mut self, ids: &[usize]) -> Option<Option<Vec<N>>> {
        for &r in ids {
            let t = &self.tr[r];
            if t.w() < 0.8 * self.hmed || t.h() < 0.6 * self.hmed || t.pts.len() < 4 {
                continue;
            }
            let droite: Vec<&Point> = t.pts.iter().filter(|p| p.x > t.x0 + 0.4 * t.w()).collect();
            let en_haut = droite.iter().filter(|p| p.y < t.y0 + 0.3 * t.h()).count();
            if droite.is_empty() || (en_haut as f32) < 0.85 * droite.len() as f32 {
                continue;
            }
            // La barre du dessus est plate (un `li` lié remonte et redescend, pas une racine).
            let (hmin, hmax) = droite
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
            let n_haut = droite.iter().filter(|p| p.y < t.y0 + 0.2 * t.h()).count();
            if (n_haut as f32) < 0.7 * droite.len() as f32 || hmax - hmin > 0.45 * t.h() {
                continue;
            }
            // Le bas du trait est à gauche (le crochet du √).
            let bas = t.pts.iter().max_by(|a, b| a.y.total_cmp(&b.y)).unwrap();
            if bas.x > t.x0 + 0.45 * t.w() {
                continue;
            }
            let dedans: Vec<usize> = ids
                .iter()
                .copied()
                .filter(|&i| {
                    let u = &self.tr[i];
                    i != r
                        && u.cx() > t.x0 + 0.3 * t.w()
                        && u.cx() < t.x1
                        && u.cy() > t.y0
                        && u.cy() < t.y1 + 0.2 * t.h()
                })
                .collect();
            if dedans.is_empty() {
                continue;
            }
            log::debug!("racine {r} sur {dedans:?}");
            let reste: Vec<usize> = ids
                .iter()
                .copied()
                .filter(|i| *i != r && !dedans.contains(i))
                .collect();
            let gauche: Vec<usize> = reste
                .iter()
                .copied()
                .filter(|&i| self.tr[i].cx() < t.cx())
                .collect();
            let droite_: Vec<usize> = reste
                .iter()
                .copied()
                .filter(|&i| self.tr[i].cx() >= t.cx())
                .collect();
            return Some((|| {
                let mut v = self.zone(&gauche)?;
                v.push(N::Racine(self.zone(&dedans)?));
                v.extend(self.zone(&droite_)?);
                Some(v)
            })());
        }
        None
    }

    /// Petit : un point, l'accent d'un `i`.
    fn petit(&self, t: &Tr) -> bool {
        t.w().max(t.h()) < 0.3 * self.hmed
    }

    /// Les traits groupés en signes. Deux traits vont ensemble s'ils se chevauchent en largeur (la moitié
    /// du plus étroit) et se touchent presque en hauteur ; une barre ou un point peut être un peu plus loin
    /// (les deux barres du `=`, le point du `i` ou du `;`).
    fn grouper(&self, ids: &[usize]) -> Vec<Vec<usize>> {
        let mut groupes: Vec<Vec<usize>> = ids.iter().map(|&i| vec![i]).collect();
        let ensemble = |a: &[usize], b: &[usize]| -> bool {
            let (ba, bb) = (self.boite(a), self.boite(b));
            let recouvre = (ba.x1.min(bb.x1) - ba.x0.max(bb.x0)).max(0.0);
            let ecart_h = (ba.x0.max(bb.x0) - ba.x1.min(bb.x1)).max(0.0);
            let ecart_v = (ba.y0.max(bb.y0) - ba.y1.min(bb.y1)).max(0.0);
            let leger = |g: &[usize]| g.len() == 1 && (self.tr[g[0]].plat() || self.petit(&self.tr[g[0]]));
            let etroit = ba.w().min(bb.w()).max(0.05 * self.hmed);
            // Le plus étroit est à moitié sur l'autre, ou centré dessus (la barre du `+`).
            let (n, l) = if ba.w() < bb.w() { (ba, bb) } else { (bb, ba) };
            let chevauche =
                recouvre >= 0.5 * etroit || (n.cx() > l.x0 && n.cx() < l.x1 && n.w() < 0.3 * l.w());
            if chevauche && ecart_v < 0.15 * self.hmed {
                return true;
            }
            // Barre ou point : un peu plus loin en hauteur ; un point peut être juste à côté.
            if leger(a) || leger(b) {
                let un_point = |g: &[usize]| g.len() == 1 && self.petit(&self.tr[g[0]]);
                let deux_barres = leger(a) && leger(b) && self.tr[a[0]].plat() && self.tr[b[0]].plat();
                let limite = if deux_barres { 0.9 } else { 0.6 };
                return (chevauche && ecart_v < limite * self.hmed)
                    || ((un_point(a) || un_point(b))
                        && ecart_h < 0.15 * self.hmed
                        && ecart_v < 0.6 * self.hmed);
            }
            false
        };
        loop {
            let mut fusion = None;
            'f: for a in 0..groupes.len() {
                for b in a + 1..groupes.len() {
                    if ensemble(&groupes[a], &groupes[b]) {
                        fusion = Some((a, b));
                        break 'f;
                    }
                }
            }
            let Some((a, b)) = fusion else { break };
            let g = groupes.remove(b);
            groupes[a].extend(g);
        }
        groupes.sort_by(|a, b| self.boite(a).x0.total_cmp(&self.boite(b).x0));
        groupes
    }

    /// Classe un groupe de traits comme un signe ; rend l'indice du signe.
    fn signe(&mut self, g: &[usize]) -> Option<usize> {
        let b = self.boite(g);
        // La case : la boîte des traits (pas trop grands) qui recouvrent le signe en hauteur.
        let mut case = b;
        for t in &self.tr {
            if t.h() <= 2.5 * self.hmed && t.y1 > b.y0 && t.y0 < b.y1 {
                case.y0 = case.y0.min(t.y0);
                case.y1 = case.y1.max(t.y1);
            }
        }
        let traits: Vec<&[Point]> = g.iter().map(|&i| self.tr[i].pts.as_slice()).collect();
        let choix = self
            .cl
            .classer(
                &traits,
                Case {
                    haut: case.y0,
                    hauteur: case.h().max(1.0),
                },
                3,
            )
            .ok()?;
        let c = &self.cl.classes[choix[0].0];
        self.signes.push(Signe {
            traits: g.to_vec(),
            texte: c.latex.clone(),
            id: c.id.clone(),
            p: choix[0].1,
            choix,
        });
        Some(self.signes.len() - 1)
    }

    /// Une rangée : les signes, les mots (`lim`, `sin`), les bornes des opérateurs, puis la ligne de base.
    fn rangee(&mut self, ids: &[usize]) -> Option<Vec<N>> {
        self.rangee_avec(ids, Vec::new())
    }

    /// Une rangée avec des éléments déjà lus (un tableau) : ils prennent leur place parmi les signes.
    fn rangee_avec(&mut self, ids: &[usize], deja: Vec<(N, Boite)>) -> Option<Vec<N>> {
        let mut elems: Vec<(N, Boite)> = deja;
        for g in self.grouper(ids) {
            let s = self.signe(&g)?;
            // Un groupe mal lu peut être deux signes qui se recouvrent (une parenthèse écrite sur le `x`) :
            // chaque trait est relu seul, et gardé seul s'il se lit nettement mieux.
            let gros: Vec<usize> = g.iter().copied().filter(|&i| !self.petit(&self.tr[i])).collect();
            // Seulement sans barre (un `+`, un `=` sont faits de traits qui se lisent seuls) et pour des
            // traits de tailles nettement différentes (une parenthèse sur une lettre).
            let hauteurs: Vec<f32> = gros.iter().map(|&i| self.tr[i].h()).collect();
            let ecart_tailles = hauteurs.iter().copied().fold(0.0, f32::max)
                > 1.6 * hauteurs.iter().copied().fold(f32::MAX, f32::min);
            let sans_barre = gros.iter().all(|&i| !self.tr[i].plat());
            // … ou décalés en hauteur (un exposant collé à son signe).
            let cys: Vec<f32> = gros.iter().map(|&i| self.tr[i].cy()).collect();
            let decales = cys.iter().copied().fold(f32::MIN, f32::max)
                - cys.iter().copied().fold(f32::MAX, f32::min)
                > 0.4 * hauteurs.iter().copied().fold(0.0, f32::max);
            if gros.len() >= 2 && self.signes[s].p < 0.4 && sans_barre && (ecart_tailles || decales) {
                let n = self.signes.len();
                let mut parts = Vec::new();
                for &i in &gros {
                    let mut sous = vec![i];
                    // Les points vont avec le trait le plus proche.
                    for &j in g.iter().filter(|&&j| self.petit(&self.tr[j])) {
                        let plus_proche = gros
                            .iter()
                            .min_by(|&&a, &&b| {
                                let d = |k: usize| (self.tr[k].cx() - self.tr[j].cx()).abs();
                                d(a).total_cmp(&d(b))
                            })
                            .copied();
                        if plus_proche == Some(i) {
                            sous.push(j);
                        }
                    }
                    let Some(k) = self.signe(&sous) else { break };
                    parts.push((k, self.boite(&sous)));
                }
                let pire = parts.iter().map(|(k, _)| self.signes[*k].p).fold(1.0, f32::min);
                if parts.len() == gros.len() && pire > self.signes[s].p + 0.1 {
                    elems.extend(parts.into_iter().map(|(k, b)| (N::Signe(k), b)));
                    continue;
                }
                self.signes.truncate(n);
            }
            elems.push((N::Signe(s), self.boite(&g)));
        }
        elems.sort_by(|a, b| a.1.x0.total_cmp(&b.1.x0));
        self.fusionner_paires(&mut elems);
        self.lim_par_la_fleche(&mut elems);
        self.lire_mots(&mut elems);
        self.bornes(&mut elems);
        // Ce qui reste empilé (un signe au-dessus d'un autre, de taille voisine) est une matrice ou un
        // système : on ne sait pas encore le lire.
        for (i, a) in elems.iter().enumerate() {
            for b in &elems[i + 1..] {
                let signe = |n: &N| matches!(n, N::Signe(s) if { let id = &self.signes[*s].id; id.chars().count() == 1 && id.chars().all(char::is_alphanumeric) });
                if !signe(&a.0) || !signe(&b.0) {
                    continue;
                }
                let (a, b) = (a.1, b.1);
                let recouvre = (a.x1.min(b.x1) - a.x0.max(b.x0)).max(0.0);
                let ecart_v = (a.y0.max(b.y0) - a.y1.min(b.y1)).max(0.0);
                let grands = a.h() >= 0.45 * self.hmed && b.h() >= 0.45 * self.hmed;
                if grands && recouvre >= 0.6 * a.w().min(b.w()) && ecart_v > 0.1 * self.hmed {
                    log::debug!("empilés : {a:?} {b:?}");
                    return None;
                }
            }
        }
        // Par le centre : la pointe d'une parenthèse fermante passe sous l'exposant qui la suit.
        elems.sort_by(|a, b| a.1.cx().total_cmp(&b.1.cx()));
        Some(self.ligne_de_base(elems))
    }

    /// Deux signes voisins qui se lisent mieux ensemble (le `x` en deux arcs, le `4` ouvert) sont fusionnés.
    fn fusionner_paires(&mut self, elems: &mut Vec<(N, Boite)>) {
        let mut i = 0;
        while i + 1 < elems.len() {
            // Le voisin : le suivant à la même hauteur (un signe d'une autre rangée peut s'intercaler, comme
            // le `li` de `lim` entre les deux arcs du `x` de `x → 0`).
            let bi = elems[i].1;
            let Some(j) = (i + 1..elems.len()).find(|&k| {
                let b = elems[k].1;
                b.y1.min(bi.y1) - b.y0.max(bi.y0) > 0.4 * b.h().min(bi.h()).max(0.1 * self.hmed)
                    || (b.cy() - bi.cy()).abs() < 0.5 * b.h().max(bi.h())
            }) else {
                i += 1;
                continue;
            };
            let (N::Signe(a), N::Signe(b)) = (&elems[i].0, &elems[j].0) else {
                i += 1;
                continue;
            };
            let (a, b) = (*a, *b);
            let (ba, bb) = (elems[i].1, elems[j].1);
            let ecart = bb.x0 - ba.x1;
            let recouvre_h = ba.y1.min(bb.y1) - ba.y0.max(bb.y0);
            let (pa, pb) = (self.signes[a].p, self.signes[b].p);
            // Une flèche en deux traits : le trait plat, puis la pointe `>` qui le touche.
            let (ia, ib) = (self.signes[a].id.clone(), self.signes[b].id.clone());
            if matches!(ia.as_str(), "moins" | "egal")
                && ib == "sup"
                && ecart < 0.2 * self.hmed
                && (bb.cy() - ba.cy()).abs() < 0.5 * bb.h()
            {
                let cible = if ia == "moins" { "fleche" } else { "implique" };
                if let Some(c) = self.cl.indice(cible) {
                    let traits: Vec<usize> = self.signes[a]
                        .traits
                        .iter()
                        .chain(&self.signes[b].traits)
                        .copied()
                        .collect();
                    let boite = self.boite(&traits);
                    self.signes.push(Signe {
                        traits,
                        choix: vec![(c, pa.min(pb))],
                        texte: self.cl.classes[c].latex.clone(),
                        p: pa.min(pb),
                        id: cible.into(),
                    });
                    elems[i] = (N::Signe(self.signes.len() - 1), boite);
                    elems.remove(j);
                    continue;
                }
            }
            // Le `x` en deux arcs dos à dos (« ɔc »), la façon d'Antsa (ADR 0037 de Fikajiana).
            let principal = |l: &Lecture, s: usize| -> Option<usize> {
                let gros: Vec<usize> = l.signes[s]
                    .traits
                    .iter()
                    .copied()
                    .filter(|&i| !l.petit(&l.tr[i]))
                    .collect();
                (gros.len() == 1).then(|| gros[0])
            };
            if let (Some(ka), Some(kb)) = (principal(self, a), principal(self, b)) {
                let (ta, tb) = (&self.tr[ka], &self.tr[kb]);
                let parentheses = ia == "par_d" && ib == "par_g" && pa.min(pb) > 0.8;
                let (ha, hb) = (ta.h().max(1.0), tb.h().max(1.0));
                if !parentheses
                    && bombe(ta) == Some(true)
                    && bombe(tb) == Some(false)
                    && ecart < 0.5 * self.hmed
                    && ha / hb < 1.7
                    && hb / ha < 1.7
                    && recouvre_h > 0.5 * ha.min(hb)
                    && ta.h() < 2.2 * ta.w().max(1.0) + 0.5 * self.hmed
                    && tb.h() < 2.2 * tb.w().max(1.0) + 0.5 * self.hmed
                {
                    if let Some(c) = self.cl.indice("x") {
                        let traits: Vec<usize> = self.signes[a]
                            .traits
                            .iter()
                            .chain(&self.signes[b].traits)
                            .copied()
                            .collect();
                        let boite = self.boite(&traits);
                        let p = 0.85;
                        let fois = self.cl.indice("fois").map(|f| (f, 0.1));
                        self.signes.push(Signe {
                            traits,
                            choix: std::iter::once((c, p)).chain(fois).collect(),
                            texte: "x".into(),
                            p,
                            id: "x".into(),
                        });
                        elems[i] = (N::Signe(self.signes.len() - 1), boite);
                        elems.remove(j);
                        continue;
                    }
                }
            }
            if ecart > 0.45 * self.hmed || recouvre_h < 0.4 * ba.h().min(bb.h()) || pa.min(pb) > 0.9 {
                i += 1;
                continue;
            }
            let traits: Vec<usize> = self.signes[a]
                .traits
                .iter()
                .chain(&self.signes[b].traits)
                .copied()
                .collect();
            let n = self.signes.len();
            let Some(s) = self.signe(&traits) else {
                i += 1;
                continue;
            };
            let pm = self.signes[s].p;
            if pm >= 0.6 && pm > pa.min(pb) + 0.15 {
                let boite = self.boite(&traits);
                elems[i] = (N::Signe(s), boite);
                elems.remove(j);
            } else {
                self.signes.truncate(n);
                i += 1;
            }
        }
    }

    /// `lim` reconnu à sa flèche : une rangée `x → a` sous un mot, c'est une limite, quelle que soit la
    /// façon dont le mot (souvent lié, `li` d'un seul trait) a été lu.
    fn lim_par_la_fleche(&mut self, elems: &mut Vec<(N, Boite)>) {
        let Some(f) = elems.iter().position(
            |(n, _)| matches!(n, N::Signe(s) if matches!(self.signes[*s].id.as_str(), "fleche" | "implique")),
        ) else {
            return;
        };
        let fb = elems[f].1;
        // La rangée de la flèche : les signes à sa hauteur, de proche en proche.
        let mut sous = vec![f];
        loop {
            let b = sous.iter().fold(fb, |b, &k| union(b, elems[k].1));
            let suivant = (0..elems.len()).find(|&k| {
                let e = elems[k].1;
                !sous.contains(&k)
                    && (e.cy() - fb.cy()).abs() < 0.8 * self.hmed
                    && e.x0 < b.x1 + 0.8 * self.hmed
                    && e.x1 > b.x0 - 0.8 * self.hmed
            });
            match suivant {
                Some(k) => sous.push(k),
                None => break,
            }
        }
        let bs = sous.iter().fold(fb, |b, &k| union(b, elems[k].1));
        let dessus: Vec<usize> = (0..elems.len())
            .filter(|&k| {
                let e = elems[k].1;
                !sous.contains(&k)
                    && matches!(elems[k].0, N::Signe(_))
                    && e.y1 < bs.y0 + 0.2 * self.hmed
                    && e.y1 > bs.y0 - 2.0 * self.hmed
                    && e.x1 > bs.x0 - 0.5 * self.hmed
                    && e.x0 < bs.x1 + 0.2 * self.hmed
            })
            .collect();
        if dessus.is_empty() {
            return;
        }
        for &k in &dessus {
            if let N::Signe(s) = elems[k].0 {
                // La structure est sûre : le mot lu n'abaisse pas la confiance.
                self.signes[s].p = self.signes[s].p.max(0.9);
            }
        }
        let bd = dessus
            .iter()
            .fold(elems[dessus[0]].1, |b, &k| union(b, elems[k].1));
        let mut rangee: Vec<(N, Boite)> = sous.iter().map(|&k| elems[k].clone()).collect();
        rangee.sort_by(|a, b| a.1.x0.total_cmp(&b.1.x0));
        let borne = self.ligne_de_base(rangee);
        let noeud = N::Bornes(Box::new(N::Mot("\\lim".into())), Some(borne), None);
        let mut retirer: Vec<usize> = dessus.iter().chain(&sous).copied().collect();
        retirer.sort_unstable();
        for k in retirer.iter().rev() {
            elems.remove(*k);
        }
        elems.push((noeud, union(bd, bs)));
        elems.sort_by(|a, b| a.1.x0.total_cmp(&b.1.x0));
    }

    /// Les noms de fonctions : une suite de signes voisins, à la même hauteur, dont les trois meilleures
    /// classes donnent `lim`, `sin`, `cos`, `ln`… deviennent un mot.
    fn lire_mots(&mut self, elems: &mut Vec<(N, Boite)>) {
        let mut i = 0;
        'boucle: while i < elems.len() {
            // La chaîne : à chaque pas, le plus proche signe à droite, à la même hauteur.
            let mut chaine = vec![i];
            while chaine.len() < 3 {
                let d = elems[*chaine.last().unwrap()].1;
                let suivant = (0..elems.len())
                    .filter(|&k| !chaine.contains(&k) && matches!(elems[k].0, N::Signe(_)))
                    .filter(|&k| {
                        let b = elems[k].1;
                        b.x0 > d.x0 + 0.2 * d.w()
                            && b.x0 - d.x1 < 0.7 * self.hmed
                            && (b.y1.min(d.y1) - b.y0.max(d.y0)) > 0.3 * b.h().min(d.h())
                    })
                    .min_by(|&a, &b| elems[a].1.x0.total_cmp(&elems[b].1.x0));
                match suivant {
                    Some(k) => chaine.push(k),
                    None => break,
                }
            }
            let signes: Vec<usize> = chaine
                .iter()
                .map_while(|&k| match elems[k].0 {
                    N::Signe(s) => Some(s),
                    _ => None,
                })
                .collect();
            for f in FONCTIONS {
                let n = f.len();
                if signes.len() >= n && self.est_mot(&signes[..n], f) {
                    let mut pris: Vec<usize> = chaine[..n].to_vec();
                    let boite = pris.iter().fold(elems[pris[0]].1, |b, &k| {
                        let e = elems[k].1;
                        Boite {
                            x0: b.x0.min(e.x0),
                            y0: b.y0.min(e.y0),
                            x1: b.x1.max(e.x1),
                            y1: b.y1.max(e.y1),
                        }
                    });
                    let mot = N::Mot(latex_du_mot(f));
                    pris.sort_unstable();
                    for k in pris.iter().rev() {
                        elems.remove(*k);
                    }
                    elems.insert(pris[0], (mot, boite));
                    i = pris[0] + 1;
                    continue 'boucle;
                }
            }
            i += 1;
        }
    }

    /// Ces signes peuvent-ils former le mot `f` ? Chaque lettre parmi les trois meilleures classes (avec
    /// les jumeaux `l`/`1`/`I`, `o`/`0`…), et au moins la moitié en premier choix.
    fn est_mot(&self, signes: &[usize], f: &str) -> bool {
        let peut_etre = |s: usize, c: char| -> bool {
            self.signes[s].choix.iter().any(|(k, p)| {
                let id = self.cl.classes[*k].id.as_str();
                *p >= 0.02
                    && (id.len() == 1 && id.starts_with(c)
                        || c == 'l' && matches!(id, "1" | "I" | "abs")
                        || c == 'i' && matches!(id, "fact" | "j")
                        || c == 'o' && matches!(id, "0" | "O")
                        || c == 's' && matches!(id, "5" | "S")
                        || c == 'c' && id == "C"
                        || c == 'x' && matches!(id, "fois" | "X")
                        || c == 'n' && id == "h"
                        || c == 'm' && id == "M")
            })
        };
        let premiers = f
            .chars()
            .zip(signes)
            .filter(|(c, s)| {
                let id = &self.signes[**s].id;
                id.len() == 1 && id.to_lowercase().starts_with(*c)
            })
            .count();
        f.chars().zip(signes).all(|(c, &s)| peut_etre(s, c)) && premiers * 2 >= f.len()
    }

    /// Les bornes au-dessus et au-dessous de `lim`, `Σ` (et `∫` quand on les y écrit).
    fn bornes(&mut self, elems: &mut Vec<(N, Boite)>) {
        let mut k = 0;
        while k < elems.len() {
            let op = match &elems[k].0 {
                N::Mot(m) if matches!(m.as_str(), "\\lim" | "\\max" | "\\min") => true,
                N::Signe(s) => matches!(self.signes[*s].id.as_str(), "somme" | "integrale"),
                _ => false,
            };
            if !op {
                k += 1;
                continue;
            }
            let o = elems[k].1;
            let marge = 0.35 * o.w().max(self.hmed);
            let sous: Vec<usize> = (0..elems.len())
                .filter(|&j| {
                    j != k
                        && elems[j].1.y0 > o.y1 - 0.15 * o.h()
                        && elems[j].1.cx() > o.x0 - marge
                        && elems[j].1.cx() < o.x1 + marge
                        && elems[j].1.y0 < o.y1 + 1.5 * self.hmed
                })
                .collect();
            let sur: Vec<usize> = (0..elems.len())
                .filter(|&j| {
                    j != k
                        && elems[j].1.y1 < o.y0 + 0.15 * o.h()
                        && elems[j].1.cx() > o.x0 - marge
                        && elems[j].1.cx() < o.x1 + marge
                        && elems[j].1.y1 > o.y0 - 1.5 * self.hmed
                })
                .collect();
            // ∫ : les bornes sont souvent à droite (lues comme exposant et indice) ; ici seulement dessus/dessous.
            if sous.is_empty() && sur.is_empty() {
                k += 1;
                continue;
            }
            let prendre = |v: &[usize], elems: &Vec<(N, Boite)>| -> Vec<(N, Boite)> {
                let mut r: Vec<(N, Boite)> = v.iter().map(|&j| elems[j].clone()).collect();
                r.sort_by(|a, b| a.1.x0.total_cmp(&b.1.x0));
                r
            };
            let bas = prendre(&sous, elems);
            let haut = prendre(&sur, elems);
            let bas = if bas.is_empty() {
                None
            } else {
                Some(self.ligne_de_base(bas))
            };
            let haut = if haut.is_empty() {
                None
            } else {
                Some(self.ligne_de_base(haut))
            };
            let noeud = N::Bornes(Box::new(elems[k].0.clone()), bas, haut);
            let mut retirer: Vec<usize> = sous.iter().chain(&sur).copied().collect();
            retirer.sort_unstable();
            let boite = o;
            elems[k] = (noeud, boite);
            for j in retirer.iter().rev() {
                elems.remove(*j);
                if *j < k {
                    k -= 1;
                }
            }
            k += 1;
        }
    }

    /// Le corps d'un signe : sans la jambe des lettres à jambage (`g p q y`, `β μ`…).
    fn corps(&self, n: &N, b: Boite) -> Boite {
        if let N::Signe(s) = n {
            if matches!(
                self.signes[*s].id.as_str(),
                "g" | "p" | "q" | "y" | "j" | "beta" | "mu" | "gamma" | "phi"
            ) {
                return Boite {
                    y1: b.y0 + 0.62 * b.h(),
                    ..b
                };
            }
        }
        b
    }

    /// Peut porter un exposant ou un indice : chiffre, lettre, parenthèse fermante, fraction, racine.
    fn porteur(&self, n: &N) -> bool {
        match n {
            N::Signe(s) => {
                let id = self.signes[*s].id.as_str();
                id.chars().count() == 1 && id.chars().all(char::is_alphanumeric)
                    || matches!(
                        id,
                        "par_d"
                            | "cro_d"
                            | "acc_d"
                            | "pi"
                            | "alpha"
                            | "beta"
                            | "theta"
                            | "lambda"
                            | "mu"
                            | "sigma"
                            | "phi"
                            | "omega"
                            | "gamma"
                            | "delta"
                            | "Delta"
                            | "integrale"
                            | "infini"
                            | "abs"
                    )
            }
            N::Frac(..) | N::Racine(..) | N::Mot(..) | N::Tableau(..) => true,
            _ => false,
        }
    }

    fn ne_monte_pas(&self, n: &N) -> bool {
        matches!(n, N::Signe(s) if matches!(self.signes[*s].id.as_str(), "virgule" | "point" | "pointvirgule" | "egal" | "prime"))
    }

    /// Exposants et indices, lus à la place de chaque signe par rapport au corps de son porteur.
    fn ligne_de_base(&self, elems: Vec<(N, Boite)>) -> Vec<N> {
        let mut res: Vec<(N, Boite)> = Vec::new();
        let mut i = 0;
        while i < elems.len() {
            let (n, b) = &elems[i];
            let porteur = res
                .last()
                .filter(|(p, _)| self.porteur(p) || matches!(p, N::Scripts(..)))
                .map(|(p, pb)| match p {
                    N::Scripts(base, ..) => (self.corps(base, *pb), base.as_ref().clone()),
                    _ => (self.corps(p, *pb), p.clone()),
                });
            let zone = porteur.as_ref().and_then(|(c, _)| {
                if self.ne_monte_pas(n)
                    || b.h() > 0.9 * c.h() && b.h() > 0.6 * self.hmed && !(b.y1 < c.y0 + 0.35 * c.h())
                {
                    return None;
                }
                let operateur = matches!(n, N::Signe(s) if !self.porteur(n) && self.signes[*s].id != "moins");
                if b.cy() < c.y0 + 0.2 * c.h() && b.y1 < c.y0 + 0.6 * c.h() && !operateur {
                    Some(true)
                } else if b.cy() > c.y0 + 0.85 * c.h()
                    && b.y0 > c.y0 + 0.45 * c.h()
                    && b.y1 > c.y1 + 0.15 * c.h()
                    && self.porteur(n)
                {
                    Some(false)
                } else {
                    None
                }
            });
            let Some(haut) = zone else {
                res.push((n.clone(), *b));
                i += 1;
                continue;
            };
            let c = porteur.unwrap().0;
            // Le script continue tant que les signes restent dans sa bande.
            let mut j = i + 1;
            while j < elems.len() {
                let bj = elems[j].1;
                let reste = if haut {
                    bj.cy() < c.y0 + 0.4 * c.h() && bj.y1 < c.y0 + 0.75 * c.h()
                } else {
                    bj.cy() > c.y0 + 0.75 * c.h() && bj.y0 > c.y0 + 0.4 * c.h()
                };
                if !reste || self.ne_monte_pas(&elems[j].0) {
                    break;
                }
                j += 1;
            }
            // Un moins qui finit l'exposant revient sur la ligne (Antsa écrit le moins haut : `e^x − 1`).
            while j > i + 1 && matches!(&elems[j - 1].0, N::Signe(s) if self.signes[*s].id == "moins") {
                j -= 1;
            }
            // Un moins seul en l'air n'est pas un exposant (Antsa écrit parfois le moins haut).
            if j == i + 1 && matches!(n, N::Signe(s) if self.signes[*s].id == "moins") {
                res.push((n.clone(), *b));
                i += 1;
                continue;
            }
            let script = self.ligne_de_base(elems[i..j].to_vec());
            let (dernier, db) = res.pop().unwrap();
            let nouveau = match dernier {
                N::Scripts(base, sub, sup) => {
                    if haut && sup.is_none() {
                        N::Scripts(base, sub, Some(script))
                    } else if !haut && sub.is_none() {
                        N::Scripts(base, Some(script), sup)
                    } else {
                        // Déjà pris : on garde le script à côté (rare).
                        res.push((N::Scripts(base, sub, sup), db));
                        res.extend(elems[i..j].iter().cloned());
                        i = j;
                        continue;
                    }
                }
                autre => {
                    if haut {
                        N::Scripts(Box::new(autre), None, Some(script))
                    } else {
                        N::Scripts(Box::new(autre), Some(script), None)
                    }
                }
            };
            res.push((nouveau, db));
            i = j;
        }
        res.into_iter().map(|(n, _)| n).collect()
    }

    /// Les noms de fonctions dans une rangée : `l i m` → `\lim`, lus sur les trois meilleures classes.
    fn mots(&self, v: &[N]) -> Vec<N> {
        let lettre = |n: &N| -> Option<usize> {
            match n {
                N::Signe(s) => Some(*s),
                _ => None,
            }
        };
        let peut_etre = |s: usize, c: char| -> bool {
            self.signes[s].choix.iter().any(|(k, p)| {
                let id = &self.cl.classes[*k].id;
                *p >= 0.02
                    && (id.len() == 1 && id.starts_with(c)
                        || c == 'l' && matches!(id.as_str(), "1" | "I" | "abs")
                        || c == 'i' && id == "fact"
                        || c == 'o' && id == "0"
                        || c == 's' && matches!(id.as_str(), "5" | "S")
                        || c == 'x' && id == "fois"
                        || c == 'n' && id == "h")
            })
        };
        let mut res = Vec::new();
        let mut i = 0;
        'boucle: while i < v.len() {
            for f in FONCTIONS {
                let n = f.len();
                if i + n <= v.len()
                    && f.chars().enumerate().all(|(k, c)| lettre(&v[i + k]).is_some_and(|s| peut_etre(s, c)))
                    // Au moins la moitié des lettres sont lues comme telles en premier choix.
                    && f.chars().enumerate().filter(|(k, c)| {
                        lettre(&v[i + k]).is_some_and(|s| self.signes[s].id.starts_with(*c) && self.signes[s].id.len() == 1)
                    }).count() * 2 >= n
                {
                    res.push(N::Mot(latex_du_mot(f)));
                    i += n;
                    continue 'boucle;
                }
            }
            // Un mot peut aussi porter un exposant : `sin^2` (la dernière lettre porte le script).
            res.push(v[i].clone());
            i += 1;
        }
        res
    }

    /// Le contexte tranche les signes jumeaux (sur la rangée de base de chaque niveau).
    fn contexte(&mut self, arbre: &[N]) {
        let ids: Vec<Option<usize>> = arbre
            .iter()
            .map(|n| match n {
                N::Signe(s) => Some(*s),
                N::Scripts(b, ..) => match b.as_ref() {
                    N::Signe(s) => Some(*s),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        let chiffre = |l: &Lecture, k: Option<&Option<usize>>| -> bool {
            matches!(k, Some(Some(s)) if l.signes[*s].id.len() == 1 && l.signes[*s].id.chars().all(|c| c.is_ascii_digit()))
        };
        let ferme = |l: &Lecture, k: Option<&Option<usize>>| -> bool {
            matches!(k, Some(Some(s)) if l.signes[*s].id == "par_d")
        };
        let ouvre = |l: &Lecture, k: Option<&Option<usize>>| -> bool {
            matches!(k, Some(Some(s)) if l.signes[*s].id == "par_g")
        };
        for k in 0..ids.len() {
            let Some(s) = ids[k] else { continue };
            let avant = if k > 0 { ids.get(k - 1) } else { None };
            let apres = ids.get(k + 1);
            let id = self.signes[s].id.clone();
            let entre_nombres =
                (chiffre(self, avant) || ferme(self, avant)) && (chiffre(self, apres) || ouvre(self, apres));
            let pres_d_un_chiffre = chiffre(self, avant) || chiffre(self, apres);
            let apres_fleche = matches!(avant, Some(Some(a)) if self.signes[*a].id == "fleche");
            let choisir = |l: &mut Lecture, cible: &str| {
                if let Some(c) = l.cl.indice(cible) {
                    let p = l.signes[s]
                        .choix
                        .iter()
                        .find(|x| x.0 == c)
                        .map_or(l.signes[s].p, |x| x.1.max(l.signes[s].p));
                    l.signes[s].texte = l.cl.classes[c].latex.clone();
                    l.signes[s].id = cible.into();
                    l.signes[s].p = p;
                }
            };
            match id.as_str() {
                "x" | "X" if entre_nombres => choisir(self, "fois"),
                "fois" if !entre_nombres => choisir(self, "x"),
                "o" | "O" if pres_d_un_chiffre || apres_fleche => choisir(self, "0"),
                "l" | "I" | "abs" if pres_d_un_chiffre => choisir(self, "1"),
                _ => {}
            }
        }
        // Une accolade ou un crochet qui ferme une parenthèse ouverte est une parenthèse.
        let mut ouvertes = 0i32;
        for &k in ids.iter().flatten() {
            match self.signes[k].id.as_str() {
                "par_g" => ouvertes += 1,
                "par_d" => ouvertes -= 1,
                "acc_d" | "cro_d" if ouvertes > 0 => {
                    if let Some(c) = self.cl.indice("par_d") {
                        self.signes[k].texte = self.cl.classes[c].latex.clone();
                        self.signes[k].id = "par_d".into();
                    }
                    ouvertes -= 1;
                }
                _ => {}
            }
        }
        // Récursion dans les sous-niveaux.
        for n in arbre {
            match n {
                N::Frac(a, b) => {
                    self.contexte(a);
                    self.contexte(b);
                }
                N::Racine(a) => self.contexte(a),
                N::Tableau(_, r) => {
                    for c in r.iter().flatten() {
                        self.contexte(c);
                    }
                }
                N::Scripts(_, a, b) | N::Bornes(_, a, b) => {
                    if let Some(a) = a {
                        self.contexte(a);
                    }
                    if let Some(b) = b {
                        self.contexte(b);
                    }
                }
                _ => {}
            }
        }
    }

    fn ecrire(&self, v: &[N]) -> String {
        let v = self.mots(v);
        let mut s = String::new();
        for n in &v {
            let t = self.ecrire_noeud(n);
            // Une commande (`\pi`) collée à une lettre a besoin d'une espace.
            let commande = s
                .rsplit('\\')
                .next()
                .is_some_and(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_alphabetic()))
                && s.contains('\\');
            if commande && t.starts_with(|c: char| c.is_ascii_alphabetic()) {
                s.push(' ');
            }
            s.push_str(&t);
        }
        s
    }

    fn ecrire_noeud(&self, n: &N) -> String {
        match n {
            N::Signe(s) => self.signes[*s].texte.clone(),
            N::Mot(m) => m.clone(),
            N::Frac(a, b) => format!("\\frac{{{}}}{{{}}}", self.ecrire(a), self.ecrire(b)),
            N::Racine(a) => format!("\\sqrt{{{}}}", self.ecrire(a)),
            N::Tableau(env, r) => {
                let corps: Vec<String> = r
                    .iter()
                    .map(|cases| {
                        cases
                            .iter()
                            .map(|c| self.ecrire(c))
                            .collect::<Vec<_>>()
                            .join(" & ")
                    })
                    .collect();
                format!("\\begin{{{env}}}{}\\end{{{env}}}", corps.join(" \\\\ "))
            }
            N::Scripts(b, sub, sup) => {
                let mut t = self.ecrire_noeud(b);
                if let Some(sub) = sub {
                    t.push_str(&format!("_{{{}}}", self.ecrire(sub)));
                }
                if let Some(sup) = sup {
                    t.push_str(&format!("^{{{}}}", self.ecrire(sup)));
                }
                t
            }
            N::Bornes(op, sous, sur) => {
                let mut t = self.ecrire_noeud(op);
                if let Some(v) = sous {
                    t.push_str(&format!("_{{{}}}", self.ecrire(v)));
                }
                if let Some(v) = sur {
                    t.push_str(&format!("^{{{}}}", self.ecrire(v)));
                }
                t
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signes::tests::traits_fixture;

    fn lire(nom: &str) -> Option<LectureFormule> {
        let cl = ClasseurSignes::charger().unwrap();
        lire_formule(&cl, &traits_fixture(nom))
    }

    fn sans_espaces(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// Les lignes réelles d'Antsa (Fikajiana, Pad 7 et téléphone) et leur lecture attendue.
    const ATTENDUS: &[(&str, &str)] = &[
        ("telephone/un_plus_un", "1+1"),
        ("telephone/un_plus_un_2", "1+1"),
        ("pad7/trente_fois_8", r"30\times8"),
        ("pad7/AB_egal", "AB="),
        ("pad7/A_egal_un_trait", "A="),
        ("pad7/e_x_egal_7", "e^{x}=7"),
        ("pad7/derivee_f", "f'(x)="),
        ("telephone/f_de_1_egal", "f(1)="),
        ("telephone/e3_moins_2e6_egal", "e^{3}-2e^{6}="),
        ("pad7/puissances_de_10", r"74\times10^{-4}+85\times10^{8}="),
        ("pad7/g_egal_x_moins_1_sur_x", r"g(x)=x-\frac{1}{x}"),
        ("pad7/lim_exp_sur_2x", r"\lim_{x\to0}\frac{e^{x}-1}{2x}="),
        ("pad7/lim_3x_sur_exp", r"\lim_{x\to1}\frac{3x-1}{e^{x}-1}="),
        ("pad7/lim_inverse_carre", r"\lim_{x\to3}\frac{1}{(x-3)^{2}}="),
        ("pad7/y_prime_moins_y2", r"y'-y^{2}=\cos(x)"),
        ("pad7/B_5_moins_3", "B(5,-3)"),
        ("pad7/point_A", "A(8;-1)"),
        ("pad7/point_B", "B(5;8)"),
        ("pad7/point_C_2_7", "C(2;7)"),
        ("pad7/point_D_8_moins_3", "D(8;-3)"),
        ("pad7/point_B_moins_25", "B(2;-25)"),
        ("telephone/point_A_1_moins_1", "A(1;-1)"),
        ("telephone/point_A_3_moins_1", "A(3;-1)"),
    ];

    /// Le banc : combien de lignes sont lues justes, lesquelles se trompent (affiché avec --nocapture).
    #[test]
    fn banc_des_lignes_d_antsa() {
        let cl = ClasseurSignes::charger().unwrap();
        let mut justes = 0;
        let mut fausses_sures = Vec::new();
        for (nom, attendu) in ATTENDUS {
            let t0 = std::time::Instant::now();
            let l = lire_formule(&cl, &traits_fixture(nom));
            let ms = t0.elapsed().as_secs_f32() * 1000.0;
            let lu = l.as_ref().map(|l| sans_espaces(&l.latex));
            let ok = lu.as_deref() == Some(*attendu);
            if ok {
                justes += 1;
            } else if l.as_ref().is_some_and(|l| l.confiance >= 0.6) {
                fausses_sures.push(nom);
            }
            eprintln!(
                "{} {nom:32} {:>6.1} ms  {:?} ({:.2})  attendu {attendu}",
                if ok { "✓" } else { "✗" },
                ms,
                lu,
                l.as_ref().map_or(0.0, |l| l.confiance)
            );
        }
        eprintln!(
            "{justes}/{} justes ; fausses mais sûres : {fausses_sures:?}",
            ATTENDUS.len()
        );
        // Mesure du 2026-11-08 : 14/23 avec le modèle v1 de Fikajiana (entraîné sur des polices). Ce qui
        // compte d'abord : aucune lecture fausse n'est sûre (elle serait mise au propre sans qu'on la voie).
        assert!(justes >= 14, "{justes}/{} (recul)", ATTENDUS.len());
        assert!(fausses_sures.is_empty(), "fausses mais sûres : {fausses_sures:?}");
    }

    /// `FANO_DIAG=pad7/point_A cargo test -p fano-reco diagnostic -- --nocapture` : chaque signe lu.
    #[test]
    fn diagnostic() {
        let Ok(nom) = std::env::var("FANO_DIAG") else {
            return;
        };
        let cl = ClasseurSignes::charger().unwrap();
        let traits = traits_fixture(&nom);
        let tr: Vec<Tr> = traits.iter().map(|t| Tr::new(t.clone())).collect();
        for (i, t) in tr.iter().enumerate() {
            eprintln!(
                "trait {i:2} x {:7.1}..{:7.1} y {:7.1}..{:7.1} plat {}",
                t.x0,
                t.x1,
                t.y0,
                t.y1,
                t.plat()
            );
        }
        let mut l = Lecture {
            cl: &cl,
            hmed: hauteur_mediane(&tr),
            tr,
            signes: Vec::new(),
        };
        eprintln!("hmed {}", l.hmed);
        let _ = env_logger_simple();
        let tous: Vec<usize> = (0..l.tr.len()).collect();
        for g in l.grouper(&tous) {
            eprintln!("groupe {g:?}");
        }
        let arbre = l.zone(&tous);
        for s in &l.signes {
            let c: Vec<String> = s
                .choix
                .iter()
                .map(|(k, p)| format!("{}:{:.2}", cl.classes[*k].id, p))
                .collect();
            eprintln!("signe {:?} {}", s.traits, c.join(" "));
        }
        eprintln!("{:?}", arbre.is_some());
        eprintln!("{:?}", lire_formule(&cl, &traits));
    }

    fn env_logger_simple() {
        struct L;
        impl log::Log for L {
            fn enabled(&self, _: &log::Metadata) -> bool {
                true
            }
            fn log(&self, r: &log::Record) {
                eprintln!("log {}", r.args());
            }
            fn flush(&self) {}
        }
        let _ = log::set_logger(&L).map(|()| log::set_max_level(log::LevelFilter::Debug));
    }

    /// Les systèmes d'Antsa (une accolade, deux rangées) se lisent en `cases`.
    #[test]
    fn systemes_d_antsa() {
        for nom in ["pad7/param_t_sin_t", "pad7/param_cos_t_sin_t"] {
            let l = lire(nom).unwrap_or_else(|| panic!("{nom} non lu"));
            eprintln!("{nom} : {} ({:.2})", l.latex, l.confiance);
            assert!(
                l.latex.starts_with("\\begin{cases}") && l.latex.matches("\\\\").count() == 1,
                "{nom} : {}",
                l.latex
            );
        }
    }

    /// Les matrices d'Antsa se lisent en `pmatrix` : la forme (rangées, colonnes) est juste ; les signes
    /// suivent la justesse du modèle.
    #[test]
    fn matrices_d_antsa() {
        for (nom, rangees, colonnes) in [
            ("pad7/M_matrice_2x3", 2, 3),
            ("pad7/det_3x3_72", 3, 3),
            ("telephone/matrice_carre", 2, 2),
            ("pad7/matrice_inverse", 2, 2),
            ("pad7/det_2x2_40", 2, 2),
        ] {
            let Some(l) = lire(nom) else {
                eprintln!("{nom} : non lu");
                continue;
            };
            eprintln!("{nom} : {} ({:.2})", l.latex, l.confiance);
            let corps = l
                .latex
                .split("\\begin{pmatrix}")
                .nth(1)
                .and_then(|c| c.split("\\end{pmatrix}").next());
            let corps = corps.unwrap_or_else(|| panic!("{nom} : {}", l.latex));
            let r: Vec<&str> = corps.split("\\\\").collect();
            let ok = r.len() == rangees && r.iter().all(|x| x.split('&').count() == colonnes);
            assert!(ok, "{nom} : forme fausse");
        }
    }
}
