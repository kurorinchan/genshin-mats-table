## words.json
This file contains the translation of "words" in the game.
It is from
https://dataset.genshin-dictionary.com/words.json

See also https://github.com/xicri/genshin-langdata where this data is generated

## Other json files
The rest of the json files are from
https://github.com/tokafew420/genshin-impact-tools.

Look under its [`data/` directory](https://github.com/tokafew420/genshin-impact-tools/tree/main/data).

## `en_itemid.json` and `jp_itemid.json`
The files are fetched from this API: https://uigf.org/en/api.html#language-identification-api

Specifically https://api.uigf.org/dict/genshin/jp.json and https://api.uigf.org/dict/genshin/en.json

As noted in the documentation, it can handle other languages too.

The JSON files contain a mapping from human readable string to an ID, called item ID.
Hence the files are renamed to `en_itemid.json` and `jp_itemid.json`.