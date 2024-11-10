use anyhow::Result;
use vergen_git2::{Emitter, Git2Builder};

fn main() -> Result<()> {
    // Use vergen-git2 to pass in an env var with the git short SHA1 hash of the current commit.
    let git2 = Git2Builder::default()
        .sha(true)
        .build()?;
    Emitter::default()
        .add_instructions(&git2)?
        .emit()?;

    Ok(())
}
