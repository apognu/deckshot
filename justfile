prepare:
  cargo check --manifest-path backend/Cargo.toml
  cargo clippy --manifest-path backend/Cargo.toml
  pip install --upgrade -r requirements.txt -t py_modules
