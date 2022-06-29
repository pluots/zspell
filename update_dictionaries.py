import base64
import json
import os
import urllib.request
from dataclasses import dataclass

# Path to directory with all dictionaries
GH_URL = "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries"
USERNAME = os.environ.get("GH_USERNAME")
TOKEN = os.environ.get("GH_TOKEN")

if USERNAME is None or TOKEN is None:
    print("Not using tokens")
else:
    print("Using token")


def make_req(url: str):
    if USERNAME is None or TOKEN is None:
        return url

    auth = base64.b64encode(bytes(f"{USERNAME}:{TOKEN}", "utf8"))
    req = urllib.request.Request(url)
    req.add_header("Authorization", f"Basic {auth}")
    return req


def get_url_data(url: str):
    return urllib.request.urlopen(make_req(url))


def download_file(url: str, path: str):
    return urllib.request.urlopen(make_req(url))


@dataclass
class LangDict:
    name: str
    dir_url: str
    dict_url: str = None
    affix_url: str = None
    license_url: str = None

    @property
    def dict_fname(self):
        return f"{self.name}.dic"

    @property
    def affix_fname(self):
        return f"{self.name}.aff"

    @property
    def license_fname(self):
        return f"{self.name}.license"


print("Gathering listing")

dict_dir_listing = get_url_data(GH_URL)

# Collection of dictionaries. Format:
# [
#   {
#     "name": "bg",
#     "path": "dictionaries/bg",
#     "sha": "f5333c51490bfd4ca38ae10bfa7a930b0ca590f4",
#     "size": 0,
#     "url": "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries/bg?ref=main",
#     "html_url": "https://github.com/wooorm/dictionaries/tree/main/dictionaries/bg",
#     "git_url": "https://api.github.com/repos/wooorm/dictionaries/git/trees/f5333c51490bfd4ca38ae10bfa7a930b0ca590f4",
#     "download_url": null,
#     "type": "dir",
#     "_links": {
#       "self": "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries/bg?ref=main",
#       "git": "https://api.github.com/repos/wooorm/dictionaries/git/trees/f5333c51490bfd4ca38ae10bfa7a930b0ca590f4",
#       "html": "https://github.com/wooorm/dictionaries/tree/main/dictionaries/bg"
#     }
#   }, ...
# ]
data = json.loads(dict_dir_listing.read())

# Get the name of the directory (language name) and the URL where we can find
# its contents
lang_dicts = [LangDict(d.get("name").replace("-", "_"), d.get("url")) for d in data]

lang_dicts_working: list[LangDict] = []

print("Loading file URLs")

for i, ldict in enumerate(lang_dicts):
    print(f"File {i} of {len(lang_dicts)}", end="\r", flush=True)

    if ldict.name is None or ldict.dir_url is None:
        continue

    lang_dir_listing = get_url_data(ldict.dir_url)
    data = json.loads(lang_dir_listing.read())

    ldict.dict_url = next(
        (
            d.get("download_url")
            for d in data
            if d.get("name", "").lower().endswith(".dic")
        ),
        None,
    )
    ldict.affix_url = next(
        (
            d.get("download_url")
            for d in data
            if d.get("name", "").lower().endswith(".aff")
        ),
        None,
    )
    ldict.license_url = next(
        (d.get("download_url") for d in data if d.get("name", "").lower() == "license"),
        None,
    )

    if ldict.dict_url is None or ldict.affix_url is None or ldict.license_url is None:
        continue

    lang_dicts_working.append(ldict)

print("Loading dictionaries")

for i, ldict in enumerate(lang_dicts_working):
    # First download the files with .tmp
    print(f"Dictionary {i} of {len(lang_dicts_working)}", end="\r", flush=True)

    download_file(ldict.dict_url, f"dictionaries/{ldict.dict_fname}.tmp")
    download_file(ldict.affix_url, f"dictionaries/{ldict.affix_fname}.tmp")
    download_file(ldict.license_url, f"dictionaries/{ldict.license_fname}.tmp")

    # If all goes well, there will be no problems. If one failed, program would abort
    # Now remove the old ones, if present
    for fname in (ldict.dict_fname, ldict.affix_fname, ldict.license_fname):
        if os.path.exists(f"dictionaries/{fname}"):
            os.remove(f"dictionaries/{fname}")

        # And replace with the new
        os.rename(f"dictionaries/{fname}.tmp", f"dictionaries/{fname}")

print("Done!")
