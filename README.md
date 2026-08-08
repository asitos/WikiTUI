# wikid

![crates.io](https://img.shields.io/crates/v/wikid.svg)
[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg?logo=rust)](https://www.rust-lang.org/)

a feature-rich terminal wikipedia client.

## features

- **tabs and splits**: work with multiple articles side-by-side or in tabs.
- **vim-like navigation**: intuitive keybindings for fast scrolling, jumping, and pane movement.
- **zen mode (`z`)**: distraction-free reading canvas with no borders, tab bars, or status indicators.
- **table of contents (`o`)**: pop-up article outline modal with instant heading jumping (`enter`).
- **random article discovery (`r`)**: instantly discover and load random wikipedia articles in new tabs.
- **in-page substring search (`/`)**: exact match highlighting and jumping through matches (`n` / `N`).

## installation

### from crates.io

requires [rust + cargo](https://www.rust-lang.org/tools/install) to be installed:

```bash
cargo install wikid
```

### build from source
```bash
git clone https://github.com/sharkthakftw/wikid.git
cd wikid
cargo build --release

# install the binary to your system path
sudo install -Dm755 target/release/wikid /usr/bin/wikid
```

## keybindings

| action | keybinding | description |
| :--- | :---: | :--- |
| **scroll down / up** | `j` / `k` | scroll line down / up |
| **page down / up** | `f` / `b` | scroll full page down / up |
| **jump to top / bottom** | `g` / `G` | jump directly to article top / bottom |
| **search wikipedia** | `ctrl-s` | open search modal (opens in new tab) |
| **edit search** | `i` | edit current search query in active tab |
| **in-page search** | `/` | in-page text search with match highlighting |
| **next / prev match** | `n` / `N` | jump to next / previous in-page search match |
| **table of contents** | `o` | open centered article outline modal |
| **zen mode** | `z` | toggle minimalist borderless reading view |
| **random article** | `r` | fetch & open random wikipedia article in new tab |
| **heading jump** | `]` / `[` | jump to next / previous section heading |
| **link navigation** | `tab` / `shift-tab` | focus next / previous article link |
| **open link** | `enter` | open link in active pane |
| **open link in new tab** | `t` or `alt-enter` | open link in a new tab |
| **open link in split** | `s` / `v` | open link in horizontal (`s`) or vertical (`v`) split |
| **split pane** | `ctrl-w` `s`/`v` | split active pane horizontally (`s`) or vertically (`v`) |
| **navigate panes** | `ctrl-h/j/k/l` | switch focus between split panes |
| **close pane** | `alt-c` | close active pane |
| **new tab** | `alt-t` | create a new empty tab |
| **switch tabs** | `alt-h` / `alt-l` | switch to previous / next tab |
| **help popup** | `?` | toggle keybindings cheat sheet |
| **quit** | `q` | exit wikid |


## license

distributed under the [MIT license](LICENSE).
