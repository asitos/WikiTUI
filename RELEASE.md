## new feature(s)
- caching article HTML in `~/.cache/wikid/articles`: [6e28c6f](https://github.com/sharkthakftw/wikid/commit/6e28c6f44e79b2d787771878d78754ab910f2717)
### new config options
#### \[reader]
- `code_line_numbers`, enable/disable gutter line numbers in codeblocks: [63dbace](https://github.com/sharkthakftw/wikid/commit/63dbace003886669baf47b617727724b383dafae)
#### \[search]
- `limit`, option for limiting search results: [73ba173](https://github.com/sharkthakftw/wikid/commit/73ba17301d591e9defc260e9a5759b32181e5070)
#### \[network]
- `timeout`, set timeout duration for api requests: [866898f](https://github.com/sharkthakftw/wikid/commit/866898f65c99c87317d0f6f6ddd1d1b640232456)
- `offline_cache`, enable/disable caching: [ac97f12](https://github.com/sharkthakftw/wikid/commit/ac97f1239e47b4f43341d46f260637f988a4f4bb)
- `cache_lifetime`, time in hours for cache expiry: [97e0e21](https://github.com/sharkthakftw/wikid/commit/97e0e214bcbbb4e8e65e06c98a25c04b5f9dee93)
## bug fix(es)
- prevented `underline_links` from underlining inline citations: [c0d7bd2](https://github.com/sharkthakftw/wikid/commit/c0d7bd25b5ca26b1be1b30dde8c40e221547ab2d)
## performance improvement(s)
- fixed double-counting `viewport_content_length()` for drawing the scroll indicator: [378fb6b](https://github.com/sharkthakftw/wikid/commit/378fb6b36c710794c1dfac917ffac1d5ff540f24)
- fixed iterating through every link for `config.underline_links`: [ce0175e](https://github.com/sharkthakftw/wikid/commit/ce0175e0041919ac37cb8baf687c024c11cdb7f5)
- optimised viewport link selection clamping via binary search: [340d997](https://github.com/sharkthakftw/wikid/commit/340d9978939413a0b5d8bb5aaed8553d315b8f50)
- eliminated intermedia `Vec<char>` and `String` allocations during search input keystrokes. now modifying `search_input` in-place: [d057e1e](https://github.com/sharkthakftw/wikid/commit/d057e1e45e37d0174315302aeb0e17297d2d0f96)
- optimised `recent_articles` retrieval with string slice set deduplication: [d88b318](https://github.com/sharkthakftw/wikid/commit/d88b318e8bc4ef138f215d8a0e8facb00d8668b4)
