use crate::init_chain;

init_chain!({
    ident: Axelar,

    name: "axelar",

    logo: "https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg",

    sdk_ver: 45,

    rpc_url: "https://rpc.cosmos.directory/axelar",

    rest_url: "https://axelar-api.polkachu.com",
});

init_chain!({
    ident: Evmos,

    name: "evmos",

    logo: "https://assets.coingecko.com/coins/images/24023/large/evmos.png",

    sdk_ver: 45,

    decimals: 18,

    rpc_url: "https://rpc.cosmos.directory/evmos",

    rest_url: "https://evmos-api.polkachu.com",
});
