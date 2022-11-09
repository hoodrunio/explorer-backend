/// # `chain!()`
///
/// - Generates code to add support for a new chain.
///
/// - You can leave `prefix` unset, if it is the same with `name`.
///
/// - You can leave `decimals` unset, if it is `6`.
///
/// - Add new chains inside `src/chains.rs` file.
///
/// # Usage
/// ```rs
/// // Axelar.
/// chain!({
///     ident: Axelar,
///     name: "axelar",
///     logo: "https://assets.coingecko.com/coins/images/24489/large/tsYr25vB_400x400.jpg",
///     sdk_ver: 45,
///     rpc_url: "https://rpc.cosmos.directory/axelar",
///     rest_url: "https://axelar-api.polkachu.com",
/// });
///
/// // Evmos.
/// chain!({
///     ident: Evmos,
///     name: "evmos",
///     logo: "https://assets.coingecko.com/coins/images/24023/large/evmos.png",
///     sdk_ver: 45,
///     decimals: 18,
///     rpc_url: "https://rpc.cosmos.directory/evmos",
///     rest_url: "https://evmos-api.polkachu.com",
/// });
///
#[macro_export]
macro_rules! chain {
    (
        {
            ident: $a:ident,
            name: $name:expr,
            logo: $logo:expr,
            prefix: $prefix:expr,
            sdk_ver: $sdk_ver:expr,
            decimals: $decimals:expr,
            rpc_url: $rpc_url:expr,
            rest_url: $rest_url:expr,
        }
    ) => {
        /// Struct represents a chain.
        pub struct $a {
            name: &'static str,
            sdk_ver: usize,
            prefix: &'static str,
            logo: &'static str,
            decimals: usize,
            rest_url: &'static str,
            rpc_url: &'static str,
            client: reqwest::Client,
        }

        impl crate::fetch::Chain for $a {
            fn name(&self) -> &'static str {
                self.name
            }

            fn sdk_version(&self) -> usize {
                self.sdk_ver
            }

            fn client(&self) -> reqwest::Client {
                self.client.clone()
            }

            fn base_prefix(&self) -> &'static str {
                self.prefix
            }

            fn logo(&self) -> &'static str {
                self.logo
            }

            fn decimals(&self) -> usize {
                self.decimals
            }

            fn rest_url(&self) -> &'static str {
                self.rest_url
            }

            fn rpc_url(&self) -> &'static str {
                self.rpc_url
            }

            fn new(client: reqwest::Client) -> Self {
                Self {
                    name: $name,
                    sdk_ver: $sdk_ver,
                    prefix: $prefix,
                    logo: $logo,
                    decimals: $decimals,
                    rest_url: $rest_url,
                    rpc_url: $rpc_url,
                    client
                }
            }

        }
    };
}
