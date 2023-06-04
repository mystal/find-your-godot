set windows-shell := ["nu", "-c"]

run *FLAGS:
  cargo run -- {{FLAGS}}
