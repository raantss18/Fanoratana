"""Essai de tool/build_site.py : `python3 tool/test_build_site.py` depuis la racine du dépôt."""
import sys, json
sys.path.insert(0, "tool")
import build_site as b
corps = "# Fanoratana 9.9.9\n\nBonjour <b>le</b> monde.\n\n**Installer** : x\n\n---\n\nHello world.\n\n<details>"
assert b.notes(corps) == {"fr": "Bonjour <b>le</b> monde.", "en": "Hello world."}
r = [{"draft": True, "tag_name": "v9", "assets": [{"name": b.APK}]},
     {"tag_name": "v1.2.3", "published_at": "2026-03-05T10:00:00Z", "body": corps,
      "assets": [{"name": b.APK, "size": 29627218, "digest": "sha256:abc", "browser_download_url": "https://x/a.apk"}]}]
v = b.derniere_version(r)
assert v[0]["tag_name"] == "v1.2.3"
m = "{{tel_version}}|{{notes}}|{{apk_url}}|{{apk_taille}}|{{apk_sha}}|{{linux_sha}}|{{video}}"
fr = b.page(m, "fr", v, False); en = b.page(m, "en", v, False)
assert fr.startswith("Version 1.2.3, publiée le 5 mars 2026.|Bonjour &lt;b&gt;le&lt;/b&gt; monde.|https://x/a.apk|30 Mo|abc|—|"), fr
assert en.startswith("Version 1.2.3, released on March 5, 2026.|Hello world."), en
assert "GitHub" in b.page(m, "fr", None, False)
print("site ok")
