# Zero to Production in Rust

[![CI/CD Prechecks](https://github.com/damccull/zero2prod/actions/workflows/general.yml/badge.svg)](https://github.com/damccull/zero2prod/actions/workflows/general.yml)
[![Security audit](https://github.com/damccull/zero2prod/actions/workflows/audit.yml/badge.svg)](https://github.com/damccull/zero2prod/actions/workflows/audit.yml)
[![Code coverage](https://github.com/damccull/zero2prod/actions/workflows/coverage.yml/badge.svg)](https://github.com/damccull/zero2prod/actions/workflows/coverage.yml)
[![Fly Deploy](https://github.com/damccull/zero2prod/actions/workflows/fly.yml/badge.svg)](https://github.com/damccull/zero2prod/actions/workflows/fly.yml)

This is my modified code for Luca Palmieri's book, [_Zero to Production in Rust_](https://www.zero2prod.com/).
I am using [Axum](https://docs.rs/axum/latest/axum/) in this version instead of
[actix_web](https://docs.rs/actix-web/4.3.1/actix_web/).
I also have a few modifications that stray from the book for security, efficiency,
documentation, or just to make the code easier on my eyes. This repo is also deployed to
[fly.io](https://fly.io) rather than [Digital Ocean](https://digitalocean.com).

Feel free to use or steal any of the code in this repo.
