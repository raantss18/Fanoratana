#!/usr/bin/env python3
"""Gestes d'essai pour la tablette (adb `input motionevent`), écran paysage 3200 × 2136.

Usage : tool/essais/gestes.py <dossier> ; écrit cours1.sh (titre, phrase, f(x)=2x+1), cours2.sh (triangle
tenu, signature tenue), cours3.sh (rature de la 2e ligne), formule.sh (x²+1=5 au fil), gras.sh (mot épais),
cases.sh (Date, 1789 dans un tableau). Les lettres viennent de Caveat (OFL) ; la formule est tracée au fil.
Jamais pendant qu'Antsa se sert de la tablette (CLAUDE.md, règle 3).
"""
import math
import random
import sys
from pathlib import Path

from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTFont

RACINE = Path(__file__).resolve().parents[2]
F = TTFont(RACINE / "assets/polices/Caveat.ttf")
GS, CMAP, UPM = F.getGlyphSet(), F.getBestCmap(), F["head"].unitsPerEm


def trait(cmds, pts, tenir=False):
    pts = [(int(x), int(y)) for x, y in pts]
    cmds.append(f"input motionevent DOWN {pts[0][0]} {pts[0][1]}")
    cmds += [f"input motionevent MOVE {x} {y}" for x, y in pts[1:]]
    if tenir:
        cmds += ["sleep 0.9", f"input motionevent MOVE {pts[-1][0]} {pts[-1][1]}"]
    cmds.append(f"input motionevent UP {pts[-1][0]} {pts[-1][1]}")


def ecrire(cmds, texte, x0, y0, h):
    k, dx = h / UPM, 0
    for ch in texte:
        g = GS[CMAP[ord(ch)]]
        if ch != " ":
            pen = RecordingPen()
            g.draw(pen)
            cur, last = [], None
            for op, args in pen.value:
                if op == "moveTo":
                    if len(cur) > 1:
                        trait(cmds, cur)
                    last = args[0]
                    cur = [(x0 + (dx + last[0]) * k, y0 - last[1] * k)]
                elif op == "lineTo":
                    last = args[0]
                    cur.append((x0 + (dx + last[0]) * k, y0 - last[1] * k))
                elif op in ("qCurveTo", "curveTo"):
                    c, p = args[0], args[-1]
                    for t in (0.5, 1.0):
                        q = [last[j] * (1 - t) ** 2 + 2 * c[j] * t * (1 - t) + p[j] * t * t for j in (0, 1)]
                        cur.append((x0 + (dx + q[0]) * k, y0 - q[1] * k))
                    last = p
                elif op in ("closePath", "endPath"):
                    if len(cur) > 1:
                        cur.append(cur[0])
                        trait(cmds, cur)
                    cur = []
        dx += g.width


FIL = {
    "x": [[(0.05, 0.45), (0.5, 1.0)], [(0.5, 0.45), (0.05, 1.0)]],
    "2": [[(0.1, 0.25), (0.3, 0.03), (0.55, 0.08), (0.6, 0.3), (0.35, 0.6), (0.08, 1.0), (0.65, 0.98)]],
    "+": [[(0.05, 0.62), (0.6, 0.62)], [(0.32, 0.35), (0.32, 0.9)]],
    "1": [[(0.12, 0.22), (0.38, 0.0), (0.38, 1.0)]],
    "=": [[(0.05, 0.5), (0.6, 0.5)], [(0.05, 0.78), (0.6, 0.78)]],
    "5": [[(0.6, 0.02), (0.18, 0.02), (0.12, 0.45), (0.42, 0.38), (0.62, 0.6), (0.5, 0.92), (0.1, 0.95)]],
}


def catmull(pts, n=8):
    if len(pts) == 2:
        return [tuple(pts[0][j] + (pts[1][j] - pts[0][j]) * t / 12 for j in (0, 1)) for t in range(13)]
    P = [pts[0]] + pts + [pts[-1]]
    out = []
    for i in range(1, len(P) - 2):
        p0, p1, p2, p3 = P[i - 1], P[i], P[i + 1], P[i + 2]
        for s in range(n):
            t = s / n
            out.append(tuple(0.5 * (2 * p1[j] + (-p0[j] + p2[j]) * t + (2 * p0[j] - 5 * p1[j] + 4 * p2[j] - p3[j]) * t * t
                                    + (-p0[j] + 3 * p1[j] - 3 * p2[j] + p3[j]) * t ** 3) for j in (0, 1)))
    return out + [pts[-1]]


def au_fil(cmds, ch, x0, y0, h):
    for tr in FIL[ch]:
        trait(cmds, [(x0 + px * h + random.uniform(-2, 2), y0 + py * h + random.uniform(-2, 2)) for px, py in catmull(tr)])


def main(dossier):
    random.seed(3)
    d = Path(dossier)
    a = []
    ecrire(a, "Les fonctions", 650, 560, 260)
    ecrire(a, "Une fonction associe", 650, 860, 190)
    ecrire(a, "f(x)=2x+1", 650, 1150, 200)
    b = []
    tri = [(700, 1750), (1150, 1760), (780, 1350), (700, 1748)]
    pts = [(tri[i][0] + (tri[i + 1][0] - tri[i][0]) * k / 10, tri[i][1] + (tri[i + 1][1] - tri[i][1]) * k / 10) for i in range(3) for k in range(10)]
    trait(b, pts + [tri[-1]], tenir=True)
    sig = [(1500 + (38 * t - 26 * math.sin(2 * t)) * 2.2, 1550 + 55 * math.cos(2 * t) + 14 * math.sin(0.7 * t)) for t in [i / 60 * 9 for i in range(61)]]
    trait(b, sig, tenir=True)
    c = []
    trait(c, [(650 if i % 2 == 0 else 2250, 700 + i * 12) for i in range(14)])
    f = []
    x, y, h = 1500, 930, 180
    au_fil(f, "x", x, y, h)
    x += h * 0.65
    au_fil(f, "2", x, y - 60, h * 0.55)
    x += h * 0.55
    for ch in "+1=5":
        au_fil(f, ch, x, y, h)
        x += h * 0.75
    g = []
    ecrire(g, "Important", 650, 1980, 200)
    fx = []
    ecrire(fx, "f(x)=2x+1", 650, 1150, 200)
    rx = []
    trait(rx, [(620 if i % 2 == 0 else 1450, 1000 + i * 12) for i in range(14)])
    e = []
    ecrire(e, "Date", 680, 860, 150)
    ecrire(e, "1789", 680, 1060, 140)
    for nom, cmds in [("cours1", a), ("cours2", b), ("cours3", c), ("formule", f), ("gras", g), ("cases", e), ("fonction", fx), ("rature_fonction", rx)]:
        (d / f"{nom}.sh").write_text("\n".join(cmds) + "\n")


if __name__ == "__main__":
    main(sys.argv[1])
