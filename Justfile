set windows-shell := ["powershell.exe", "-c"]

web:
  trunk serve

test:
    cargo test

cov:
    cargo llvm-cov --lcov --output-path lcov.info

interactive TEST:
    cargo test {{TEST}} -- --test-threads=1