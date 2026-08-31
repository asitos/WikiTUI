## bug fix(es)
- use `CatmullRom` filter for sharper halfblock image resampling: [df24c9c](https://github.com/sharkthakftw/wikid/commit/df24c9cced6f9b988b0e01ecd69a685474708be0)
### by [asitos](https://github.com/asitos) in [#7](https://github.com/sharkthakftw/wikid/pull/7)
- fix kitty graphics protocol placement
- prevent aspect-ratio distortion during scrolling
- preserve terminal cursor position during hardware image drawing
- align image dimensions to `block.inner(rect)` to prevent overlap with borders
- filter out `noviewer` and math fallback image tags during article parsing
