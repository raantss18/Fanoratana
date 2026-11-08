#!/usr/bin/env python3
"""Relève les lignes lues dans le journal de la tablette (adb logcat) et les écrit en fixtures de test.

Usage : tool/essais/journal.py <appareil> <dossier> ; écrit <dossier>/lecture_<id>.json (format des fixtures de
Fikajiana : [{"points": [[x, y], …]}, …]) et affiche la lecture obtenue pour chacune.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

appareil, dossier = sys.argv[1], Path(sys.argv[2])
dossier.mkdir(parents=True, exist_ok=True)
log = subprocess.run(["adb", "-s", appareil, "logcat", "-d", "-s", "Fanoratana:I"], capture_output=True, text=True).stdout
lues = {}
for ligne in log.splitlines():
    m = re.search(r"lecture (\d+) traits (\[.*\])$", ligne)
    if m:
        traits = json.loads(m.group(2))
        (dossier / f"lecture_{m.group(1)}.json").write_text(json.dumps([{"points": t} for t in traits]))
        lues.setdefault(m.group(1), "(pas de résultat)")
    m = re.search(r"lecture (\d+) (lue : .*|: erreur .*)$", ligne)
    if m:
        lues[m.group(1)] = m.group(2)
    if "dollars :" in ligne:
        print(ligne.split("Fanoratana:")[-1].strip())
for k, v in lues.items():
    print(k, v)
