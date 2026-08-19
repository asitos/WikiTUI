## ui overhaul
### content
- added a colored section heading marker (▍): [f3f0bee](https://github.com/sharkthakftw/wikid/commit/f3f0bee59eae008c101eec81555b931d6c6f9000)
- inline citation markers are now in superscript: [c19007d](https://github.com/sharkthakftw/wikid/commit/c19007d314b6ca98dcf16dd0563cdc7a6038f831)
- domain chip after external link text: [02fa8c5](https://github.com/sharkthakftw/wikid/commit/02fa8c5c1bc4fd31ae4f7902bc9c1221b637ad1b)
- scroll indicator on the right edge: [68b979a](https://github.com/sharkthakftw/wikid/commit/68b979adf2598f346818c9340f7a73559b8786ae)
- added line numbers in codeblocks: [a1784ae](https://github.com/sharkthakftw/wikid/commit/a1784aeb7ad610012e8f428046a076d045e7f5d5)
- added a spinner animation during article loading: [c0f1029](https://github.com/sharkthakftw/wikid/commit/c0f102988651c58302b08525e91b99b565af3fd9)
- row highlighting for focused search result: [13a7eda](https://github.com/sharkthakftw/wikid/commit/13a7edacb689aade7727a8b151b5328b87b6519f)
- table of contents section numbering: [a4d1d72](https://github.com/sharkthakftw/wikid/commit/a4d1d725714b10acd58fb806c45ab1ed21ace6fe)
### launch page
- wikid logo: [b33a686](https://github.com/sharkthakftw/wikid/commit/b33a686e25e6350c18a6eb59d8d95283195f0c90)
- wikipedia statistics: [cbd15e8](https://github.com/sharkthakftw/wikid/commit/cbd15e8523e211c0562766092d310c943f2891bf)
- continue reading section (stored in `recent_articles.json`): [e54ae6d](https://github.com/sharkthakftw/wikid/commit/e54ae6dbcb2a79502dc27e2ecd23b0b3adf9964f)
- rotating quotes under the logo: [f35727a](https://github.com/sharkthakftw/wikid/commit/f35727ac3c5a8f079a142d35ef2af4aa7408abd4)
### tab bar
- new pilled tab design and added icons: [297cc54](https://github.com/sharkthakftw/wikid/commit/297cc54945e7c624243b169af08278f66dff5995)
- show `*` on article title if it's saved in a list: [8c76e29](https://github.com/sharkthakftw/wikid/commit/8c76e294f17e65a7a1a04fe9f1ac2ba8628e094d)
### status bar
- redesigned with 3 segments (mode, hint text, position): [d2cace5](https://github.com/sharkthakftw/wikid/commit/d2cace5b70a9d03b09eeb8ad72caaa59c8d3276b)
- article breadcrumb trail: [de70733](https://github.com/sharkthakftw/wikid/commit/de7073347d5ca1ae4058eca148d207f19eeaeed8)
### modals
- added icons: [5fe3e32](https://github.com/sharkthakftw/wikid/commit/5fe3e32868aa7ce318ea70d6a43416d0c0e9205f)
- rounded borders: [a2db870](https://github.com/sharkthakftw/wikid/commit/a2db870ed59c4e97f4dedd82906c565c74ee894c)
## new config options
### \[general]
- `confirm_quit`, enable/disable confirmation prompt on quit: [3eb251c](https://github.com/sharkthakftw/wikid/commit/3eb251ca146b8bc5e32b9ecfd416e6f32f2df89d)
### \[reader]
- `toc_section_numbers`, enable/disable table of contents section numbers: [a4d1d72](https://github.com/sharkthakftw/wikid/commit/a4d1d725714b10acd58fb806c45ab1ed21ace6fe)
### \[ui]
- `rounded_borders`, enable/disable rounded borders: [a2db870](https://github.com/sharkthakftw/wikid/commit/a2db870ed59c4e97f4dedd82906c565c74ee894c)
- `icons`, enable/disable icons in wikid: [e8217f3](https://github.com/sharkthakftw/wikid/commit/e8217f3a919c0a3d7bc8a77d5daccbfa3507d91c)
- `scroll_indicator`, enable/disable the scroll indicator on the right edge: [debe9bb](https://github.com/sharkthakftw/wikid/commit/debe9bb6bfb4148adfeef93565f5cbc9add1eac0)
- `heading_marker`, enable/disable heading markers: [b16630e](https://github.com/sharkthakftw/wikid/commit/b16630eb7257b7d84774d1ecb7694574f7e0e469)
## bug fix(es)
- filtered out raw LaTeX and math SVG fallback tags: [8adca07](https://github.com/sharkthakftw/wikid/commit/8adca07167649f2a3ccc4728b9b559eb622aa527)
