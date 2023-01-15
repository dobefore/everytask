# for cross compiling
rustup target add aarch64-unknown-linux-musl
# It seems that there is no need to specify the sqlite library.
# It's so in version rusqlite 0.25.1 and with bundled feature in Cargo.toml .
# And tested on ubuntu 22.04 LTS

#export SQLITE3_LIB_DIR=$HOME/sql/lib
 #         export SQLITE3_INCLUDE_DIR=$HOME/sql/include
          export SQLITE3_STATIC=1
          export PATH="$HOME/aarch64-linux-musl-cross/bin:$PATH"
cargo build --target=aarch64-unknown-linux-musl --release