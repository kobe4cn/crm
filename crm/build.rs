use std::fs;

use anyhow::Result;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    // let mut config = prost_build::Config::new();
    // config
    //     .out_dir("src/pb")
    //     .compile_protos(&["../protos/crm.proto"], &["../protos/"])?;

    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        .compile_protos(&["../protos/crm.proto"], &["../protos/"])?;
    Ok(())
}
