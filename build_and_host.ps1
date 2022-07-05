$env:RUSTFLAGS="--cfg=web_sys_unstable_apis"

cargo build --release --target wasm32-unknown-unknown

wasm-bindgen --out-dir .\target\wasm --target web .\target\wasm32-unknown-unknown\release\auto.wasm

Copy-Item .\index.html -Destination .\target\wasm

python -m http.server --directory .\target\wasm\