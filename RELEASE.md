## new feature(s)
### [wikipedia main page](https://en.wikipedia.org/wiki/Main_Page)
- action bar on the dashboard for each feed: [c983998](https://github.com/sharkthakftw/wikid/commit/c98399829b534e2c6da78eef55417d4b2b28f8bd) [93e8608](https://github.com/sharkthakftw/wikid/commit/93e8608a176567f5aaa19c1de060a000f9c2279e)
- modals for each feed: [829476b](https://github.com/sharkthakftw/wikid/commit/829476bd25c306a2cf1639cd547ca8fef9037db8)
#### featured
press `f` on the dashboard to open today's featured article
#### on this day
- see all past events, notable births and deaths which happened on today's date: [466c466](https://github.com/sharkthakftw/wikid/commit/466c46603d0ade8c85d2819b4a03481fb960b040)
- sub-tab switcher to switch categories (events, births, deaths, holidays): [58773d7](https://github.com/sharkthakftw/wikid/commit/58773d7b711bfc5dc7220986d4a2a77a65ad9846)
- link cycling for each headline: [d146cc6](https://github.com/sharkthakftw/wikid/commit/d146cc640a255732155047ae8b7c382e6adb0188)
- relative timestamps: [0c04005](https://github.com/sharkthakftw/wikid/commit/0c040052e8e1a3f683045cd96e6d9c478695cc19)
#### news
- see current news with convenient link cycling: [e78fbfe](https://github.com/sharkthakftw/wikid/commit/e78fbfed950b92fc065ea16b55949e851c3f49ce)
- ongoing events and recent deaths: [2f8354d](https://github.com/sharkthakftw/wikid/commit/2f8354d37127205b1872210bae98cf7501e9e0a9)
#### trending
- press `t` on the dashboard see the top 25 most read articles
## bug fix(es)
- fixed stale `local_matches` on window resizes: [0758e8c](https://github.com/sharkthakftw/wikid/commit/0758e8c4e5046d5d3bf926af0889b17251000ed3)
- preserve scroll offset for restored sessions and reopened tabs/splits: [4a5ae4b](https://github.com/sharkthakftw/wikid/commit/4a5ae4bf2233d939b79638414a0004152c22bf02)
- correctly compute bounds for mouse hit detection using `active_pane_rect()`: [a7cd8e2](https://github.com/sharkthakftw/wikid/commit/a7cd8e2fac457f8504eb93339c234900a7cac235)
- prevent async response races by tracking monotonic request IDs per pane: [89629a4](https://github.com/sharkthakftw/wikid/commit/89629a48777aeb2977248b5050f981b1ba42ec8c)
- windowed the search result rendering loop and implemented zero-allocation search line counting: [72ae288](https://github.com/sharkthakftw/wikid/commit/72ae2887b1ce67e9fba2f73bfae95b2c425c10f2)
- caching saved lists for faster tab bar lookup. cache is rebuilt on mutating the lists: [26291eb](https://github.com/sharkthakftw/wikid/commit/26291eb8f744e47b17f4f3eac0339c46942f21c9)
## qol/style change(s)
- replaced `alt-c` and `alt-shift-c` with `x` and `u` for closing and reopening pages respectively: [e333ca8](https://github.com/sharkthakftw/wikid/commit/e333ca8bdd67416eadacc04ad58b0471416f188c)
