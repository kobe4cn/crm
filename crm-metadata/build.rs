use anyhow::Result;

use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        // .with_serde(
        //     &["Content", "Publisher", "ContentType"],
        //     true,
        //     true,
        //     Some(&[r#"#[serde(rename_all = "camelCase")]"#]),
        // )
        .with_derive_builder(&["Content", "Publisher"], None)
        .with_sqlx_from_row(&["Content"], None)
        .compile_protos(
            &[
                "../protos/crm_metadata/messages.proto",
                "../protos/crm_metadata/rpc.proto",
            ],
            &["../protos"],
        )?;
    Ok(())
}
