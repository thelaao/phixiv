# phixiv

[pixiv](https://www.pixiv.net/) embed fixer. If you run into any issues or have any suggestions to make this service better, please [make an issue](https://github.com/thelaao/phixiv/issues/new).

## How to use

Replace "pixiv" with "phixiv" in the url to embed properly on Discord, etc. Alternatively, if on discord you can also paste the pixiv url and send `s/i/p` after, this will edit the previous message, replacing `pixiv` with `ppxiv` which will also embed properly; please note this will require the link to include the first `i` in your message.

Additionally, when embedding a post with multiple images, add `/<index>` to the end of the link to embed that image.

## Path Formats

The following are the valid paths for artworks, if there is a format which isn't listed which should be embedded, please [make an issue](https://github.com/thelaao/phixiv/issues/new).

```text
/artworks/:id
/:language/artworks/:id
/artworks/:id/:index
/:language/artworks/:id/:index
/member_illust.php?illust_id=:id
```

A simple API for basic information such as tags and direct image links is provided.

```text
/api/info?id=<id>&language=<language>
```

### Examples

- Arwork with ID 124748386: https://www.phixiv.net/artworks/124748386
- Second image of the same artwork: https://www.phixiv.net/artworks/124748386/2
- Same artwork with the tags translated to english: https://www.phixiv.net/en/artworks/124748386
