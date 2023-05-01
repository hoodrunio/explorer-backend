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
                "axelar/evm/v1beta1/service.proto",
                "axelar/tss/exported/v1beta1/types.proto",
                "axelar/utils/v1beta1/threshold.proto",
                "axelar/nexus/v1beta1/service.proto",
                "axelar/nexus/exported/v1beta1/types.proto",
                "cosmos/gov/v1beta1/gov.proto",
                "cosmos/gov/v1beta1/query.proto",
                "cosmos/gov/v1/query.proto",
                "cosmos/gov/v1/gov.proto",
                "cosmos/gov/v1/tx.proto",
                "cosmos/auth/v1beta1/auth.proto",
                "cosmos/auth/v1beta1/genesis.proto",
                "cosmos/auth/v1beta1/query.proto",
                "cosmos/auth/v1beta1/tx.proto",
                "cosmos/vesting/v1beta1/vesting.proto",
                "cosmos/vesting/v1beta1/tx.proto",
                "cosmos/bank/v1beta1/bank.proto",
                "cosmos/base/query/v1beta1/pagination.proto",
                "cosmos/base/v1beta1/coin.proto",
                "cosmos/params/v1beta1/params.proto",
                "cosmos/params/v1beta1/query.proto",
                "cosmos/upgrade/v1beta1/upgrade.proto",
                "cosmos/upgrade/v1beta1/tx.proto",
                "cosmos/distribution/v1beta1/distribution.proto",
                "cosmos/bank/v1beta1/query.proto",
                "cosmos/staking/v1beta1/staking.proto",
                "cosmos/mint/v1beta1/query.proto",
                "cosmos/staking/v1beta1/query.proto",
                "cosmos/slashing/v1beta1/query.proto",
                "cosmos/tx/v1beta1/service.proto",
                "cosmos/tx/signing/v1beta1/signing.proto",
                "cosmos/crypto/multisig/v1beta1/multisig.proto",
                "cosmos/base/abci/v1beta1/abci.proto",
                "ibc/core/client/v1/client.proto",
                "evmos/erc20/v1/erc20.proto",
                "evmos/incentives/v1/incentives.proto",
                "evmos/inflation/v1/query.proto",
                "osmosis/pool-incentives/v1beta1/gov.proto",
                "umee/leverage/v1/tx.proto",
                "gravity/v1/types.proto",
                "quicksilver/interchainstaking/v1/proposals.proto",
                "kyve/global/v1beta1/tx.proto",
                "cosmwasm/wasm/v1/proposal.proto",
                "osmosis/superfluid/v1beta1/gov.proto",
                "osmosis/superfluid/superfluid.proto",
                "osmosis/txfees/v1beta1/gov.proto",
                "lavanet/lava/plans/plans_add_proposal.proto",
                "lavanet/lava/spec/spec_add_proposal.proto",
                "lavanet/lava/projects/query.proto",
                "tendermint/types/types.proto",
                "tendermint/abci/types.proto",
                "tendermint/crypto/keys.proto",
                "tendermint/version/types.proto",
            ],
            &["protos", "protos/lavanet/lava"],
        )
        .unwrap();

    let descriptor_bytes = std::fs::read(descriptor_file).unwrap();
    let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();

    prost_wkt_build::add_serde(out, descriptor);
}
