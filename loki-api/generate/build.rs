use prost_build::compile_protos;
use std::io;
fn main() -> io::Result<()> {
    compile_protos(&["proto/logproto.proto"], &["proto/"])?;
    Ok(())
}
