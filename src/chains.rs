use crate::chain;

chain!({
    ident: Axelar,
    name: "axelar",
    logo: "https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg",
    sdk_ver: 45,
    rpc_url: "https://rpc.cosmos.directory/axelar",
    rest_url: "https://axelar-api.polkachu.com",
});

chain!({
    ident: Celestia,
    name: "celestia",
    logo: "",
    sdk_ver: 45,
    rpc_url: "https://rpc.celestia.testnet.run",
    rest_url: "https://api.celestia.testnet.run",
});

chain!({
    ident: Cosmos,
    name: "cosmos",
    logo: "https://assets.coingecko.com/coins/images/1481/large/cosmos_hub.png",
    sdk_ver: 45,
    rpc_url: "",
    rest_url: "",
});

chain!({
    ident: Evmos,
    name: "evmos",
    logo: "https://assets.coingecko.com/coins/images/24023/large/evmos.png",
    sdk_ver: 45,
    decimals: 18,
    rpc_url: "https://rpc.cosmos.directory/evmos",
    rest_url: "https://evmos-api.polkachu.com",
});

chain!({
    ident: Kyve,
    name: "kyve",
    logo: "https://assets.coingecko.com/coins/images/26229/large/78351592.png",
    sdk_ver: 45,
    rpc_url: "https://rpc.beta.kyve.network",
    rest_url: "https://api.beta.kyve.network",
});

chain!({
    ident: Osmosis,
    name: "osmosis",
    logo: "https://assets.coingecko.com/coins/images/16724/large/osmo.png",
    prefix: "osmo",
    sdk_ver: 45,
    rpc_url: "https://rpc.cosmos.directory/osmosis",
    rest_url: "https://rest.cosmos.directory/osmosis",
});

chain!({
    ident: Secret,
    name: "secret",
    logo: "https://raw.githubusercontent.com/cosmos/chain-registry/master/secretnetwork/images/scrt.svg",
    sdk_ver: 45,
    rpc_url: "https://rpc.cosmos.directory/secretnetwork",
    rest_url: "https://rest.cosmos.directory/secretnetwork",
});
