set windows-shell := ["nu", "-c"]

run *FLAGS:
  cargo run -- {{FLAGS}}

run-rel *FLAGS:
  cargo run --release -- {{FLAGS}}
