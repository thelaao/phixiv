# phixiv

[pixiv](https://www.pixiv.net/) embed fixer. If you run into any issues or have any suggestions to make this service better, please feel free to [open an issue](https://github.com/thelaao/phixiv/issues/new).

## How to Use

Just replace "pixiv" with "phixiv" or "ppxiv" in urls to embed web content properly on apps, such as Discord and Telegram.

To choose a specified image from a artwork (e.g., manga), you should append `/<index>` to the link. The index starts from 1.

If you want a more compact embed, use c.phixiv.net or c.ppxiv.net, those will omit the text from the pixiv post.

### Path Formats

The following are valid paths for artworks, if there should be more embedding paths, please [open an issue](https://github.com/thelaao/phixiv/issues/new).
To embed multiple images at once you can use a range for the :index parameter, e.g. 1-3 for the first three images, 4-6 for the next three and so on.

```text
/artworks/:id
/:language/artworks/:id
/artworks/:id/:index
/:language/artworks/:id/:index
/i/:id
/member_illust.php?illust_id=:id
```

Here are some examples.

| URL | Description |
|:- |:- |
| https://www.phixiv.net/member_illust.php?illust_id=124748386 | Artwork with ID 124748386 |
| https://www.phixiv.net/artworks/124748386 | The same artwork |
| https://www.phixiv.net/en/artworks/124748386 | The same artwork with tags in English translation |
| https://www.phixiv.net/artworks/124748386/2 | The 2nd (not 3rd) image of the same artwork |
| https://www.phixiv.net/artworks/124748386/1-3 | The first three images of the artwork |

### Discord Shortcut

On Discord, instead of editing pixiv urls, typing `s/i/p` and hitting Enter could be a bit more quick. This command will replace `pixiv` with `ppxiv`. If there are multiple `i`'s in that message, you should use `s/pixiv/ppxiv` to shoot it accurately.

### API

The API at https://www.phixiv.net/api/info?id=:id is no longer available.
Developers should call the Pixiv API directly, https://www.pixiv.net/ajax/illust/142030918 for general information and https://www.pixiv.net/ajax/illust/142030918/pages for the original image urls.

## Advanced Usages

There are some APIs for internal usage. It's recommended to read the source code before using them directly.

| Path | Description |
|:- |:- |
| `/health` | Health check |
| `/e/?n=:author_name&i=:author_id` | oEmbed-like API |
| `/i/:path` | Proxy API |
| `/api/v1/statuses/:status_id` | Mastodon-like API |
