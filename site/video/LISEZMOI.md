# La vidéo de démonstration

`demo.fr.mp4` et `demo.en.mp4` (2 min, 1280 × 920) sont montées par `tool/video_demo.py` à partir de `sources/`,
qui n'est pas dans Git (fichiers lourds) :

| Fichier | Ce que c'est |
|---|---|
| `a.mp4` … `f.mp4`, `r.mp4` | l'écran de la Pad 7 (`adb shell screenrecord --size 1600x1068`) pendant le rejeu de traits écrits « au fil » par `tool/essais/fil.py` ; ce qui s'affiche est la vraie lecture de l'app |
| `fikajiana.mp4` | la démonstration de Fikajiana (`Fikajiana/site/video/demo.mp4`), dont on garde l'écran sans son bandeau |
| `sidewalk-shade.mp3` | « Sidewalk Shade », Kevin MacLeod (incompetech.com), licence CC BY 4.0 — https://incompetech.com/music/royalty-free/mp3-royaltyfree/Sidewalk%20Shade.mp3 ; le crédit est dans le dernier carton de la vidéo et sous la vidéo sur le site |

Pour refaire la vidéo : remettre ces fichiers dans `sources/`, puis `python3 tool/video_demo.py`.
