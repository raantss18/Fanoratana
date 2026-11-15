#!/usr/bin/env python3
"""Construit le site de Fanoratana (GitHub Pages) dans un dossier.

    tool/build_site.py --releases releases.json --out _site

`releases.json` est la réponse de l'API GitHub (`repos/<dépôt>/releases`). Le numéro, la date, les tailles, les
empreintes SHA-256, les liens de téléchargement et le message de la version viennent de la dernière version publiée
qui a une APK : le site suit l'app sans qu'on y touche. Sans version publiée, la page renvoie vers GitHub.
La vidéo `site/video/demo.<langue>.mp4` (montée par `tool/video_demo.py`) est montrée si elle existe.
"""
import argparse
import html
import json
import re
import shutil
from pathlib import Path

RACINE = Path(__file__).resolve().parent.parent
DEPOT = "raantss18/Fanoratana"
SITE = "https://raantss18.github.io/Fanoratana/"
APK = "fanoratana-arm64-v8a.apk"
LINUX = "fanoratana-linux-x86_64"

TEXTES = {
    "fr": {
        "accroche_courte": "écris ton cours, il se met au propre",
        "description": "Fanoratana : un cahier de cours pour enseignants. Écris au doigt, au stylet ou au clavier ; "
        "l'écriture et les formules se mettent au propre. Libre, hors ligne, pour Android et Linux.",
        "aller_telecharger": "Aller au téléchargement",
        "nav": "Sections",
        "nav_demo": "Démo",
        "nav_maths": "Les maths",
        "nav_telecharger": "Télécharger",
        "autre_nom": "English",
        "titre": "Écris ton cours à la main. Il se met au propre.",
        "chapeau": "Fanoratana est un cahier de cours pour les enseignants de toutes les matières. Tu écris au doigt, "
        "au stylet ou au clavier ; après une pause, ton écriture devient un cours net, formules comprises.",
        "bouton_telecharger": "Télécharger",
        "bouton_demo": "Voir la démo",
        "gratuit": "Gratuit et libre. Sans compte, sans publicité, hors ligne. Android et Linux.",
        "alt_maths": "Un cours sur les fonctions affines mis au propre : définition encadrée, formules et courbes.",
        "demo_titre": "En trois gestes",
        "demo_intro": "Pas de menus à apprendre : la feuille prend tout l'écran, et quatre boutons suffisent.",
        "video_titre": "La démo en vidéo",
        "video_credit": "Deux minutes avec Fanoratana et Fikajiana. Musique : « Sidewalk Shade », Kevin MacLeod "
        "(incompetech.com), licence <a href=\"https://creativecommons.org/licenses/by/4.0/\">CC BY 4.0</a>.",
        "alt_cours": "Une leçon d'histoire : titre souligné, définition encadrée, frise et formes nettes.",
        "etape1_titre": "Écris",
        "etape1": "Titres, phrases, schémas, flèches : écris comme sur du papier. Après une pause, tout est mis au "
        "propre, et un gribouillis sur un mot l'efface.",
        "alt_annoter": "Un sujet en PDF avec une remarque écrite en rouge par-dessus.",
        "etape2_titre": "Annote",
        "etape2": "Ouvre un PDF ou une photo : le document reste dessous, tu écris par-dessus sans jamais l'effacer, "
        "puis tu exportes le tout en PDF.",
        "alt_ranger": "Le panneau Mes cours : niveaux, chapitres et leçons.",
        "etape3_titre": "Range et présente",
        "etape3": "Tes leçons se rangent par niveau et par chapitre. Présente-les en diapositives, ou exporte-les en "
        "PDF, en image ou en Markdown.",
        "maths_titre": "Les maths, vraiment",
        "maths_intro": "Les formules écrites à la main sont lues à leur structure : fractions, racines, limites, "
        "sommes, intégrales, systèmes et matrices.",
        "regle_alt": "La règle des deux dollars : une formule écrite entre deux signes dollar est mise au propre.",
        "maths1": "Une seule règle à retenir : entoure une formule de deux <code>$</code> écrits à la main, elle est "
        "toujours lue comme une formule.",
        "maths2": "Sans <code>$</code>, l'app tranche seule entre une phrase et une formule ; « sur R » devient ℝ.",
        "maths3": "Écris <code>f(x)=2x+1</code> dans un repère : la courbe se trace, comme dans GeoGebra.",
        "fikajiana": "Fanoratana a une sœur : <a href=\"https://raantss18.github.io/Fikajiana/\">Fikajiana</a>, la "
        "feuille de calcul manuscrite qui pose et résout les calculs. Deux outils de maths, la même main.",
        "plus_titre": "Et bien plus",
        "plus1_titre": "Hors ligne",
        "plus1": "La lecture de l'écriture tourne sur l'appareil. Le réseau ne sert qu'aux mises à jour.",
        "plus2_titre": "Tes cours restent à toi",
        "plus2": "Des fichiers simples dans un dossier, sans compte ni nuage imposé.",
        "plus3_titre": "Tableaux et listes",
        "plus3": "Des cases à remplir à la main ou au clavier, des listes à puces et numérotées.",
        "plus4_titre": "Encadrés",
        "plus4": "Définition, À retenir, Exemple, Exercice, Citation : un cours qui se lit d'un coup d'œil.",
        "plus5_titre": "Français et anglais",
        "plus5": "L'interface existe dans les deux langues ; l'écriture se lit en français, anglais et malgache.",
        "plus6_titre": "Mises à jour sûres",
        "plus6": "L'app se met à jour elle-même : chaque version est signée et vérifiée avant d'être installée.",
        "tel_titre": "Télécharger",
        "tel_version": "Version {version}, publiée le {date}.",
        "tel_sans_version": "Les versions sont publiées sur GitHub.",
        "tel_android": "Tablette Android",
        "tel_apk": "Télécharger l'APK",
        "tel_linux": "Linux (x86_64)",
        "tel_binaire": "Télécharger le programme",
        "android1": "Ouvre le fichier téléchargé.",
        "android2": "Autorise l'installation quand Android le demande.",
        "android3": "Les versions suivantes s'installent depuis l'app.",
        "linux1": "Rends le fichier exécutable : <code>chmod +x fanoratana-linux-x86_64</code>",
        "linux2": "Lance-le : <code>./fanoratana-linux-x86_64</code>",
        "linux3": "Pour annoter un PDF : <code>./fanoratana-linux-x86_64 sujet.pdf</code>",
        "verifier": "Vérifier le téléchargement (SHA-256)",
        "verifier_texte": "Compare l'empreinte du fichier reçu (<code>sha256sum fichier</code>) avec celle-ci.",
        "toutes_versions": "Toutes les versions et leurs notes",
        "pied_licence": "Logiciel libre sous licence GPL-3.0-or-later.",
        "pied_autrice": "Antsa Malala RANIRIAMANJAKA",
        "pied_code": "Code source",
        "pied_probleme": "Signaler un problème",
        "mois": ["janvier", "février", "mars", "avril", "mai", "juin", "juillet", "août", "septembre", "octobre",
                 "novembre", "décembre"],
        "mo": "Mo",
    },
    "en": {
        "accroche_courte": "write your lesson, it gets tidied up",
        "description": "Fanoratana: a lesson notebook for teachers. Write with your finger, a stylus or the keyboard; "
        "handwriting and formulas are tidied up. Free software, offline, for Android and Linux.",
        "aller_telecharger": "Skip to download",
        "nav": "Sections",
        "nav_demo": "Demo",
        "nav_maths": "Maths",
        "nav_telecharger": "Download",
        "autre_nom": "Français",
        "titre": "Write your lesson by hand. It gets tidied up.",
        "chapeau": "Fanoratana is a lesson notebook for teachers of every subject. You write with your finger, a "
        "stylus or the keyboard; after a pause, your handwriting becomes a clean lesson, formulas included.",
        "bouton_telecharger": "Download",
        "bouton_demo": "See the demo",
        "gratuit": "Free and open source. No account, no ads, offline. Android and Linux.",
        "alt_maths": "A tidied-up lesson on linear functions: a boxed definition, formulas and graphs.",
        "demo_titre": "In three moves",
        "demo_intro": "No menus to learn: the sheet takes the whole screen, and four buttons are enough.",
        "video_titre": "The demo video",
        "video_credit": "Two minutes with Fanoratana and Fikajiana. Music: “Sidewalk Shade”, Kevin MacLeod "
        "(incompetech.com), licensed under <a href=\"https://creativecommons.org/licenses/by/4.0/\">CC BY 4.0</a>.",
        "alt_cours": "A history lesson: underlined title, boxed definition, timeline and clean shapes.",
        "etape1_titre": "Write",
        "etape1": "Titles, sentences, diagrams, arrows: write as you would on paper. After a pause, everything is "
        "tidied up, and scribbling over a word erases it.",
        "alt_annoter": "A PDF exam paper with a remark written over it in red.",
        "etape2_titre": "Annotate",
        "etape2": "Open a PDF or a photo: the document stays underneath, you write over it without ever erasing it, "
        "then export the whole thing as a PDF.",
        "alt_ranger": "The My courses panel: levels, chapters and lessons.",
        "etape3_titre": "Sort and present",
        "etape3": "Your lessons are sorted by level and chapter. Present them as slides, or export them as PDF, "
        "image or Markdown.",
        "maths_titre": "Maths, for real",
        "maths_intro": "Handwritten formulas are read by their structure: fractions, roots, limits, sums, "
        "integrals, systems and matrices.",
        "regle_alt": "The two-dollar rule: a formula written between two dollar signs is tidied up.",
        "maths1": "One rule to remember: write two <code>$</code> around a formula by hand, and it is always read "
        "as a formula.",
        "maths2": "Without <code>$</code>, the app decides between a sentence and a formula; “on R” becomes ℝ.",
        "maths3": "Write <code>f(x)=2x+1</code> in a graph: the curve is drawn, as in GeoGebra.",
        "fikajiana": "Fanoratana has a sister: <a href=\"https://raantss18.github.io/Fikajiana/\">Fikajiana</a>, the "
        "handwritten calculation sheet that sets out and solves sums. Two maths tools, the same hand.",
        "plus_titre": "And much more",
        "plus1_titre": "Offline",
        "plus1": "Handwriting is read on the device. The network is only used for updates.",
        "plus2_titre": "Your lessons stay yours",
        "plus2": "Plain files in a folder, with no account and no forced cloud.",
        "plus3_titre": "Tables and lists",
        "plus3": "Cells you fill in by hand or with the keyboard, bulleted and numbered lists.",
        "plus4_titre": "Boxes",
        "plus4": "Definition, Remember, Example, Exercise, Quote: a lesson you can read at a glance.",
        "plus5_titre": "French and English",
        "plus5": "The interface comes in both languages; handwriting is read in French, English and Malagasy.",
        "plus6_titre": "Safe updates",
        "plus6": "The app updates itself: every version is signed and checked before it is installed.",
        "tel_titre": "Download",
        "tel_version": "Version {version}, released on {date}.",
        "tel_sans_version": "Releases are published on GitHub.",
        "tel_android": "Android tablet",
        "tel_apk": "Download the APK",
        "tel_linux": "Linux (x86_64)",
        "tel_binaire": "Download the program",
        "android1": "Open the downloaded file.",
        "android2": "Allow the installation when Android asks.",
        "android3": "Later versions install from inside the app.",
        "linux1": "Make the file executable: <code>chmod +x fanoratana-linux-x86_64</code>",
        "linux2": "Run it: <code>./fanoratana-linux-x86_64</code>",
        "linux3": "To annotate a PDF: <code>./fanoratana-linux-x86_64 paper.pdf</code>",
        "verifier": "Check the download (SHA-256)",
        "verifier_texte": "Compare the checksum of the file you received (<code>sha256sum file</code>) with this one.",
        "toutes_versions": "All versions and their notes",
        "pied_licence": "Free software under the GPL-3.0-or-later licence.",
        "pied_autrice": "Antsa Malala RANIRIAMANJAKA",
        "pied_code": "Source code",
        "pied_probleme": "Report a problem",
        "mois": ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
                 "November", "December"],
        "mo": "MB",
    },
}


def derniere_version(releases):
    """La dernière version publiée (ni brouillon ni préversion) qui a une APK, ou None."""
    for r in releases:
        if r.get("draft") or r.get("prerelease"):
            continue
        fichiers = {a["name"]: a for a in r.get("assets", [])}
        if APK in fichiers:
            return r, fichiers
    return None


def notes(corps):
    """Les messages fr et en du corps d'une version (écrit par scripts/release.sh)."""
    corps = corps or ""
    fr = re.search(r"^# .*?\n+(.*?)\n+\*\*Installer\*\*", corps, re.S)
    en = re.search(r"\n---\n+(.*?)\n+<details>", corps, re.S)
    net = lambda m: " ".join(m.group(1).split()) if m else ""
    return {"fr": net(fr), "en": net(en)}


def date(iso, t, langue):
    a, m, j = (int(x) for x in iso[:10].split("-"))
    mois = t["mois"][m - 1]
    return f"{j} {mois} {a}" if langue == "fr" else f"{mois} {j}, {a}"


def taille(octets, t):
    return f"{octets / 1e6:.0f} {t['mo']}".replace(".", "," if t["mo"] == "Mo" else ".")


def page(modele, langue, version, video):
    t = dict(TEXTES[langue])
    autre = "en" if langue == "fr" else "fr"
    v = {
        "lang": langue,
        "autre_lang": autre,
        "racine": "" if langue == "fr" else "../",
        "racine_langue": "./",
        "autre_url": "en/" if langue == "fr" else "../",
        "site": SITE,
        "versions_url": f"https://github.com/{DEPOT}/releases",
        "apk_url": f"https://github.com/{DEPOT}/releases/latest",
        "linux_url": f"https://github.com/{DEPOT}/releases/latest",
        "apk_taille": "",
        "linux_taille": "",
        "apk_sha": "—",
        "linux_sha": "—",
        "notes": "",
        "tel_version": t["tel_sans_version"],
        "video": "",
    }
    if version:
        r, fichiers = version
        numero = r["tag_name"].lstrip("v")
        v["tel_version"] = t["tel_version"].format(
            version=html.escape(numero), date=date(r["published_at"], t, langue)
        )
        v["notes"] = html.escape(notes(r.get("body"))[langue])
        for cle, nom in (("apk", APK), ("linux", LINUX)):
            f = fichiers.get(nom)
            if not f:
                continue
            v[f"{cle}_url"] = html.escape(f["browser_download_url"], quote=True)
            v[f"{cle}_taille"] = taille(f["size"], t)
            empreinte = (f.get("digest") or "").removeprefix("sha256:")
            v[f"{cle}_sha"] = html.escape(empreinte) or "—"
    if video:
        racine = v["racine"]
        v["video"] = (
            f'  <figure class="video"><video controls preload="metadata" playsinline '
            f'poster="{racine}video/affiche.{langue}.jpg" aria-label="{html.escape(t["video_titre"], quote=True)}">'
            f'<source src="{racine}video/demo.{langue}.mp4" type="video/mp4"></video>'
            f'<figcaption>{t["video_credit"]}</figcaption></figure>'
        )
    t.update(v)
    sortie = re.sub(r"\{\{(\w+)\}\}", lambda m: str(t[m.group(1)]), modele)
    return sortie


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--releases", type=Path, help="réponse de l'API GitHub ; sans elle, page sans version")
    p.add_argument("--out", type=Path, required=True)
    a = p.parse_args()
    releases = json.loads(a.releases.read_text(encoding="utf-8")) if a.releases else []
    version = derniere_version(releases if isinstance(releases, list) else [])
    source = RACINE / "site"
    modele = (source / "modele.html").read_text(encoding="utf-8")
    if a.out.exists():
        shutil.rmtree(a.out)
    shutil.copytree(source, a.out, ignore=shutil.ignore_patterns("modele.html", "sources", "LISEZMOI.md"))
    # Les polices et l'icône viennent de l'app : une seule source.
    (a.out / "polices").mkdir(exist_ok=True)
    for police in ("Caveat.ttf", "Nunito.ttf", "OFL-Caveat.txt", "OFL-Nunito.txt"):
        shutil.copy(RACINE / "assets" / "polices" / police, a.out / "polices" / police)
    (a.out / "img").mkdir(exist_ok=True)
    shutil.copy(RACINE / "brand" / "icone.svg", a.out / "img" / "icone.svg")
    video = lambda langue: (source / "video" / f"demo.{langue}.mp4").exists()
    (a.out / "index.html").write_text(page(modele, "fr", version, video("fr")), encoding="utf-8")
    (a.out / "en").mkdir(exist_ok=True)
    (a.out / "en" / "index.html").write_text(page(modele, "en", version, video("en")), encoding="utf-8")
    (a.out / ".nojekyll").write_text("", encoding="utf-8")
    print(f"site construit dans {a.out} — version {version[0]['tag_name'] if version else 'aucune'}")


if __name__ == "__main__":
    main()
