# Composants tiers

| Composant | Licence | Usage | Source |
|---|---|---|---|
| Caveat (police) | SIL OFL 1.1 | écriture au propre | https://github.com/googlefonts/caveat — `assets/polices/OFL-Caveat.txt` |
| Nunito (police) | SIL OFL 1.1 | interface | https://github.com/googlefonts/nunito — `assets/polices/OFL-Nunito.txt` |
| Noto Sans Math (police) | SIL OFL 1.1 | symboles et lettres grecques | https://github.com/notofonts/math — `assets/polices/OFL-NotoSansMath.txt` |
| latin_PP-OCRv5_mobile_rec (modèle ONNX) | Apache-2.0 | lecture de l'écriture | https://huggingface.co/PaddlePaddle/latin_PP-OCRv5_mobile_rec_onnx (SHA-256 `7888113072263cb4…121498`) ; données d'entraînement non publiées par l'éditeur |
| Pix2Text-MFR 1.0 (modèle ONNX, **non embarqué**) | MIT | formules manuscrites → LaTeX, téléchargé à la demande depuis la page de l'auteur | https://huggingface.co/breezedeus/pix2text-mfr (révision `bea257ed`) ; provenance des données non publiée |
| Liste de mots français FrequencyWords 2018 (Hermit Dave) | données CC BY-SA 4.0 (code MIT) | correction des mots incertains (`assets/lexique/fr.txt`, filtrée) | https://github.com/hermitdave/FrequencyWords — d'après OpenSubtitles2018 (OPUS) |
| mahay-android, FanoActivity, ApkInstaller | MIT ou Apache-2.0 | passerelle Android | https://github.com/raantss18/mahay (`libs/core/crates/android`, `android/shell`) |
| tract | MIT ou Apache-2.0 | exécution du modèle | https://github.com/sonos/tract |
| egui, eframe, epaint | MIT ou Apache-2.0 | interface | https://github.com/emilk/egui |
| resvg, usvg, tiny-skia, svg2pdf, pdf-writer | MIT ou Apache-2.0 / BSD-3 (tiny-skia) | export | https://github.com/linebender/resvg · https://github.com/typst/svg2pdf |
| ureq, rustls, ring, webpki-roots | MIT/Apache-2.0, ISC, MPL-2.0 (webpki-roots) | mises à jour | https://github.com/algesten/ureq |
| ed25519-dalek, sha2 | BSD-3, MIT/Apache-2.0 | vérification des mises à jour | https://github.com/dalek-cryptography/curve25519-dalek |

Méthodes réécrites ici (pas de code repris) : perfect-freehand (Steve Ruiz, MIT) pour le contour du trait ; rough.js (MIT) pour le
trait fait main ; ShortStraw (Wolin et al., 2008) pour les coins ; seuils de rature de Fikajiana (ADR 0021 là-bas).
L'icône `brand/icone.svg` est redessinée d'après l'image de référence d'Antsa.
