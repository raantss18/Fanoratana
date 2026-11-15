#!/usr/bin/env python3
"""Écriture « au fil » pour les essais et la vidéo de démonstration : chaque lettre de Caveat (OFL) est réduite à
son trait central (squelette), puis rejouée d'un seul trait par lettre ou presque, comme un stylet.

`gestes.py` trace le contour des lettres : le lecteur y voit des boucles et lit mal (relevé le 2026-11-15 sur la
Pad 7 : « Les fonctions » lu « bis Rome Biome »). Ici le tracé ressemble à une vraie écriture.

    tool/essais/fil.py <sortie.sh> <x> <y> <hauteur> "<texte>" [<x> <y> <hauteur> "<texte>" ...]

Coordonnées en pixels d'écran (paysage 3200 × 2136) : `y` est la ligne de base, `hauteur` le corps.
Jamais pendant qu'Antsa se sert de la tablette (CLAUDE.md, règle 3).
"""
import random
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

RACINE = Path(__file__).resolve().parents[2]
CORPS = 240  # px du rendu dont on tire le squelette
POLICE = ImageFont.truetype(str(RACINE / "assets/polices/Caveat.ttf"), CORPS)
POLICE.set_variation_by_axes([400]) if hasattr(POLICE, "set_variation_by_axes") else None


def amincir(img):
    """Amincissement de Zhang-Suen : il reste une ligne d'un pixel au milieu du trait."""
    a = img.copy()
    while True:
        change = False
        for passe in (0, 1):
            p2, p3, p4 = np.roll(a, 1, 0), np.roll(np.roll(a, 1, 0), -1, 1), np.roll(a, -1, 1)
            p5, p6, p7 = np.roll(np.roll(a, -1, 0), -1, 1), np.roll(a, -1, 0), np.roll(np.roll(a, -1, 0), 1, 1)
            p8, p9 = np.roll(a, 1, 1), np.roll(np.roll(a, 1, 0), 1, 1)
            v = [p2, p3, p4, p5, p6, p7, p8, p9]
            b = sum(v)
            t = sum((v[i] == 0) & (v[(i + 1) % 8] == 1) for i in range(8))
            if passe == 0:
                c = (p2 * p4 * p6 == 0) & (p4 * p6 * p8 == 0)
            else:
                c = (p2 * p4 * p8 == 0) & (p2 * p6 * p8 == 0)
            retire = (a == 1) & (b >= 2) & (b <= 6) & (t == 1) & c
            if retire.any():
                a[retire] = 0
                change = True
        if not change:
            return sans_marches(a)


def sans_marches(a):
    """Retire les pixels de coin (« marches d'escalier ») : il reste une ligne où chaque pixel a deux voisins."""
    h, w = a.shape
    for y in range(1, h - 1):
        for x in range(1, w - 1):
            if not a[y, x]:
                continue
            n, s, e, o = a[y - 1, x], a[y + 1, x], a[y, x + 1], a[y, x - 1]
            if (
                (n and e and not a[y + 1, x - 1] and not s and not o)
                or (n and o and not a[y + 1, x + 1] and not s and not e)
                or (s and e and not a[y - 1, x - 1] and not n and not o)
                or (s and o and not a[y - 1, x + 1] and not n and not e)
            ):
                a[y, x] = 0
    return a


def chemins(sq):
    """Les traits du squelette : suites de pixels voisins, des bouts vers les croisements."""
    pts = {(int(y), int(x)) for y, x in zip(*np.nonzero(sq))}
    voisins = lambda p: [q for q in ((p[0] + dy, p[1] + dx) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if dy or dx) if q in pts]
    degre = {p: len(voisins(p)) for p in pts}
    vus, res = set(), []

    def suivre(depart, premier):
        ligne, avant, p = [depart, premier], depart, premier
        vus.add(frozenset((depart, premier)))
        while degre[p] == 2:
            suite = [q for q in voisins(p) if q != avant and frozenset((p, q)) not in vus]
            if not suite:
                break
            q = suite[0]
            vus.add(frozenset((p, q)))
            ligne.append(q)
            avant, p = p, q
        return ligne

    # D'abord depuis les bouts (le plus à gauche d'abord), puis depuis les croisements, puis les boucles fermées.
    for groupe in (lambda d: d == 1, lambda d: d > 2, lambda d: d == 2):
        for p in sorted((p for p in pts if groupe(degre[p])), key=lambda p: (p[1], p[0])):
            for q in voisins(p):
                if frozenset((p, q)) not in vus:
                    res.append(suivre(p, q))
    return [l for l in res if len(l) >= 7]


def recoller(lignes):
    """Recolle les morceaux qui se suivent à un croisement (même direction), pour tracer la lettre d'un geste."""
    lignes = [list(l) for l in lignes]
    fini = False
    while not fini:
        fini = True
        for i, a in enumerate(lignes):
            for j, b in enumerate(lignes):
                if i == j:
                    continue
                for bb in (b, b[::-1]):
                    if abs(a[-1][0] - bb[0][0]) <= 2 and abs(a[-1][1] - bb[0][1]) <= 2 and len(a) > 6 and len(bb) > 6:
                        da = np.subtract(a[-1], a[-6])
                        db = np.subtract(bb[5], bb[0])
                        if np.dot(da, db) / (np.linalg.norm(da) * np.linalg.norm(db) + 1e-9) > 0.5:
                            lignes[i] = a + bb[1:]
                            del lignes[j]
                            fini = False
                            break
                if not fini:
                    break
            if not fini:
                break
    return lignes


_cache = {}


def lettre(ch):
    """Les traits d'une lettre, en fractions du corps : [(x, y), …] avec y vers le bas depuis la ligne de base."""
    if ch in _cache:
        return _cache[ch]
    marge = 40
    w = int(POLICE.getlength(ch)) + 2 * marge
    img = Image.new("L", (w, CORPS * 2), 0)
    base = int(CORPS * 1.3)
    ImageDraw.Draw(img).text((marge, base), ch, font=POLICE, fill=255, anchor="ls")
    sq = amincir((np.array(img) > 110).astype(np.uint8))
    traits = []
    for l in recoller(chemins(sq)):
        # Écriture de gauche à droite, de haut en bas.
        if (l[0][1], l[0][0]) > (l[-1][1], l[-1][0]) and abs(l[0][1] - l[-1][1]) > 6:
            l = l[::-1]
        elif abs(l[0][1] - l[-1][1]) <= 6 and l[0][0] > l[-1][0]:
            l = l[::-1]
        pas = max(1, len(l) // max(6, min(22, len(l) // 9)))
        pts = l[::pas] + ([l[-1]] if (len(l) - 1) % pas else [])
        traits.append([((x - marge) / CORPS, (y - base) / CORPS) for y, x in pts])
    traits.sort(key=lambda t: min(p[0] for p in t))
    _cache[ch] = (traits, POLICE.getlength(ch) / CORPS)
    return _cache[ch]


def ecrire(cmds, texte, x0, y0, h):
    dx = 0.0
    for ch in texte:
        if ch == " ":
            dx += 0.28
            continue
        traits, avance = lettre(ch)
        for t in traits:
            pts = [(int(x0 + (dx + x) * h + random.uniform(-1.5, 1.5)), int(y0 + y * h + random.uniform(-1.5, 1.5))) for x, y in t]
            cmds.append(f"input motionevent DOWN {pts[0][0]} {pts[0][1]}")
            cmds += [f"input motionevent MOVE {x} {y}" for x, y in pts[1:]]
            cmds.append(f"input motionevent UP {pts[-1][0]} {pts[-1][1]}")
        dx += avance


def main(argv):
    random.seed(7)
    sortie, reste = argv[0], argv[1:]
    cmds = []
    for i in range(0, len(reste), 4):
        x, y, h, texte = float(reste[i]), float(reste[i + 1]), float(reste[i + 2]), reste[i + 3]
        ecrire(cmds, texte, x, y, h)
    Path(sortie).write_text("\n".join(cmds) + "\n")
    print(f"{sortie} : {len(cmds)} commandes")


if __name__ == "__main__":
    main(sys.argv[1:])
