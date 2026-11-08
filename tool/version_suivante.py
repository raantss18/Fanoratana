#!/usr/bin/env python3
"""La version suivante, règle d'Antsa (2026-11-07) : chaque chiffre va de 0 à 9.

0.0.9 -> 0.1.0 ; 0.9.9 -> 1.0.0. Usage : tool/version_suivante.py 0.1.9
"""
import sys


def suivante(version: str) -> str:
    chiffres = [int(c) for c in version.split(".")]
    if len(chiffres) != 3 or any(not 0 <= c <= 9 for c in chiffres):
        raise ValueError(f"version hors règle : {version}")
    for i in (2, 1, 0):
        if chiffres[i] < 9:
            chiffres[i] += 1
            return ".".join(map(str, chiffres))
        chiffres[i] = 0
    raise ValueError("après 9.9.9, à décider avec Antsa")


if __name__ == "__main__":
    for essai, attendu in [("0.1.9", "0.2.0"), ("1.9.9", "2.0.0"), ("1.0.0", "1.0.1"), ("0.9.9", "1.0.0")]:
        assert suivante(essai) == attendu, (essai, suivante(essai))
    print(suivante(sys.argv[1]))
