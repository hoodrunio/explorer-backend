use prost_wkt_build::{FileDescriptorSet, Message};
use std::env;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let descriptor_file = out.join("descriptor.bin");

    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
        .extern_path(".google.protobuf.Duration", "::prost_wkt_types::Duration")
        .build_server(false)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_file)
        .compile(
            &[
                "cosmos/gov/v1beta1/gov.proto",
                "cosmos/gov/v1beta1/query.proto",
                "cosmos/gov/v1/query.proto",
                "cosmos/gov/v1/gov.proto",
                "cosmos/gov/v1/tx.proto",
                "cosmos/bank/v1beta1/bank.proto",
                "cosmos/base/query/v1beta1/pagination.proto",
                "cosmos/base/v1beta1/coin.proto",
                "cosmos/params/v1beta1/params.proto",
                "cosmos/upgrade/v1beta1/upgrade.proto",
                "cosmos/distribution/v1beta1/distribution.proto",
                "ibc/core/client/v1/client.proto",
                "evmos/erc20/v1/erc20.proto",
                "evmos/incentives/v1/incentives.proto",
                "osmosis/pool-incentives/v1beta1/gov.proto",
                "umee/leverage/v1/tx.proto",
                "gravity/v1/types.proto",
                "quicksilver/interchainstaking/v1/proposals.proto",
                "kyve/global/v1beta1/tx.proto",
            ],
            &["protos"],
        )
        .unwrap();

    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();

    prost_wkt_build::add_serde(out, descriptor);
}
