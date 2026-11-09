//! Les mises à jour, sans quitter l'app (ADR 0008).
//!
//! 1. `release.json` et sa signature `release.json.sig` sont lus à une adresse fixe (pièces jointes
//!    de la dernière version publiée) ; la signature Ed25519 est vérifiée avec la clé publique
//!    embarquée : un fichier modifié en route est refusé.
//! 2. Si la version est plus récente, le fichier de la plateforme est téléchargé, puis son SHA-256
//!    est comparé à celui de `release.json` (signé).
//! 3. L'app l'installe : l'installeur d'Android pour l'APK ; sur Linux, le programme est remplacé et
//!    prend effet au prochain lancement.
//!
//! Le réseau ne sert qu'à cela, et seulement à la demande ou une fois par jour (réglage).

use std::io::Read;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Où sont les fichiers de la dernière version.
pub const ADRESSE: &str = "https://github.com/raantss18/Fanoratana/releases/latest/download";
/// Les mêmes fichiers par l'API de GitHub : le secours quand l'adresse directe répond 5xx (504 relevés le
/// 2026-11-09 sur `releases/download/…`, alors que l'API servait les fichiers).
const API: &str = "https://api.github.com/repos/raantss18/Fanoratana/releases";

type Reponse = ureq::http::Response<ureq::Body>;
/// La clé publique des mises à jour (la privée reste dans `release-keys/`, jamais publiée).
pub const CLE_PUBLIQUE: &str = include_str!("../../../brand/maj-ed25519.pub");

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fichier {
    pub nom: String,
    pub sha256: String,
    pub taille: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    /// Ce que la version permet de faire (2 à 4 lignes).
    pub notes_fr: String,
    pub notes_en: String,
    /// Par plateforme : `android-arm64`, `linux-x86_64`.
    pub fichiers: std::collections::BTreeMap<String, Fichier>,
}

/// La plateforme de ce programme.
pub fn plateforme() -> &'static str {
    if cfg!(target_os = "android") {
        "android-arm64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x86_64"
    } else {
        "autre"
    }
}

/// `a` est-elle plus récente que `b` ? (chiffres séparés par des points)
pub fn plus_recente(a: &str, b: &str) -> bool {
    let n = |s: &str| {
        s.trim_start_matches('v')
            .split('.')
            .map(|x| x.parse::<u32>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    n(a) > n(b)
}

/// Vérifie la signature de `contenu` (base64 dans `signature`).
pub fn verifier(contenu: &[u8], signature: &str) -> Result<(), String> {
    let cle = base64::engine::general_purpose::STANDARD
        .decode(CLE_PUBLIQUE.trim())
        .map_err(|e| e.to_string())?;
    let cle: [u8; 32] = cle.try_into().map_err(|_| "clé publique invalide")?;
    let cle = VerifyingKey::from_bytes(&cle).map_err(|e| e.to_string())?;
    let sig = base64::engine::general_purpose::STANDARD
        .decode(signature.trim())
        .map_err(|e| e.to_string())?;
    let sig: [u8; 64] = sig.try_into().map_err(|_| "signature invalide")?;
    cle.verify(contenu, &Signature::from_bytes(&sig))
        .map_err(|_| "signature refusée : fichier modifié".to_owned())
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // Pas de délai global : un modèle de 117 Mo sur une connexion lente prend plusieurs minutes
        // (« timeout: global » relevé sur le Pad 7, 2026-11-08). On coupe seulement une connexion
        // muette : 20 s pour se connecter, 60 s sans aucun octet reçu.
        .timeout_global(None)
        .timeout_connect(Some(std::time::Duration::from_secs(20)))
        .timeout_per_read(Some(std::time::Duration::from_secs(60)))
        .user_agent(concat!("Fanoratana/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// Une requête GET, réessayée deux fois quand le serveur répond 5xx ou que la connexion tombe.
fn get(url: &str, binaire: bool) -> Result<Reponse, String> {
    let mut essai = 0;
    loop {
        let mut req = agent().get(url);
        if binaire {
            req = req.header("Accept", "application/octet-stream");
        }
        match req.call() {
            Ok(r) => return Ok(r),
            Err(ureq::Error::StatusCode(c)) if c < 500 || essai >= 2 => {
                return Err(format!("http status: {c}"));
            }
            Err(e) if essai >= 2 => return Err(e.to_string()),
            Err(_) => {
                essai += 1;
                std::thread::sleep(std::time::Duration::from_secs(2 * essai));
            }
        }
    }
}

/// Le fichier `nom` d'une version publiée (`None` : la dernière) : l'adresse directe, sinon l'API.
fn fichier_publie(version: Option<&str>, nom: &str) -> Result<Reponse, String> {
    let direct = match version {
        Some(v) => format!("https://github.com/raantss18/Fanoratana/releases/download/v{v}/{nom}"),
        None => format!("{ADRESSE}/{nom}"),
    };
    match get(&direct, false) {
        Ok(r) => Ok(r),
        // Pas publié : l'API n'en dira pas plus.
        Err(e) if e.contains("404") => Err(e),
        Err(e) => par_api(version, nom).map_err(|e2| format!("{e} ; API : {e2}")),
    }
}

fn par_api(version: Option<&str>, nom: &str) -> Result<Reponse, String> {
    let url = match version {
        Some(v) => format!("{API}/tags/v{v}"),
        None => format!("{API}/latest"),
    };
    let mut rep = get(&url, false)?;
    let mut json = Vec::new();
    rep.body_mut()
        .as_reader()
        .take(1 << 20)
        .read_to_end(&mut json)
        .map_err(|e| e.to_string())?;
    let id = id_du_fichier(&json, nom).ok_or_else(|| format!("{nom} absent de la version"))?;
    get(&format!("{API}/assets/{id}"), true)
}

/// L'identifiant du fichier `nom` dans la réponse de l'API (`assets[].name`, `assets[].id`).
fn id_du_fichier(json: &[u8], nom: &str) -> Option<u64> {
    let v: serde_json::Value = serde_json::from_slice(json).ok()?;
    v["assets"].as_array()?.iter().find(|a| a["name"] == nom)?["id"].as_u64()
}

fn lire(rep: Result<Reponse, String>, max: u64) -> Result<Vec<u8>, String> {
    let mut rep = rep?;
    let mut v = Vec::new();
    rep.body_mut()
        .as_reader()
        .take(max)
        .read_to_end(&mut v)
        .map_err(|e| e.to_string())?;
    Ok(v)
}

/// Lit et vérifie la dernière version publiée. `Ok(None)` : on est à jour.
pub fn chercher(actuelle: &str) -> Result<Option<Version>, String> {
    // Aucune version publiée (404) : rien à installer, ce n'est pas une erreur.
    let json = match lire(fichier_publie(None, "release.json"), 1 << 20) {
        Err(e) if e.contains("404") => return Ok(None),
        r => r?,
    };
    let sig = lire(fichier_publie(None, "release.json.sig"), 4096)?;
    verifier(&json, &String::from_utf8_lossy(&sig))?;
    let v: Version = serde_json::from_slice(&json).map_err(|e| e.to_string())?;
    Ok((plus_recente(&v.version, actuelle) && v.fichiers.contains_key(plateforme())).then_some(v))
}

/// Télécharge le fichier de cette plateforme dans `dossier`, vérifie son SHA-256, rend son chemin.
pub fn telecharger(v: &Version, dossier: &Path, progres: impl Fn(f32)) -> Result<PathBuf, String> {
    let f = v.fichiers.get(plateforme()).ok_or("rien pour cette plateforme")?;
    std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    let mut rep = fichier_publie(Some(&v.version), &f.nom)?;
    let mut lecteur = rep.body_mut().with_config().limit(f.taille + 1024).reader();
    let dest = dossier.join(&f.nom);
    let mut sortie = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut lu = 0u64;
    loop {
        let n = lecteur.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        std::io::Write::write_all(&mut sortie, &buf[..n]).map_err(|e| e.to_string())?;
        lu += n as u64;
        progres((lu as f32 / f.taille.max(1) as f32).min(1.0));
    }
    let somme: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if somme != f.sha256.to_lowercase() {
        let _ = std::fs::remove_file(&dest);
        return Err("le fichier téléchargé ne correspond pas : refusé".into());
    }
    Ok(dest)
}

/// Télécharge `url` dans `dest`, vérifie la taille et le SHA-256 (sinon le fichier est effacé).
pub fn telecharger_fichier(
    url: &str,
    dest: &Path,
    taille: u64,
    sha256: &str,
    progres: impl Fn(u64),
) -> Result<(), String> {
    if let Some(d) = dest.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut rep = get(url, false)?;
    let mut lecteur = rep.body_mut().with_config().limit(taille + 1024).reader();
    let tmp = dest.with_extension("partiel");
    let mut sortie = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut lu = 0u64;
    loop {
        let n = lecteur.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        std::io::Write::write_all(&mut sortie, &buf[..n]).map_err(|e| e.to_string())?;
        lu += n as u64;
        progres(lu);
    }
    let somme: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if lu != taille || somme != sha256.to_lowercase() {
        let _ = std::fs::remove_file(&tmp);
        return Err("le fichier téléchargé ne correspond pas : refusé".into());
    }
    std::fs::rename(&tmp, dest).map_err(|e| e.to_string())
}

/// Linux : remplace le programme en cours par `nouveau` (pris en compte au prochain lancement).
pub fn remplacer_programme(nouveau: &Path) -> Result<PathBuf, String> {
    let actuel = std::env::current_exe().map_err(|e| e.to_string())?;
    let tmp = actuel.with_extension("nouveau");
    std::fs::copy(nouveau, &tmp).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, &actuel).map_err(|e| e.to_string())?;
    Ok(actuel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(plus_recente("0.0.2", "0.0.1"));
        assert!(plus_recente("0.1.0", "0.0.9"));
        assert!(plus_recente("1.0.0", "0.9.9"));
        assert!(!plus_recente("0.0.1", "0.0.1"));
        assert!(plus_recente("v0.0.10", "0.0.9"));
    }

    #[test]
    fn fichier_trouve_dans_la_reponse_de_l_api() {
        let json = br#"{"tag_name":"v0.0.2","assets":[{"id":7,"name":"release.json"},{"id":726274269,"name":"fanoratana-arm64-v8a.apk"}]}"#;
        assert_eq!(id_du_fichier(json, "fanoratana-arm64-v8a.apk"), Some(726274269));
        assert_eq!(id_du_fichier(json, "absent"), None);
        assert_eq!(id_du_fichier(b"pas du json", "release.json"), None);
    }

    /// Le secours par l'API sur la vraie version publiée (`FANO_RESEAU=1`, réseau requis).
    #[test]
    fn secours_par_l_api() {
        if std::env::var("FANO_RESEAU").is_err() {
            return;
        }
        let json = lire(par_api(None, "release.json"), 1 << 20).unwrap();
        let sig = lire(par_api(None, "release.json.sig"), 4096).unwrap();
        verifier(&json, &String::from_utf8_lossy(&sig)).unwrap();
        let v: Version = serde_json::from_slice(&json).unwrap();
        let mut rep = par_api(Some(&v.version), "fanoratana.svg").unwrap();
        let mut svg = Vec::new();
        rep.body_mut().as_reader().read_to_end(&mut svg).unwrap();
        assert!(String::from_utf8_lossy(&svg).contains("<svg"));
    }

    #[test]
    fn cle_embarquee_valide() {
        let cle = base64::engine::general_purpose::STANDARD
            .decode(CLE_PUBLIQUE.trim())
            .unwrap();
        assert_eq!(cle.len(), 32);
        assert!(verifier(b"x", &base64::engine::general_purpose::STANDARD.encode([0u8; 64])).is_err());
    }
}

#[cfg(test)]
mod test_release {
    /// Vérifie la release préparée par `scripts/release.sh` (signature openssl → vérification Rust).
    #[test]
    fn signature_de_la_release() {
        let Ok(d) = std::env::var("FANO_RELEASE") else {
            return;
        };
        let json = std::fs::read(format!("{d}/release.json")).unwrap();
        let sig = std::fs::read_to_string(format!("{d}/release.json.sig")).unwrap();
        super::verifier(&json, &sig).expect("la signature de release.json doit être acceptée par l'app");
        let v: super::Version = serde_json::from_slice(&json).unwrap();
        assert_eq!(v.version, env!("CARGO_PKG_VERSION"));
        let mut faux = json.clone();
        faux[10] ^= 1;
        assert!(
            super::verifier(&faux, &sig).is_err(),
            "un fichier modifié doit être refusé"
        );
    }
}
