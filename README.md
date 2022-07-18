# Blob foxes emotes repository

A repository containing the "blobfox" emotes, [originally made by Feuerfuchs](https://web.archive.org/web/20211115174913/https://www.feuerfuchs.dev/en/projects/blobfox-emojis/)
and released under the Apache 2.0 license.
<!-- TODO: find new link+email for feuerfuchs -->
Work was initially made [on a public archive](https://git.lambdaurora.dev/lambdaurora/diverse_archive) to vectorized some of the blobfoxes,
but as more variants were created, the project needed a new, dedicated place to reside.

This repository also contains additional blobfoxes, blobcats and other blob animal characters, which are all made available under the Apache 2.0 license.

The goal of this project is to vectorize the entire set of blobfoxes.
Automation is key for that, and a solution that can generate clean (both in terms of readability and visual accuracy) results with minimal tweaking will have to be built.

If you enjoy this project, then come say hi here:

- [subreddit dedicated to this project](https://reddit.com/blobfox)
- matrix room (TODO)

## Installation and usage

*TODO :)*

## Project structure

*(this will likely have changed by the time you are reading this)*

- `blobfox.tar.gz` is the tar archive of the original set of blobfox emojis; run `mkdir original && tar -xf blobfox.tar.gz -C original` to uncompress the archive
- `custom/` contains PNG versions of variants based on the above archive that haven't been vectorized yet
- `vector/` contains vectorized versions of the blobfoxes (not necessary limited to the original blobfoxes)

## License

All the code, images and assets of this repository are made available under the Apache 2.0 license.
See [LICENSE.txt](LICENSE.txt) for more information.

For simplicity, the authors of the different parts of this project have been grouped together under the "blobfox team", which contains but is not limited to:

- Feuerfuchs: [original website (down as of writing)](https://feuerfuchs.dev/), [archive](https://web.archive.org/web/20211115174913/https://www.feuerfuchs.dev/en/projects/blobfox-emojis/)
- [LambdAurora](https://git.lambdaurora.dev/lambdaurora/), hosts [a mirror](https://git.lambdaurora.dev/lambdaurora/blobfox)
- [Shad Amethyst](https://git.shadamethyst.xyz/adri326/)

<!-- Add yourself here as you please :) -->
<!-- If we get more people, then we can create a CONTRIBUTORS.txt file -->

## Contributing

Any contribution to the [original repository of this project](https://git.shadamethyst.xyz/adri326/blobfox) must be made available under the [license of this project](./LICENSE.txt).

This means that:
- You must either be the author of the code/asset/image you wish to contribute, or have been given explicit permission by the original author to contribute it here.
- Assets and images from websites like `slackmojis`, `blobs.gg`, `emojos.in` or `emoji.gg` are generally **not** welcome,
  as these website either don't care about copyright ownership or have a questionable licensing record.
- Contributions derived from assets and images from the above websites are likewise not welcome;
  as a rule of thumb, asset contributions should only base themselves on what is present in this repository.

For svg files, please include the [svg-default-metadata.xml](./svg-default-metadata.xml) file in the svg file.
