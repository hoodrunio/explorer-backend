pub mod account;
pub mod amount_util;
pub mod apr;
pub mod assets;
pub mod blocks;
pub mod chain;
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
pub mod ibc {
    pub mod core {
        pub mod client {
            pub mod v1 {
                tonic::include_proto!("ibc.core.client.v1");
            }
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

        pub mod spec {
            tonic::include_proto!("lavanet.lava.spec");
        }
    }
}