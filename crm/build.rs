use anyhow::Result;

use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    // builder.out_dir("src/pb").compile_protos(
    //     &[
    //         "../protos/user_stats/messages.proto",
    //         "../protos/user_stats/rpc.proto",
    //     ],
    //     &["../protos"],
    // )?;

    builder
        .out_dir("src/pb")
        .with_derive_builder(&["WelcomeRequest", "RecallRequest", "RemindRequest"], None)
        .with_sqlx_from_row(&["User"], None)
        .with_field_attributes(
            &["User.name", "User.email", "RawQueryRequest.query"],
            &[r#"#[builder(setter(into))]"#],
        )
        .with_field_attributes(
            &["TimeQuery.before", "TimeQuery.after"],
            &[r#"#[builder(setter(into,strip_option))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.timestamps"],
            &[r#"#[builder(setter(each(name="timestamp", into)))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.ids"],
            &[r#"#[builder(setter(each(name="id", into)))]"#],
        )
        .compile_protos(
            &["../protos/crm/messages.proto", "../protos/crm/rpc.proto"],
            &["../protos"],
        )?;
    Ok(())
}
