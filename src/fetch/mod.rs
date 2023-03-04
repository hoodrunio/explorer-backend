pub mod account;
pub mod amount_util;
pub mod apr;
pub mod assets;
pub mod blocks;
pub mod delegations;
pub mod delegators;
pub mod evm;
pub mod heartbeats;
pub mod others;
pub mod params;
pub mod proposals;
pub mod requests;
pub mod socket;
pub mod tokenomics;
pub mod transactions;
pub mod utils;
pub mod validators;

pub mod cosmos {
    pub mod gov {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.gov.v1beta1");
        }
    }

    pub mod params {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.params.v1beta1");
        }
    }

    pub mod base {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.base.v1beta1");
        }
        pub mod query {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.base.query.v1beta1");
            }
        }
    }
}