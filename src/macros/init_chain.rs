#[macro_export]
macro_rules! init_chain {
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
        pub struct $a<'a> {
            name: &'static str,
            sdk_ver: usize,
            prefix: &'static str,
            logo: &'static str,
            decimals: usize,
            rest_url: &'static str,
            rpc_url: &'static str,
            client: &'a reqwest::Client,
        }

        impl<'a> crate::fetch::chain::Chain for $a<'a> {
            fn name(&self) -> &'static str {
                self.name
            }
            fn sdk_version(&self) -> usize {
                self.sdk_ver
            }
            fn client(&self) -> &reqwest::Client {
                self.client
            }
            fn base_prefix(&self) -> &'static str {
                self.name
            }
            fn logo(&self) -> &'static str {
                self.logo
            }
            fn decimals(&self) -> usize {
                self.decimals
            }
            fn rest_api_url(&self) -> &'static str {
                self.rest_api_url
            }
            fn rpc_url(&self) -> &'static str {
                self.rpc_url
            }

        }
    };

    (
        {
            ident: $a:ident,
            name: $name:expr,
            logo: $logo:expr,
            sdk_ver: $sdk_ver:expr,
            decimals: $decimals:expr,
            rpc_url: $rpc_url:expr,
            rest_url: $rest_url:expr,
        }
    ) => {
        init_chain!(
            {
                ident: $a,
                name: $name,
                logo: $logo,
                prefix: $name,
                sdk_ver: $sdk_ver,
                decimals: $decimals,
                rpc_url: $rpc_url,
                rest_url: $rest_url,
            }
        );
    };

    (
        {
            ident: $a:ident,
            name: $name:expr,
            logo: $logo:expr,
            sdk_ver: $sdk_ver:expr,
            rpc_url: $rpc_url:expr,
            rest_url: $rest_url:expr,
        }
    ) => {
        init_chain!(
            {
                ident: $a,
                name: $name,
                logo: $logo,
                prefix: $name,
                sdk_ver: $sdk_ver,
                decimals: 6,
                rpc_url: $rpc_url,
                rest_url: $rest_url,
            }
        );
    };


    (
        {
            ident: $a:ident,
            name: $name:expr,
            logo: $logo:expr,
            prefix: $prefix:expr,
            sdk_ver: $sdk_ver:expr,
            rpc_url: $rpc_url:expr,
            rest_url: $rest_url:expr,
        }
    ) => {
        init_chain!(
            {
                ident: $a,
                name: $name,
                logo: $logo,
                prefix: $name,
                sdk_ver: $sdk_ver,
                decimals: 6,
                rpc_url: $rpc_url,
                rest_url: $rest_url,
            }
        );
    };
}
