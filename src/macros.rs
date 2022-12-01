/// Initializes a new chain.
#[macro_export]
macro_rules! init_chain {
    (
        name: $name:expr,
        gecko: $gecko:expr,
        base_prefix: $base_prefix:expr,
        valoper_prefix:$valoper_prefix:expr,
        cons_prefix: $cons_prefix:expr,
        main_denom: $main_denom:expr,
        rpc_url: $rpc_url:expr,
        jsonrpc_url: $jsonrpc_url:expr,
        rest_url: $rest_url:expr,
        wss_url: $wss_url:expr,
        sdk_version: $sdk_version:expr,
        decimals_pow: $decimals_pow:expr,
        client: $client:expr,
    ) => {
        Chain::new(crate::chain::ChainConfig {
            name: $name,
            gecko: $gecko,
            base_prefix: $base_prefix,
            valoper_prefix: $valoper_prefix,
            cons_prefix: $cons_prefix,
            main_denom: $main_denom,
            rpc_url: $rpc_url,
            jsonrpc_url: $jsonrpc_url,
            rest_url: $rest_url,
            wss_url: $wss_url,
            sdk_version: $sdk_version,
            decimals_pow: $decimals_pow,
            client: $client,
            data: ChainData::new($name),
        })
    };
}
