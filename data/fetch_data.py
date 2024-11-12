#!/usr/bin/env python3

# Run this script in this directory to update the json files.
# This should not depend on third party packages so that its easy to run it.

import base64
import json
import os
import subprocess
import logging


# Class for fetching data that could be simply fetched with a wget command.
class Data:
    output_file_name: str
    url: str

    def __init__(self, output_file_name: str, url: str):
        self.output_file_name = output_file_name
        self.url = url

    def fetch(self):
        logging.info(f"Fetching {self.url}")
        subprocess.run(["wget", self.url, "-O", self.output_file_name])


# Class for fetching data from Github API.
# Only public files are supported.
class GithubData:
    output_file_name: str
    repo: str
    path: str

    def __init__(self, output_file_name: str, repo: str, path: str):
        self.output_file_name = output_file_name
        self.repo = repo
        self.path = path

    def fetch(self):
        logging.info(f"Fetching from Github: {self.repo}'s {self.path}")
        # Using github api.
        # https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28
        temporary_file_name = self.output_file_name + ".tmp"
        cmd = [
            "curl",
            # Note that no authorization token is used so only public repository
            # can be fetched.
            "-H",
            "Accept: application/vnd.github+json",
            "-H",
            "X-GitHub-Api-Version: 2022-11-28",
            "-o",
            temporary_file_name,
            f"https://api.github.com/repos/{self.repo}/contents/{self.path}",
        ]
        subprocess.run(cmd)

        with open(temporary_file_name, "r") as f:
            result = json.loads(f.read())
        if result["encoding"] != "base64":
            raise ValueError(
                f"Repo {self.repo}'s {self.path} has Invalid encoding: {result['encoding']}"
            )
        content = result["content"]
        if content is None:
            raise ValueError(f"Repo {self.repo}'s {self.path} has no content")

        logging.debug(
            f"Download url for {self.repo}'s {self.path}: {result['download_url']}"
        )
        logging.debug(f"Writing decoded content to{self.output_file_name}")
        with open(self.output_file_name, "wb") as f:
            f.write(base64.b64decode(content))

        os.remove(temporary_file_name)


# Add files to fetch here.
DATA: list[Data | GithubData] = [
    Data("words.json", "https://dataset.genshin-dictionary.com/words.json"),
    Data("jp_itemid.json", "https://api.uigf.org/dict/genshin/jp.json"),
    Data("en_itemid.json", "https://api.uigf.org/dict/genshin/en.json"),
    GithubData(
        "achievements.json",
        "tokafew420/genshin-impact-tools",
        "data/achievements.json",
    ),
    GithubData(
        "character-ascension.json",
        "tokafew420/genshin-impact-tools",
        "data/character-ascension.json",
    ),
    GithubData(
        "character-level.json",
        "tokafew420/genshin-impact-tools",
        "data/character-level.json",
    ),
    GithubData(
        "character-talent-level-up.json",
        "tokafew420/genshin-impact-tools",
        "data/character-talent-level-up.json",
    ),
    GithubData(
        "characters.json",
        "tokafew420/genshin-impact-tools",
        "data/characters.json",
    ),
    GithubData(
        "elements.json",
        "tokafew420/genshin-impact-tools",
        "data/elements.json",
    ),
    GithubData(
        "resources.json",
        "tokafew420/genshin-impact-tools",
        "data/resources.json",
    ),
    GithubData(
        "weapon-ascension.json",
        "tokafew420/genshin-impact-tools",
        "data/weapon-ascension.json",
    ),
    GithubData(
        "weapon-level.json",
        "tokafew420/genshin-impact-tools",
        "data/weapon-level.json",
    ),
    GithubData(
        "weapons.json",
        "tokafew420/genshin-impact-tools",
        "data/weapons.json",
    ),
]


def main():
    for data in DATA:
        data.fetch()


if __name__ == "__main__":
    main()
