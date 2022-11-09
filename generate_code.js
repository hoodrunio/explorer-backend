#!/usr/bin/env node
const fs = require('fs/promises')

/** Parses the content of `.CHAINS.JSON` file and returns an array of chain parameters. */
const parseChainsParamsFromJson = async () => {
    const filePath = '.CHAINS.JSON'

    const fileContent = await fs.readFile(filePath, { encoding: 'utf-8' })

    return JSON.parse(fileContent)
}

/** Writes given function names to `./src/server/web_server.rs`.*/
const saveWebServerFile = async (functionNamesArray) => {
    const webServerFilePath = `./src/server/web_server.rs`;
    const newContent = functionNamesArray.map(fnName => `            .service(${fnName})`).join('\n')

    const oldWebServerFileContent = await fs.readFile(webServerFilePath, { encoding: 'utf-8' });

    const [oldWebServerFileConstantPart1, rest] = oldWebServerFileContent.split('            // Common services.')
    const [_, oldWebServerFileConstantPart2] = rest.split('})')

    const webServerFileContent = `${oldWebServerFileConstantPart1}            // Common services.\n${newContent}\n    })${oldWebServerFileConstantPart2}`


    await fs.writeFile(webServerFilePath, webServerFileContent, { encoding: 'utf-8' })
}

/** Parses the content of `.CHAINS` file and returns an array of `Chain` objects. */
const saveRestFiles = async ({
    macros,
    blocks,
    transactions,
    params,
    validators,
    delegators,
    tokenomics,
    proposals,
    delegations,
    stakingPool,
    signingInfo,
}) => {
    const folderPath = './src/server/routes/common/rest';
    const macrosPath = './src/chains.rs';
    const blocksPath = `${folderPath}/blocks.rs`;
    const transactionsPath = `${folderPath}/transactions.rs`;
    const paramsPath = `${folderPath}/params.rs`;
    const validatorsPath = `${folderPath}/validators.rs`;
    const delegatorsPath = `${folderPath}/delegators.rs`;
    const tokenomicsPath = `${folderPath}/tokenomics.rs`;
    const proposalsPath = `${folderPath}/proposals.rs`;
    const delegationsPath = `${folderPath}/delegations.rs`;
    const stakingPoolPath = `${folderPath}/staking_pool.rs`;
    const signingInfoPath = `${folderPath}/signing_info.rs`;

    const modFilePath = `${folderPath}/mod.rs`;
    const modFileContent = `
mod macros;
mod blocks;
mod transactions;
mod params;
mod validators;
mod delegators;
mod tokenomics;
mod proposals;
mod delegations;
mod staking_pool;
mod signing_info;

pub use macros::*;
pub use blocks::*;
pub use transactions::*;
pub use params::*;
pub use validators::*;
pub use delegators::*;
pub use tokenomics::*;
pub use proposals::*;
pub use delegations::*;
pub use staking_pool::*;
pub use signing_info::*;

pub mod necessities {
    pub use crate::fetch::types::*;
    pub use actix_web::{get, web, Responder};
    pub use web::{Data, Json, Path};

    pub use crate::{
        fetch::{types::PaginationConfig, Chain},
        server::state::ServerState,
    };
}
`.slice(1);

    await Promise.all(
        [
            fs.writeFile(macrosPath, `use crate::chain;\n\n${macros}`, { encoding: 'utf-8' }),
            fs.writeFile(blocksPath, `use super::necessities::*;\n\n${blocks}`, { encoding: 'utf-8' }),
            fs.writeFile(transactionsPath, `use super::necessities::*;\n\n${transactions}`, { encoding: 'utf-8' }),
            fs.writeFile(paramsPath, `use super::necessities::*;\n\n${params}`, { encoding: 'utf-8' }),
            fs.writeFile(validatorsPath, `use super::necessities::*;\n\n${validators}`, { encoding: 'utf-8' }),
            fs.writeFile(delegatorsPath, `use super::necessities::*;\n\n${delegators}`, { encoding: 'utf-8' }),
            fs.writeFile(tokenomicsPath, `use super::necessities::*;\n\n${tokenomics}`, { encoding: 'utf-8' }),
            fs.writeFile(proposalsPath, `use super::necessities::*;\n\n${proposals}`, { encoding: 'utf-8' }),
            fs.writeFile(delegationsPath, `use super::necessities::*;\n\n${delegations}`, { encoding: 'utf-8' }),
            fs.writeFile(stakingPoolPath, `use super::necessities::*;\n\n${stakingPool}`, { encoding: 'utf-8' }),
            fs.writeFile(signingInfoPath, `use super::necessities::*;\n\n${signingInfo}`, { encoding: 'utf-8' }),
            fs.writeFile(modFilePath, modFileContent, { encoding: 'utf-8' }),
        ]
    );
}



class Chain {
    constructor({ name, logo, prefix, decimals, sdk_ver, rpc_url, rest_url }) {
        this.ident = name[0].toUpperCase() + name.slice(1);
        this.prefix = prefix ?? name;
        this.name = name;
        this.logo = logo;
        this.sdk_ver = sdk_ver;
        this.decimals = decimals ?? 6;
        this.rpc_url = rpc_url;
        this.rest_url = rest_url;
    }

    getRestFunctionNames () {
        return [`block_by_height_${this.name}`, `block_by_hash_${this.name}`, `blockchain_by_heights_${this.name}`, `tx_by_hash_${this.name}`, `txs_on_latest_block_${this.name}`, `txs_by_height_${this.name}`, `txs_of_sender_${this.name}`, `txs_of_recipient_${this.name}`, `staking_params_${this.name}`, `tally_params_${this.name}`, `voting_params_${this.name}`, `deposit_params_${this.name}`, `slashing_params_${this.name}`, `validator_${this.name}`, `validator_commission_${this.name}`, `validator_rewards_${this.name}`, `validators_bonded_${this.name}`, `validators_unbonded_${this.name}`, `validators_unbonding_${this.name}`, `validators_unspecified_${this.name}`, `validators_of_delegator_${this.name}`, `validator_delegator_pair_${this.name}`, `delegator_rewards_${this.name}`, `delegator_withdraw_address_${this.name}`, `supply_${this.name}`, `supplies_${this.name}`, `inflation_${this.name}`, `proposals_passed_${this.name}`, `proposals_voting_${this.name}`, `proposals_failed_${this.name}`, `proposals_rejected_${this.name}`, `proposals_unspecified_${this.name}`, `proposal_deposits_${this.name}`, `proposal_details_${this.name}`, `proposal_tally_${this.name}`, `proposal_votes_${this.name}`, `proposal_vote_${this.name}`, `proposal_deposit_${this.name}`, `delegations_${this.name}`, `unbonding_delegations_${this.name}`, `redelegations_${this.name}`, `staking_pool_${this.name}`, `signing_${this.name}`,]
    }

    restGenerateMacro () {
        return `
// Defining ${this.name.toUpperCase()}.

chain!({
    ident: ${this.ident},
    name: "${this.name}",
    logo: "${this.logo}",
    prefix: "${this.prefix}",
    sdk_ver: ${this.sdk_ver},
    decimals: ${this.decimals},
    rpc_url: "${this.rpc_url}",
    rest_url: "${this.rest_url}",
});

`.slice(1)
    }

    restGenerateBlockPaths () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/block/{height}")]
pub async fn block_by_height_${this.name}(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();
    
    let chain = &server_state.chains.${this.name};
    
    let resp = chain.get_block_by_height(Some(height)).await;
    
    Json(resp)
}
    
#[get("${this.name}/block/{hash}")]
pub async fn block_by_hash_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();
    
    let chain = &server_state.chains.${this.name};
    
    let resp = chain.get_block_by_hash(&hash).await;
    
    Json(resp)
}
    
#[get("${this.name}/blockchain/{min_height}/{max_height}")]
pub async fn blockchain_by_heights_${this.name}(path: Path<(u64, u64)>, server_state: Data<ServerState>) -> impl Responder {
    let (min_height, max_height) = path.into_inner();
    
    let chain = &server_state.chains.${this.name};
    
    let resp = chain.get_blockchain(min_height, max_height).await;
    
    Json(resp)
}

`.slice(1)
    }

    restGenerateTransactionPaths () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/tx/{hash}")]
pub async fn tx_by_hash_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let hash = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_tx_by_hash(&hash).await;

    Json(resp)
}

#[get("${this.name}/txs-on-latest-block")]
pub async fn txs_on_latest_block_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_txs_by_height(None, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/txs-on-block/{heigth}")]
pub async fn txs_by_height_${this.name}(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let height = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_txs_by_height(Some(height), PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("${this.name}/txs-of-sender/{address}")]
pub async fn txs_of_sender_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_txs_by_sender(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

#[get("${this.name}/txs-of-recipient/{address}")]
pub async fn txs_of_recipient_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_txs_by_recipient(&address, PaginationConfig::new().limit(100).offset(0).reverse()).await;

    Json(resp)
}

`.slice(1)
    }


    restGenerateParams () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/params/staking")]
pub async fn staking_params_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_staking_params().await;

    Json(resp)
}

#[get("${this.name}/params/tally")]
pub async fn tally_params_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_tally_params().await;

    Json(resp)
}

#[get("${this.name}/params/voting")]
pub async fn voting_params_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_voting_params().await;

    Json(resp)
}

#[get("${this.name}/params/deposit")]
pub async fn deposit_params_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_deposit_params().await;

    Json(resp)
}

#[get("${this.name}/params/slashing")]
pub async fn slashing_params_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_slashing_params().await;

    Json(resp)
}

`.slice(1)
    }


    restGenerateValidators () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/validator/{address}")]
pub async fn validator_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_validator(&validator_address).await;

    Json(resp)
}

#[get("${this.name}/validator-commission/{address}")]
pub async fn validator_commission_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_validator_commission(&validator_address).await;

    Json(resp)
}

#[get("${this.name}/validator-rewards/{address}")]
pub async fn validator_rewards_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let validator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_validator_rewards(&validator_address).await;

    Json(resp)
}

#[get("${this.name}/validators-bonded")]
pub async fn validators_bonded_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_validators_bonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/validators-unbonded")]
pub async fn validators_unbonded_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_validators_unbonded(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/validators-unbonding")]
pub async fn validators_unbonding_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_validators_unbonding(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/validators-unspecified")]
pub async fn validators_unspecified_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_validators_unspecified(PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/validators-of/{address}")]
pub async fn validators_of_delegator_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_validators_by_delegator(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/validator-delegator-pair/{validator_address}/{delegator_address}")]
pub async fn validator_delegator_pair_${this.name}(path: Path<(String, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (validator_address, delegator_address) = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_delegator_validator_pair_info(&delegator_address, &validator_address)
        .await;

    Json(resp)
}

`.slice(1)
    }

    restGestGenerateDelegators () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/delegator-rewards/{address}")]
pub async fn delegator_rewards_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_delegator_rewards(&delegator_address).await;

    Json(resp)
}

#[get("${this.name}/delegator-withdraw-address/{address}")]
pub async fn delegator_withdraw_address_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_delegator_withdraw_address(&delegator_address).await;

    Json(resp)
}

`.slice(1)
    }

    restGenerateTokenomics () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/supply/{denom}")]
pub async fn supply_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let denom = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_supply_by_denom(&denom).await;

    Json(resp)
}

#[get("${this.name}/supplies")]
pub async fn supplies_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_supply_of_all_tokens(PaginationConfig::new().limit(1000).offset(0))
        .await;

    Json(resp)
}

#[get("${this.name}/inflation")]
pub async fn inflation_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_inflation_rate().await;

    Json(resp)
}

`.slice(1)
    }

    restGenerateProposals () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/proposals-passed")]
pub async fn proposals_passed_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposals_passed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposals-voting")]
pub async fn proposals_voting_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposals_in_voting_period(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposals-failed")]
pub async fn proposals_failed_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposals_failed(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposals-rejected")]
pub async fn proposals_rejected_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposals_rejected(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposals-unspecified")]
pub async fn proposals_unspecified_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposals_unspecified(PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposal-deposits/{id}")]
pub async fn proposal_deposits_${this.name}(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposal_deposits(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposal-details/{id}")]
pub async fn proposal_details_${this.name}(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposal_details(id)
        .await;

    Json(resp)
}

#[get("${this.name}/proposal-tally/{id}")]
pub async fn proposal_tally_${this.name}(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposal_tally(id)
        .await;

    Json(resp)
}

#[get("${this.name}/proposal-votes/{id}")]
pub async fn proposal_votes_${this.name}(path: Path<u64>, server_state: Data<ServerState>) -> impl Responder {
    let id = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_proposal_votes(id, PaginationConfig::new().limit(1000).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/proposal-vote/{id}/{voter_address}")]
pub async fn proposal_vote_${this.name}(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, voter_address) = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_proposal_vote_by_voter(id, &voter_address).await;

    Json(resp)
}

#[get("${this.name}/proposal-deposit/{id}/{depositor_address}")]
pub async fn proposal_deposit_${this.name}(path: Path<(u64, String)>, server_state: Data<ServerState>) -> impl Responder {
    let (id, depositor_address) = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_proposal_deposit_by_depositor(id, &depositor_address).await;

    Json(resp)
}

`.slice(1)
    }

    restGenerateDelegations () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/delegations/{delegator_address}")]
pub async fn delegations_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_delegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/unbonding-delegations/{delegator_address}")]
pub async fn unbonding_delegations_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_delegations_unbonding(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

#[get("${this.name}/redelegations/{delegator_address}")]
pub async fn redelegations_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let delegator_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain
        .get_redelegations(&delegator_address, PaginationConfig::new().limit(100).offset(0).reverse())
        .await;

    Json(resp)
}

`

            .slice(1)
    }

    restGenerateStakingPool () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/staking-pool")]
pub async fn staking_pool_${this.name}(server_state: Data<ServerState>) -> impl Responder {
    let chain = &server_state.chains.${this.name};

    let resp = chain.get_staking_pool().await;

    Json(resp)
}

`.slice(1)
    }

    restGenerateSigningInfo () {
        return `
// ==== ${this.name.toUpperCase()} ====

#[get("${this.name}/signing/{address}")]
pub async fn signing_${this.name}(path: Path<String>, server_state: Data<ServerState>) -> impl Responder {
    let cons_address = path.into_inner();

    let chain = &server_state.chains.${this.name};

    let resp = chain.get_signing_info(&cons_address).await;

    Json(resp)
}

`.slice(1)
    }

    wssGenerateParams () {
        return `
        
        `
    }
}

/** Generates REST API code. */
const restApiGenerate = async (chainParamsArray) => {
    const fileContents = {
        macros: '',
        blocks: '',
        transactions: '',
        params: '',
        validators: '',
        delegators: '',
        tokenomics: '',
        proposals: '',
        delegations: '',
        stakingPool: '',
        signingInfo: '',
    };

    let functionNamesArray = [];

    for await (chainParams of chainParamsArray) {
        const chain = new Chain(chainParams)

        fileContents.macros += chain.restGenerateMacro()
        fileContents.blocks += chain.restGenerateBlockPaths()
        fileContents.transactions += chain.restGenerateTransactionPaths()
        fileContents.params += chain.restGenerateParams()
        fileContents.validators += chain.restGenerateValidators()
        fileContents.delegators += chain.restGestGenerateDelegators()
        fileContents.tokenomics += chain.restGenerateTokenomics()
        fileContents.proposals += chain.restGenerateProposals()
        fileContents.delegations += chain.restGenerateDelegations()
        fileContents.stakingPool += chain.restGenerateStakingPool()
        fileContents.signingInfo += chain.restGenerateSigningInfo()

        functionNamesArray = functionNamesArray.concat(chain.getRestFunctionNames().map(name => `rest::${name}`))
    }

    await saveRestFiles(fileContents)

    return functionNamesArray
}


/** The entry of the application. */
const main = async () => {
    const chainParamsArray = await parseChainsParamsFromJson()

    const restApiFunctionNames = await restApiGenerate(chainParamsArray)

    await saveWebServerFile(restApiFunctionNames)
}

main()

