//! Le coffre de cours : un dossier ordinaire, façon Obsidian.
//!
//! ```text
//! <coffre>/
//!   01 Seconde/                 un niveau
//!     01 Fonctions/             un chapitre
//!       01 Généralités.fano     une leçon (JSON)
//!   medias/                     images insérées
//!   .corbeille/                 ce qui a été supprimé (on peut le récupérer à la main)
//! ```
//!
//! Le numéro devant le nom donne l'ordre ; il n'est pas montré. Les écritures sont atomiques
//! (fichier temporaire puis renommage) : une coupure ne laisse jamais une leçon à moitié écrite.

use std::fs;
use std::path::{Path, PathBuf};

use crate::page::{Format, Lecon};

pub const EXTENSION: &str = "fano";
pub const MEDIAS: &str = "medias";
pub const CORBEILLE: &str = ".corbeille";

/// Les niveaux proposés par défaut (ADR 0004) ; chacun les renomme ou les supprime.
pub const NIVEAUX_TYPE: [&str; 13] = [
    "6e",
    "5e",
    "4e",
    "3e",
    "Seconde",
    "Première",
    "Terminale",
    "L1",
    "L2",
    "L3",
    "M1",
    "M2",
    "Formation",
];

#[derive(Debug)]
pub enum Erreur {
    Io(std::io::Error),
    Json(serde_json::Error),
    NomVide,
    Existe(PathBuf),
}

impl std::fmt::Display for Erreur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Erreur::Io(e) => write!(f, "{e}"),
            Erreur::Json(e) => write!(f, "fichier de leçon illisible : {e}"),
            Erreur::NomVide => write!(f, "le nom est vide"),
            Erreur::Existe(p) => write!(f, "« {} » existe déjà", p.display()),
        }
    }
}

impl std::error::Error for Erreur {}

impl From<std::io::Error> for Erreur {
    fn from(e: std::io::Error) -> Self {
        Erreur::Io(e)
    }
}
impl From<serde_json::Error> for Erreur {
    fn from(e: serde_json::Error) -> Self {
        Erreur::Json(e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Genre {
    Niveau,
    Chapitre,
    Lecon,
}

/// Une entrée de l'arbre du coffre.
#[derive(Clone, Debug, PartialEq)]
pub struct Entree {
    pub genre: Genre,
    /// Le nom montré, sans numéro ni extension.
    pub nom: String,
    pub chemin: PathBuf,
    pub enfants: Vec<Entree>,
}

#[derive(Clone, Debug)]
pub struct Coffre {
    pub racine: PathBuf,
}

/// Le nom montré : sans le numéro d'ordre « 01 » ni l'extension.
pub fn nom_montre(chemin: &Path) -> String {
    let base = chemin
        .file_stem()
        .or_else(|| chemin.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let base = if chemin.extension().is_some_and(|e| e == EXTENSION) {
        base
    } else {
        chemin
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    match base.split_once(' ') {
        Some((num, reste)) if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) => reste.to_owned(),
        _ => base,
    }
}

fn numero(chemin: &Path) -> Option<u32> {
    chemin.file_name()?.to_str()?.split_once(' ')?.0.parse().ok()
}

/// Un nom de fichier sûr : ni séparateur, ni caractère réservé, ni point en tête.
pub fn nom_sur(nom: &str) -> String {
    let s: String = nom
        .trim()
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '-'
            } else {
                c
            }
        })
        .collect();
    s.trim_start_matches('.').chars().take(80).collect()
}

fn trie(mut v: Vec<PathBuf>) -> Vec<PathBuf> {
    v.sort_by(|a, b| {
        numero(a)
            .unwrap_or(u32::MAX)
            .cmp(&numero(b).unwrap_or(u32::MAX))
            .then_with(|| a.cmp(b))
    });
    v
}

fn sous_dossiers(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    trie(
        rd.filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| !n.starts_with('.') && n != MEDIAS)
            })
            .collect(),
    )
}

fn lecons(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    trie(
        rd.filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == EXTENSION))
            .collect(),
    )
}

/// Écrit `contenu` dans `chemin` sans jamais laisser un fichier à moitié écrit.
pub fn ecrire_atomique(chemin: &Path, contenu: &[u8]) -> std::io::Result<()> {
    let tmp = chemin.with_extension("tmp~");
    fs::write(&tmp, contenu)?;
    fs::rename(&tmp, chemin)
}

impl Coffre {
    /// Ouvre (et crée au besoin) le coffre. Un coffre neuf reçoit une première leçon d'accueil.
    pub fn ouvrir(racine: impl Into<PathBuf>) -> Result<Coffre, Erreur> {
        let racine = racine.into();
        let neuf = !racine.exists() || fs::read_dir(&racine)?.next().is_none();
        fs::create_dir_all(racine.join(MEDIAS))?;
        let c = Coffre { racine };
        if neuf {
            let niveau = c.creer_niveau("Premiers pas")?;
            let chap = c.creer_chapitre(&niveau, "Découvrir Fanoratana")?;
            c.creer_lecon(&chap, "Ma première leçon", Format::A4)?;
        }
        Ok(c)
    }

    /// L'arbre complet : niveaux → chapitres → leçons.
    pub fn arbre(&self) -> Vec<Entree> {
        sous_dossiers(&self.racine)
            .into_iter()
            .map(|n| Entree {
                genre: Genre::Niveau,
                nom: nom_montre(&n),
                enfants: sous_dossiers(&n)
                    .into_iter()
                    .map(|c| Entree {
                        genre: Genre::Chapitre,
                        nom: nom_montre(&c),
                        enfants: lecons(&c)
                            .into_iter()
                            .map(|l| Entree {
                                genre: Genre::Lecon,
                                nom: nom_montre(&l),
                                chemin: l,
                                enfants: vec![],
                            })
                            .collect(),
                        chemin: c,
                    })
                    .collect(),
                chemin: n,
            })
            .collect()
    }

    fn prochain(dir: &Path) -> u32 {
        let Ok(rd) = fs::read_dir(dir) else { return 1 };
        rd.filter_map(Result::ok)
            .filter_map(|e| numero(&e.path()))
            .max()
            .unwrap_or(0)
            + 1
    }

    fn creer_dossier(&self, parent: &Path, nom: &str) -> Result<PathBuf, Erreur> {
        let nom = nom_sur(nom);
        if nom.is_empty() {
            return Err(Erreur::NomVide);
        }
        let p = parent.join(format!("{:02} {nom}", Self::prochain(parent)));
        fs::create_dir_all(&p)?;
        Ok(p)
    }

    pub fn creer_niveau(&self, nom: &str) -> Result<PathBuf, Erreur> {
        self.creer_dossier(&self.racine, nom)
    }

    /// Ajoute les niveaux type absents (6e … M2).
    pub fn creer_niveaux_type(&self) -> Result<(), Erreur> {
        let existants: Vec<String> = sous_dossiers(&self.racine)
            .iter()
            .map(|p| nom_montre(p))
            .collect();
        for n in NIVEAUX_TYPE {
            if !existants.iter().any(|e| e == n) {
                self.creer_niveau(n)?;
            }
        }
        Ok(())
    }

    pub fn creer_chapitre(&self, niveau: &Path, nom: &str) -> Result<PathBuf, Erreur> {
        self.creer_dossier(niveau, nom)
    }

    pub fn creer_lecon(&self, chapitre: &Path, titre: &str, format: Format) -> Result<PathBuf, Erreur> {
        let nom = nom_sur(titre);
        if nom.is_empty() {
            return Err(Erreur::NomVide);
        }
        let p = chapitre.join(format!("{:02} {nom}.{EXTENSION}", Self::prochain(chapitre)));
        if p.exists() {
            return Err(Erreur::Existe(p));
        }
        self.ecrire_lecon(&p, &Lecon::nouvelle(titre.trim(), format))?;
        Ok(p)
    }

    pub fn lire_lecon(&self, chemin: &Path) -> Result<Lecon, Erreur> {
        Ok(Lecon::depuis_json(&fs::read_to_string(chemin)?)?)
    }

    pub fn ecrire_lecon(&self, chemin: &Path, lecon: &Lecon) -> Result<(), Erreur> {
        ecrire_atomique(chemin, lecon.vers_json().as_bytes())?;
        Ok(())
    }

    /// Renomme en gardant le numéro d'ordre. Rend le nouveau chemin.
    pub fn renommer(&self, chemin: &Path, nouveau: &str) -> Result<PathBuf, Erreur> {
        let nom = nom_sur(nouveau);
        if nom.is_empty() {
            return Err(Erreur::NomVide);
        }
        let num = numero(chemin).unwrap_or(0);
        let mut fichier = format!("{num:02} {nom}");
        if chemin.is_file() {
            fichier.push('.');
            fichier.push_str(EXTENSION);
        }
        let dest = chemin.with_file_name(fichier);
        if dest != chemin && dest.exists() {
            return Err(Erreur::Existe(dest));
        }
        fs::rename(chemin, &dest)?;
        Ok(dest)
    }

    /// Met à la corbeille du coffre (rien n'est effacé pour de bon).
    pub fn supprimer(&self, chemin: &Path) -> Result<(), Erreur> {
        let corbeille = self.racine.join(CORBEILLE);
        fs::create_dir_all(&corbeille)?;
        let nom = chemin
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut dest = corbeille.join(&nom);
        let mut i = 1;
        while dest.exists() {
            dest = corbeille.join(format!("{nom} ({i})"));
            i += 1;
        }
        fs::rename(chemin, dest)?;
        Ok(())
    }

    /// Copie une image dans `medias/` et rend son chemin relatif au coffre.
    pub fn importer_media(&self, nom: &str, octets: &[u8]) -> Result<String, Erreur> {
        let dir = self.racine.join(MEDIAS);
        fs::create_dir_all(&dir)?;
        let nom = nom_sur(nom);
        let (base, ext) = match nom.rsplit_once('.') {
            Some((b, e)) => (b.to_owned(), format!(".{}", e.to_lowercase())),
            None => (nom.clone(), String::new()),
        };
        let mut fichier = format!("{base}{ext}");
        let mut i = 1;
        while dir.join(&fichier).exists() {
            fichier = format!("{base}-{i}{ext}");
            i += 1;
        }
        fs::write(dir.join(&fichier), octets)?;
        Ok(format!("{MEDIAS}/{fichier}"))
    }

    /// Les leçons dont le titre ou le texte au propre contient `requete` (sans casse).
    pub fn chercher(&self, requete: &str) -> Vec<(PathBuf, String)> {
        let q = requete.to_lowercase();
        if q.trim().is_empty() {
            return Vec::new();
        }
        let mut res = Vec::new();
        for n in self.arbre() {
            for c in n.enfants {
                for l in c.enfants {
                    let titre_ok = l.nom.to_lowercase().contains(&q);
                    let texte_ok = self
                        .lire_lecon(&l.chemin)
                        .is_ok_and(|lec| lec.markdown().to_lowercase().contains(&q));
                    if titre_ok || texte_ok {
                        res.push((l.chemin.clone(), format!("{} › {} › {}", n.nom, c.nom, l.nom)));
                    }
                }
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coffre_neuf_et_ordre() {
        let dir = tempfile::tempdir().unwrap();
        let c = Coffre::ouvrir(dir.path().join("cours")).unwrap();
        let a = c.arbre();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].nom, "Premiers pas");
        assert_eq!(a[0].enfants[0].enfants[0].nom, "Ma première leçon");

        let n = c.creer_niveau("Seconde").unwrap();
        let ch = c.creer_chapitre(&n, "Fonctions").unwrap();
        let l1 = c.creer_lecon(&ch, "Généralités", Format::A4).unwrap();
        let l2 = c.creer_lecon(&ch, "Variations", Format::Diapo).unwrap();
        assert!(l1.ends_with("01 Généralités.fano"));
        assert!(l2.ends_with("02 Variations.fano"));
        let a = c.arbre();
        assert_eq!(a[1].nom, "Seconde");
        assert_eq!(
            a[1].enfants[0]
                .enfants
                .iter()
                .map(|e| e.nom.as_str())
                .collect::<Vec<_>>(),
            ["Généralités", "Variations"]
        );

        let l1b = c.renommer(&l1, "Définitions").unwrap();
        assert!(l1b.ends_with("01 Définitions.fano"));
        assert_eq!(c.lire_lecon(&l1b).unwrap().titre, "Généralités");
        c.supprimer(&l2).unwrap();
        assert!(c.racine.join(CORBEILLE).join("02 Variations.fano").exists());
        assert_eq!(c.chercher("défin").len(), 1);
    }

    #[test]
    fn noms_surs_et_medias() {
        assert_eq!(nom_sur("../a/b:c"), "-a-b-c");
        assert_eq!(nom_montre(Path::new("/x/03 Les angles.fano")), "Les angles");
        assert_eq!(nom_montre(Path::new("/x/12 Seconde")), "Seconde");
        assert_eq!(nom_montre(Path::new("/x/Libre")), "Libre");
        let dir = tempfile::tempdir().unwrap();
        let c = Coffre::ouvrir(dir.path()).unwrap();
        assert_eq!(c.importer_media("Fig.PNG", b"x").unwrap(), "medias/Fig.png");
        assert_eq!(c.importer_media("Fig.png", b"y").unwrap(), "medias/Fig-1.png");
        c.creer_niveaux_type().unwrap();
        c.creer_niveaux_type().unwrap();
        assert_eq!(c.arbre().len(), 1 + NIVEAUX_TYPE.len());
    }
}
