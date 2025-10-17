[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/saf59/cx58)
<p>
	<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 400 400"><path d="M22 14a5 5 0 0 0-4.35-4.94A5.999 5.999 0 0 0 6 11a4 4 0 1 0 0 8h4" style="fill:none;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round;stroke:#40c0e7;stroke-opacity:1;stroke-miterlimit:4" transform="scale(16.66667)"/><path d="m287.59 282.375-10.586 20.84c-.156.293-.43.508-.762.586s-.664 0-.918-.215c-5.937-4.688-16.113-8.242-23.691-8.242-14.649 0-24.903 9.922-24.903 24.12 0 14.2 10.254 24.122 24.903 24.122 7.93 0 16.074-2.54 21.23-6.66.235-.215.586-.274.918-.235.313.079.586.274.742.567l11.739 21.387a1.115 1.115 0 0 1-.274 1.425c-8.886 7.032-20.156 10.45-34.453 10.45-30.469 0-52.578-21.485-52.578-51.055 0-29.59 22.11-51.055 52.578-51.055 13.926 0 25.977 4.238 35.781 12.598.391.351.508.898.274 1.367M390.023 273.965l-30.96 45.062 30.8 45.75c.45.684.504 1.551.114 2.27a2.2 2.2 0 0 1-1.942 1.168h-25.992a2.19 2.19 0 0 1-1.82-.961l-17.157-24.961-17.156 24.969c-.41.601-1.09.96-1.816.96h-25.985c-.812 0-1.57-.449-1.941-1.167a2.2 2.2 0 0 1 .11-2.266l30.792-45.762-30.96-45.062a2.22 2.22 0 0 1-.13-2.277 2.22 2.22 0 0 1 1.95-1.176h27.308c.739 0 1.422.363 1.832.972l15.996 23.782 16.008-23.782c.41-.609 1.09-.972 1.828-.972h27.301c.82 0 1.567.457 1.961 1.175a2.23 2.23 0 0 1-.14 2.278m0 0" style="stroke:none;fill-rule:nonzero;fill:#40c0e7;fill-opacity:1"/></svg>
</p>

# Leptos Construct-X/5.8 Starter Template

This is a template for use with the [Leptos](https://github.com/leptos-rs/leptos) web framework and the [cargo-leptos](https://github.com/akesson/cargo-leptos) tool using [Axum](https://github.com/tokio-rs/axum).

## Creating your template repo

If you don't have `cargo-leptos` installed you can install it with

```bash
cargo install cargo-leptos --locked
```

Then run
```bash
cargo leptos new --git https://github.com/saf59/cx58
```

to generate a new project template.

```bash
cd cx58
```

to go to your newly created project.
Feel free to explore the project structure, but the best place to start with your application code is in `src/app.rs`.
Additionally, Cargo.toml may need updating as new versions of the dependencies are released, especially if things are not working after a `cargo update`.

## Running your project

```bash
cargo leptos watch
```

## Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
2. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
3. `cargo install cargo-generate` - install `cargo-generate` binary (should be installed automatically in future)
4. `npm install -g sass` - install `dart-sass` (should be optional in future
5. Run `npm install` in end2end subdirectory before test

## Compiling for Release
```bash
cargo leptos build --release
```

Will generate your server binary in target/release and your site package in target/site

## Testing Your Project
```bash
cargo leptos end-to-end
```

```bash
cargo leptos end-to-end --release
```

Cargo-leptos uses Playwright as the end-to-end test tool.
Tests are located in end2end/tests directory.

## Executing a Server on a Remote Machine Without the Toolchain
After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/server/release`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:
```text
cx58
site/
```
Set the following environment variables (updating for your project as needed):
```sh
export LEPTOS_OUTPUT_NAME="cx58"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
```
Finally, run the server binary.

## Licensing

This template itself is released under the Unlicense. You should replace the LICENSE for your own application with an appropriate license if you plan to release it publicly.
