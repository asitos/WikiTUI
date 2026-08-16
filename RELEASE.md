## optimised dependencies: [a78f03f](https://github.com/sharkthakftw/wikid/commit/a78f03f8aeade7682cd088e6d018a40c3ef9f6ee), [684efe1](https://github.com/sharkthakftw/wikid/commit/684efe1d4ec3cca3532e49533b6372b854137a6d), [34e16b9](https://github.com/sharkthakftw/wikid/commit/34e16b9643b141e4098da523a27aa07380a85616)
- removed `chrono`
- replaced `rand` with `fastrand`
- downgraded `unicode-width`
- replaced `scraper` with `tl` (42 less crates)
- replaced `reqwest` + `tokio` with `ureq` (44 less crates)

significant reduction in compile times
