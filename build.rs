fn main() {
    tonic_build::configure()
    .build_server(false)
        .build_client(true)
        .compile(&[
            "proto/cosmos/gov/v1beta1/gov.proto",
            "proto/cosmos/gov/v1beta1/query.proto",
            "cosmos/base/query/v1beta1/pagination.proto",
            "cosmos/base/v1beta1/coin.proto",
            "cosmos/params/v1beta1/params.proto"
        ], &["proto"])
        .unwrap();
}