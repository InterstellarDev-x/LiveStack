Cargo Lock / Blocking Mental Model
1. Cargo Uses Shared Global Cache
All your Rust projects share downloaded crates/build cache.
2. Only One Cargo Process Can Mutate It at a Time
To prevent corruption, Cargo locks the cache.
3. “Blocking Waiting for File Lock” Means

Another Cargo/Cargo-related process is already using it.

It is not an error.

Common Hidden Cause
4. Your IDE Often Runs Cargo Automatically

Usually via Rust Analyzer:

cargo check
background diagnostics
autocomplete/type checking