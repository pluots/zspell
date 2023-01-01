#!/usr/bin/env python3

"""This script downloads files from the github `wooorm/dictionaries` repository.

Be sure to obey licensing.
"""

import argparse
import base64
import json
import os
import urllib.request
from dataclasses import dataclass
from typing import Any

# Path to directory with all dictionaries
ROOT_GH_URL = "https://api.github.com/repos/wooorm/dictionaries/contents/dictionaries"


@dataclass
class AuthInfo:
    """Login information"""

    username: str
    token: str


@dataclass
class LangDict:
    """Represent the URLs for a specific language"""

    name: str
    dir_url: str
    dict_url: str = None
    affix_url: str = None
    license_url: str = None

    def set_urls(self, auth: AuthInfo | None):
        """Set dict, affix, and license URLs from the name and dir URL"""
        listing: list[dict[str, Any]] = get_url_data_json(self.dir_url, auth)
        self.dict_url = next(
            d["download_url"] for d in listing if d["name"].endswith(".dic")
        )
        self.affix_url = next(
            d["download_url"] for d in listing if d["name"].endswith(".aff")
        )
        self.license_url = next(
            d["download_url"] for d in listing if d["name"].lower() == "license"
        )

    def download(self, path: str, auth: AuthInfo | None) -> None:
        """Download the files to a designated path"""
        print(f"Downloading files for language '{self.name}'")

        dict_path = f"{path}/{self.dict_fname}"
        affix_path = f"{path}/{self.affix_fname}"
        license_path = f"{path}/{self.license_fname}"
        all_paths = (dict_path, affix_path, license_path)

        for fname in all_paths:
            if os.path.exists(fname):
                print(f"Language '{self.name}' already exists, found '{fname}'")
                print("Skipping")
                return

        download_file(self.dict_url, f"{path}/{self.dict_fname}.tmp", auth)
        download_file(self.affix_url, f"{path}/{self.affix_fname}.tmp", auth)
        download_file(self.license_url, f"{path}/{self.license_fname}.tmp", auth)

        # If all goes well, there will be no problems. If one failed, program would abort
        # Now remove the old ones, if present
        for fname in all_paths:
            if os.path.exists(fname):
                os.remove(fname)

            # And replace with the new
            os.rename(f"{fname}.tmp", f"{fname}")

        print(f"Finished downloading files for '{self.name}'")

    @property
    def dict_fname(self):
        return f"{self.name}.dic"

    @property
    def affix_fname(self):
        return f"{self.name}.aff"

    @property
    def license_fname(self):
        return f"{self.name}.license"


def make_req(url: str, auth: AuthInfo | None) -> str | urllib.request.Request:
    """Make a request with auth information"""
    if auth is None:
        return url

    auth_str = base64.b64encode(bytes(f"{auth.username}:{auth.token}", "utf8"))
    req = urllib.request.Request(url)
    req.add_header("Authorization", f"Basic {auth_str}")
    return req


def get_url_data_json(url: str, auth: AuthInfo | None):
    return json.loads(urllib.request.urlopen(make_req(url, auth)).read())


def download_file(url: str, path: str, auth: AuthInfo | None):
    return urllib.request.urlretrieve(make_req(url, auth), path)


def parse_args():
    parser = argparse.ArgumentParser(
        prog="Dictionary downloader",
        description="Download dictionaries for development",
    )
    parser.add_argument(
        "languages", nargs="+", help="Specify language codes to download"
    )
    parser.add_argument("--username", help="specify a github username")
    parser.add_argument("--token", help="specify a github token")
    parser.add_argument(
        "--output-dir", help="specify the output directory", default="dictionaries"
    )
    args = parser.parse_args()
    return args


def make_lang_dicts(languages: list[str], auth: AuthInfo) -> list[LangDict]:
    print("Gathering listing")

    listing_data: list[dict] = get_url_data_json(ROOT_GH_URL, auth)

    lang_dicts: list[LangDict] = []

    for lang in languages:
        lang_name = lang.replace("_", "-")
        listing = next(
            (listing for listing in listing_data if listing.get("name") == lang_name),
            None,
        )
        if listing is None:
            print(f"Unable to find language {lang}")
            exit(1)
        lang_dicts.append(LangDict(listing["name"], listing["url"]))

    return lang_dicts


def main():
    print(__doc__)
    args = parse_args()
    username = args.username or os.environ.get("GH_USERNAME")
    token = args.token or os.environ.get("GH_TOKEN")

    if username is None or token is None:
        print("Not using authentication, large requests may fail")
        auth = None
    else:
        print("Using token authentication")
        auth = AuthInfo(username, token)

    print(username, token)
    lang_dicts = make_lang_dicts(args.languages, auth)

    for ldict in lang_dicts:
        ldict.set_urls(auth)
        ldict.download(args.output_dir, auth)


if __name__ == "__main__":
    main()
