use anyhow::Result;
use crm::pb::User;
use prost::Message;
// pub mod pb {
//     use std::time::SystemTime;

//     use prost_types::Timestamp;

//     include!(concat!(env!("OUT_DIR"), "/crm.rs"));

// }
fn main() -> Result<()> {
    let user = User::new(1, "kevin", "kevin.yang@lianwei.com.cn");
    let encode = user.encode_to_vec();
    println!("{:?}", user);
    println!("{:?}", encode);
    Ok(())
}
