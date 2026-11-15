#!/usr/bin/env python3
"""Monte la vidéo de démonstration (2 min) de Fanoratana et Fikajiana, en français et en anglais.

    tool/video_demo.py            → site/video/demo.fr.mp4, site/video/demo.en.mp4

Sources, dans `site/video/sources/` (hors de Git : trop lourdes, voir `site/video/LISEZMOI.md`) :
- `a.mp4` … `f.mp4`, `r.mp4` : écran de la Pad 7 (1600 × 1068) pendant le rejeu des traits (`tool/essais/fil.py`) ;
- `fikajiana.mp4` : la démonstration de Fikajiana (`Fikajiana/site/video/demo.mp4`), dont on garde l'écran sans le bandeau ;
- `sidewalk-shade.mp3` : « Sidewalk Shade », Kevin MacLeod (incompetech.com), CC BY 4.0.

Image finale : 1280 × 920 (l'écran en 1280 × 800, le texte dans un bandeau noir de 120 dessous), 30 images/s.
"""
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

RACINE = Path(__file__).resolve().parent.parent
SRC = RACINE / "site" / "video" / "sources"
SORTIE = RACINE / "site" / "video"
CAVEAT = str(RACINE / "assets" / "polices" / "Caveat.ttf")
NUNITO = str(RACINE / "assets" / "polices" / "Nunito.ttf")
L, H, BANDE = 1280, 920, 120
IPS = 30

# (source, début, fin, vitesse, texte fr, texte en). Source None : un carton (titre fr, titre en, sous-titre fr, en).
PLAN = [
    ("carton", 4.5, ("Deux outils de maths.\nLa même main.", "Two maths tools.\nThe same hand."),
     ("Fanoratana et Fikajiana", "Fanoratana and Fikajiana")),
    ("a.mp4", 1.5, 44.5, 3.0, "Fanoratana : écris ton cours à la main…", "Fanoratana: write your lesson by hand…"),
    ("b.mp4", 1.5, 64.0, 4.0, "… après une pause, il se met au propre. « sur R » devient l'ensemble des réels.",
     "… after a pause, it is tidied up. “on R” becomes the set of real numbers."),
    ("c.mp4", 1.5, 27.5, 2.5, "Les formules sont lues et composées, comme dans un livre.",
     "Formulas are read and typeset, as in a book."),
    ("d.mp4", 1.0, 6.6, 1.0, "Un triangle à main levée devient une figure nette.",
     "A freehand triangle becomes a clean figure."),
    ("e.mp4", 1.5, 21.5, 3.6, "Une erreur ?", "A mistake?"),
    ("r.mp4", 0.6, 4.4, 1.0, "Un gribouillis dessus, et elle disparaît.", "Scribble over it, and it is gone."),
    ("f.mp4", 1.5, 19.5, 1.9, "Ouvre un PDF et écris par-dessus : le document ne s'efface jamais.",
     "Open a PDF and write over it: the document is never erased."),
    ("carton", 3.5, ("Et pour calculer :\nFikajiana", "And to calculate:\nFikajiana"),
     ("Écris un calcul, la réponse s'écrit à côté.", "Write a calculation, the answer appears beside it.")),
    ("fikajiana.mp4", 27.0, 72.0, 1.4, "Équation, limite, déterminant : chaque ligne est lue sur l'appareil.",
     "Equation, limit, determinant: every line is read on the device."),
    ("fikajiana.mp4", 112.5, 117.5, 1.0, "Une fonction se trace d'un toucher.", "A function is plotted with one tap."),
    ("carton", 12.0, ("Libres. Hors ligne.\nSans compte, sans publicité.", "Free software. Offline.\nNo account, no ads."),
     ("raantss18.github.io/Fanoratana  ·  raantss18.github.io/Fikajiana\nMusique : « Sidewalk Shade », Kevin MacLeod (incompetech.com), CC BY 4.0",
      "raantss18.github.io/Fanoratana  ·  raantss18.github.io/Fikajiana\nMusic: “Sidewalk Shade”, Kevin MacLeod (incompetech.com), CC BY 4.0")),
]


def ffmpeg(*args):
    subprocess.run(["ffmpeg", "-v", "error", "-y", *map(str, args)], check=True)


def bandeau(texte, chemin):
    """Le bandeau de texte (1280 × 120), noir, texte blanc centré."""
    img = Image.new("RGB", (L, BANDE), "black")
    d = ImageDraw.Draw(img)
    police = ImageFont.truetype(NUNITO, 30)
    try:
        police.set_variation_by_axes([700])
    except Exception:
        pass
    d.multiline_text((L / 2, BANDE / 2), texte, font=police, fill="white", anchor="mm", align="center", spacing=8)
    img.save(chemin)


def carton(titre, sous_titre, chemin):
    """Un carton plein écran : papier ligné, titre en Caveat, sous-titre en Nunito."""
    img = Image.new("RGB", (L, H), "white")
    d = ImageDraw.Draw(img)
    for y in range(120, H, 48):
        d.line([(0, y), (L, y)], fill=(232, 232, 232), width=1)
    d.line([(150, 0), (150, H)], fill=(214, 214, 214), width=2)
    grand = ImageFont.truetype(CAVEAT, 118)
    try:
        grand.set_variation_by_axes([700])
    except Exception:
        pass
    petit = ImageFont.truetype(NUNITO, 28)
    try:
        petit.set_variation_by_axes([650])
    except Exception:
        pass
    lignes_sous = sous_titre.count("\n") + 1
    d.multiline_text((L / 2, H / 2 - 60), titre, font=grand, fill="black", anchor="mm", align="center", spacing=4)
    d.multiline_text((L / 2, H - 150 - 20 * lignes_sous), sous_titre, font=petit, fill=(30, 30, 30), anchor="mm",
                     align="center", spacing=12)
    img.save(chemin)


def segment(i, piece, langue, tmp):
    """Rend le segment i en mp4 muet 1280 × 920 ; rend son chemin."""
    sortie = tmp / f"{langue}_{i:02d}.mp4"
    k = 0 if langue == "fr" else 1
    if piece[0] == "carton":
        _, duree, titres, sous = piece
        png = tmp / f"{langue}_{i:02d}.png"
        carton(titres[k], sous[k], png)
        ffmpeg("-loop", "1", "-t", duree, "-i", png, "-vf", f"fps={IPS},fade=t=in:d=0.4,fade=t=out:st={duree - 0.4}:d=0.4,format=yuv420p",
               "-c:v", "libx264", "-crf", "20", "-preset", "medium", sortie)
        return sortie
    source, debut, fin, vitesse = piece[:4]
    texte = piece[4 + k]
    png = tmp / f"{langue}_{i:02d}.png"
    bandeau(texte, png)
    # Fanoratana : 1600 × 1068, on retire la barre d'état (46 px) ; Fikajiana : 1280 × 920, on retire son bandeau.
    cadre = "crop=1280:800:0:0" if source == "fikajiana.mp4" else "crop=1600:1000:0:46,scale=1280:800:flags=lanczos"
    filtre = (f"[0:v]trim={debut}:{fin},setpts=(PTS-STARTPTS)/{vitesse},{cadre},fps={IPS}[v];"
              f"[v][1:v]vstack=inputs=2:shortest=1,format=yuv420p[o]")
    ffmpeg("-i", SRC / source, "-loop", "1", "-i", png, "-filter_complex", filtre, "-map", "[o]", "-shortest",
           "-an", "-c:v", "libx264", "-crf", "20", "-preset", "medium", sortie)
    return sortie


def monter(langue, tmp):
    morceaux = [segment(i, p, langue, tmp) for i, p in enumerate(PLAN)]
    for m, p in zip(morceaux, PLAN):
        d = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", m],
                           capture_output=True, text=True, check=True).stdout.strip()
        print(f"  {p[0]:14} {float(d):5.1f} s")
    # Assemblage par le filtre concat (réencodage) : la copie directe laissait des trous entre les segments.
    duree = sum(
        float(subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", m],
                             capture_output=True, text=True, check=True).stdout)
        for m in morceaux
    )
    final = SORTIE / f"demo.{langue}.mp4"
    entrees = [x for m in morceaux for x in ("-i", m)]
    n = len(morceaux)
    filtre = "".join(f"[{k}:v]" for k in range(n)) + f"concat=n={n}:v=1:a=0,fps={IPS},format=yuv420p[v];"
    # La musique dès le début, avec une sortie en fondu sur 4 s.
    filtre += f"[{n}:a]atrim=0:{duree},volume=0.8,afade=t=in:d=0.6,afade=t=out:st={duree - 4}:d=4[a]"
    ffmpeg(*entrees, "-i", SRC / "sidewalk-shade.mp3", "-filter_complex", filtre, "-map", "[v]", "-map", "[a]",
           "-c:v", "libx264", "-crf", "23", "-preset", "medium", "-c:a", "aac", "-b:a", "160k", "-t", f"{duree:.2f}",
           "-movflags", "+faststart", final)
    print(f"{final} : {duree:.1f} s")


def main():
    with tempfile.TemporaryDirectory() as t:
        for langue in sys.argv[1:] or ["fr", "en"]:
            monter(langue, Path(t))


if __name__ == "__main__":
    main()
