pub mod account;
pub mod amount_util;
pub mod apr;
pub mod assets;
pub mod blocks;
pub mod chain;
pub mod chain_socket;
pub mod delegations;
pub mod delegators;
pub mod evm;
pub mod evm_socket_handler;
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

use crate::fetch::cosmos::base::query::v1beta1::PageResponse;
use base64::{
    Engine,
    engine::general_purpose::STANDARD,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaginationResponse {
    pub next_key: String,
    pub total: u64,
}

impl From<PageResponse> for PaginationResponse {
    fn from(value: PageResponse) -> Self {
        PaginationResponse {
            next_key: STANDARD.encode(value.next_key.as_slice()),
            total: value.total,
        }
    }
}

pub mod axelar {
    use sha2::digest::typenum::op;

    pub mod evm {
        pub mod v1beta1 {
            tonic::include_proto!("axelar.evm.v1beta1");
        }
    }

    pub mod tss {
        pub mod exported {
            pub mod v1beta1 {
                tonic::include_proto!("axelar.tss.exported.v1beta1");
            }
        }
    }

    pub mod utils {
        pub mod v1beta1 {
            tonic::include_proto!("axelar.utils.v1beta1");
        }
    }

    pub mod nexus {
        pub mod v1beta1 {
            tonic::include_proto!("axelar.nexus.v1beta1");
        }

        pub mod exported {
            pub mod v1beta1 {
                tonic::include_proto!("axelar.nexus.exported.v1beta1");
            }
        }
    }

    pub mod snapshot {
        pub mod v1beta1 {
            tonic::include_proto!("axelar.snapshot.v1beta1");
        }
    }
}

pub mod ibc {
    pub mod core {
        pub mod client {
            pub mod v1 {
                tonic::include_proto!("ibc.core.client.v1");
            }
        }
    }
}

pub mod c4e {
    pub mod minter {
        pub mod v1beta1 {
            tonic::include_proto!("chain4energy.c4echain.cfeminter");
        }
    }
}

pub mod cosmos {
    pub mod auth {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.auth.v1beta1");
        }
    }
    pub mod bank {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.bank.v1beta1");
        }
    }

    pub mod vesting {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.vesting.v1beta1");
        }
    }

    pub mod upgrade {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.upgrade.v1beta1");
        }
    }
    pub mod distribution {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.distribution.v1beta1");
        }
    }
    pub mod gov {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.gov.v1beta1");
        }

        pub mod v1 {
            tonic::include_proto!("cosmos.gov.v1");
        }
    }

    pub mod params {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.params.v1beta1");
        }
    }

    pub mod base {
        use sha2::digest::typenum::op;

        pub mod v1beta1 {
            tonic::include_proto!("cosmos.base.v1beta1");
        }
        pub mod query {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.base.query.v1beta1");
            }
        }

        pub mod abci {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.base.abci.v1beta1");
            }
        }

        pub mod tendermint {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.base.tendermint.v1beta1");
            }
        }
    }

    pub mod staking {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.staking.v1beta1");
        }
    }

    pub mod mint {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.mint.v1beta1");
        }
    }

    pub mod slashing {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.slashing.v1beta1");
        }
    }

    pub mod crypto {
        pub mod multisig {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.crypto.multisig.v1beta1");
            }
        }

        pub mod ed25519 {
            tonic::include_proto!("cosmos.crypto.ed25519");
        }

        pub mod secp256k1 {
            tonic::include_proto!("cosmos.crypto.secp256k1");
        }

        pub mod secp256r1 {
            tonic::include_proto!("cosmos.crypto.secp256r1");
        }
    }

    pub mod tx {
        pub mod v1beta1 {
            tonic::include_proto!("cosmos.tx.v1beta1");
        }

        pub mod signing {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.tx.signing.v1beta1");
            }
        }
    }


}

pub mod evmos {
    pub mod erc20 {
        pub mod v1 {
            tonic::include_proto!("evmos.erc20.v1");
        }
    }

    pub mod incentives {
        pub mod v1 {
            tonic::include_proto!("evmos.incentives.v1");
        }
    }

    pub mod inflation {
        pub mod v1 {
            tonic::include_proto!("evmos.inflation.v1");
        }
    }
}

pub mod osmosis {
    pub mod poolincentives {
        pub mod v1beta1 {
            tonic::include_proto!("osmosis.poolincentives.v1beta1");
        }
    }

    pub mod txfees {
        pub mod v1beta1 {
            tonic::include_proto!("osmosis.txfees.v1beta1");
        }
    }

    pub mod superfluid {
        tonic::include_proto!("osmosis.superfluid");

        pub mod v1beta1 {
            tonic::include_proto!("osmosis.superfluid.v1beta1");
        }
    }
}

pub mod umee {
    pub mod leverage {
        pub mod v1 {
            tonic::include_proto!("umee.leverage.v1");
        }
    }
}

pub mod gravity {
    pub mod v1 {
        tonic::include_proto!("gravity.v1");
    }
}

pub mod quicksilver {
    pub mod interchainstaking {
        pub mod v1 {
            tonic::include_proto!("quicksilver.interchainstaking.v1");
        }
    }
}

pub mod kyve {
    pub mod global {
        pub mod v1beta1 {
            tonic::include_proto!("kyve.global.v1beta1");
        }
    }
}


pub mod cosmwasm {
    pub mod wasm {
        pub mod v1 {
            tonic::include_proto!("cosmwasm.wasm.v1");
        }
    }
}

pub mod lavanet {
    pub mod lava {
        pub mod plans {
            tonic::include_proto!("lavanet.lava.plans");
        }

        pub mod projects {
            tonic::include_proto!("lavanet.lava.projects");
        }

        pub mod spec {
            tonic::include_proto!("lavanet.lava.spec");
        }
    }
}

pub mod tendermint {
    pub mod types {
        tonic::include_proto!("tendermint.types");
    }

    pub mod abci {
        tonic::include_proto!("tendermint.abci");
    }

    pub mod crypto {
        tonic::include_proto!("tendermint.crypto");
    }

    pub mod version {
        tonic::include_proto!("tendermint.version");
    }

    pub mod p2p {
        tonic::include_proto!("tendermint.p2p");
    }
}