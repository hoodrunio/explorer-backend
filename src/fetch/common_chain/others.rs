use serde::Deserialize;

// Returns the mint parameters of the chain.
/* async fn get_mint_params(&self) -> Option<MintParams> {
    if self.name() == "evmos" {
        self.rest_api_request::<ParamsResp<InflationParams>>("/evmos/inflation/v1/params", &[]).await.and_then(|a|)
    } else if self.name() == "echelon" {
        self.rest_api_request::<ParamsResp<InflationParams>>("/echelon/inflation/v1/params", &[]).await
    } else {
        self.rest_api_request::<ParamsResp<MintParams>>("/cosmos/mint/v1beta1/params", &[])
            .await
            .ok()
            .and_then(|res| Some(res.params))
    }
    .unwrap_or(None)
}
*/

#[derive(Deserialize, Debug)]
pub struct Pagination {
    /// Pagination next key. Might be `None`. Eg: `"FGxWOxzuw4bZozVHta3qYgdKOuRC"`
    pub next_key: Option<String>,
    /// Total. Eg: `"0"`
    pub total: String,
}

#[derive(Deserialize, Debug)]
pub struct DenomAmount {
    /// The name of the token. Eg: `"uatom"`
    pub denom: String,
    /// The amount of the token. Eg: `"450000"`
    pub amount: String,
}

#[derive(Deserialize, Debug)]
pub struct TallyingParamsResp {
    /// Tally parameters.
    pub tally_params: TallyParams,
}

#[derive(Deserialize, Debug)]
pub struct TallyParams {
    /// Quorum. Eg: `"0.400000000000000000"`
    pub quorum: String,
    /// Threshold. Eg: `"0.500000000000000000"`
    pub threshold: String,
    /// Veto threshold. Eg: `"0.334000000000000000"`
    pub veto_threshold: String,
}

#[derive(Deserialize, Debug)]
pub struct DepositParamsResp {
    /// Deposit parameters.
    pub deposit_params: DepositParams,
}

#[derive(Deserialize, Debug)]
pub struct DepositParams {
    /// Array of denoms and amounts.
    pub min_deposit: Vec<DenomAmount>,
    /// Maximum deposit period. Eg: `"0s"`
    pub max_deposit_period: String,
}

#[derive(Deserialize, Debug)]
pub struct VotingParamsResp {
    /// Voting parameters.
    pub voting_params: VotingParams,
}

#[derive(Deserialize, Debug)]
pub struct VotingParams {
    /// Voting period. Eg: `"1209600s"`
    pub voting_period: String,
}

#[derive(Deserialize, Debug)]
pub struct DelegatorRewardsResp {
    /// Array of rewards.
    pub rewards: Vec<DelegatorReward>,
    /// Array of amounts and denoms.
    pub total: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct DelegatorReward {
    /// Validator address. Eg: `"cosmosvaloper1c4k24jzduc365kywrsvf5ujz4ya6mwympnc4en"`
    pub validator_address: String,
    /// Array of amounts and denoms.
    pub reward: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct WithdrawAddressResp {
    /// Delegator withdraw address. Eg: `"cosmos1a3yjj7d3qnx4spgvjcwjq9cw9snrrrhu3rw8nv"`
    pub withdraw_address: String,
}

#[derive(Deserialize, Debug)]
pub struct SigningInfoResp {
    /// Validator signing info.
    pub val_signing_info: SlashingSigningInfoItem,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorResp {
    /// Validator.
    pub validator: ValidatorListValidator,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorsResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct UnbondingDelegationResp {
    /// Array of unbonding delegation responses.
    pub unbonding_responses: Vec<UnbondingDelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct UnbondingDelegationResponse {
    /// Delegator address. Eg: `cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6`
    pub delegator_address: String,
    /// Validator address. Eg: `cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf`
    pub validator_address: String,
    // Array of unbonding delegation entries.
    pub entries: Vec<UnbondingDelegationEntry>,
}

#[derive(Deserialize, Debug)]
pub struct UnbondingDelegationEntry {
    /// Unbonding entry creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Unbonding entry competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Unbonding entry inital balance. Eg: `""`
    pub initial_balance: String,
    /// Unbonding entry balance. Eg: `""`
    pub balance: String,
}

#[derive(Deserialize, Debug)]
pub struct RedelagationsResp {
    /// Array of redelegation responses.
    pub redelegation_responses: Vec<RedelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct RedelegationResponse {
    /// Delegation.
    pub redelegation: Redelegation,
    /// Amount and denom.
    pub entries: Vec<RedelegationEntry>,
}

#[derive(Deserialize, Debug)]
pub struct Redelegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator source address. Eg: `""`
    pub validator_src_address: String,
    /// Validator destination address. Eg: `""`
    pub validator_dst_address: String,
    /// Array of redelegation entries.
    pub entries: Vec<RedelegationEntry>,
}

#[derive(Deserialize, Debug)]
pub struct RedelegationResponseEntry {
    /// Redelegation entry.
    pub redelegation_entry: RedelegationEntry,
    /// Balance. Eg: `""`
    pub balance: String,
}

#[derive(Deserialize, Debug)]
pub struct RedelegationEntry {
    /// Redelagation creation height. Eg: `"524000"`
    pub creation_height: String,
    /// Redelagation competion time. Eg: `"2022-11-06T00:14:50.583Z"`
    pub completion_time: String,
    /// Redelagation inital balance. Eg: `""`
    pub initial_balance: String,
    /// Redelagation shares destination. Eg: `""`
    pub shares_dst: String,
}

#[derive(Deserialize, Debug)]
pub struct DelagationsResp {
    /// Array of delegation responses.
    pub delegation_responses: Vec<DelegationResponse>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct DelegationResponse {
    /// Delegation.
    pub delegation: Delegation,
    /// Amount and denom.
    pub balance: DenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct Delegation {
    /// Delegator address. Eg: `"cosmos156gqf9837u7d4c4678yt3rl4ls9c5vuuxyhkw6"`
    pub delegator_address: String,
    /// Validator address. Eg: `"cosmosvaloper156gqf9837u7d4c4678yt3rl4ls9c5vuursrrzf"`
    pub validator_address: String,
    /// Delegation shares. Eg: `"1899999.000000000000000000"`
    pub shares: String,
}

#[derive(Deserialize, Debug)]
pub struct ProposalVoteByVoterResp {
    /// Proposal vote.
    pub vote: ProposalVote,
}

#[derive(Deserialize, Debug)]
pub struct ProposalVotesResp {
    /// Array of proposal votes.
    pub votes: Vec<ProposalVote>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ProposalVote {
    /// Proposal ID. Eg: `"34"`
    pub proposal_id: String,
    /// Proposal voter. Eg: `""`
    pub voter: String,
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Array of proposal options.
    pub options: Vec<ProposalOption>,
}

#[derive(Deserialize, Debug)]
pub struct ProposalOption {
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Proposal vote option weight. Eg: `""`
    pub weight: String,
}

#[derive(Deserialize, Debug)]
pub struct ProposalTallyResp {
    /// Proposal tally.
    pub tally: ProposalFinalTallyResult,
}

#[derive(Deserialize, Debug)]
pub struct ProposalDepositByDepositorResp {
    /// Proposal deposit.
    pub deposit: ProposalDeposit,
}

#[derive(Deserialize, Debug)]
pub struct ProposalDepositsResp {
    /// Proposal deposits.
    pub deposits: Vec<ProposalDeposit>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ProposalDeposit {
    /// Proposal ID. Eg: `"35"`
    pub proposal_id: String,
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Array of amounts and denoms deposited.
    pub amount: DenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct ProposalsDetailsResp {
    /// Proposal details.
    pub proposal: Proposal,
}

#[derive(Deserialize, Debug)]
pub struct ProposalsResp {
    /// Array of proposals.
    pub proposals: Vec<Proposal>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct Proposal {
    /// Proposal ID. Eg: `"79"`
    pub proposal_id: String,
    /// Proposal content.
    pub content: ProposalContent,
    /// Proposal status. Eg: `"PROPOSAL_STATUS_VOTING_PERIOD"`
    pub status: String,
    /// Proposal final tally result.
    pub final_tally_result: ProposalFinalTallyResult,
    /// Proposal submit time. Eg: `"2022-10-24T19:45:39.969555358Z"`
    pub submit_time: String,
    /// Proposal deposit deadline time. Eg: `"2022-11-07T19:45:39.969555358Z"`
    pub deposit_end_time: String,
    /// Proposal total deposit. Array of amounts and denoms.
    pub total_deposit: Vec<DenomAmount>,
    /// Proposal voting start time. Eg: `"2022-10-24T19:45:39.969555358Z"`
    pub voting_start_time: String,
    /// Proposal voting start time. Eg: `"2022-11-07T19:45:39.969555358Z"`
    pub voting_end_time: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum ProposalContent {
    #[serde(rename = "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal")]
    CommunityPoolSpendProposal {
        /// Community pool spend proposal title. Eg: `"Adan: non-profit fighting for sound crypto regulation"`
        title: String,
        /// Community pool spend proposal description. Eg: `"# Adan: non-profit fighting for sound crypto regulation\n\n## Summary\n\n- Adan is a non-profit organization representing the crypto-asset industry in Europe\n- Since 2020, Adan has worked relentlessly to fight disproportional and overreaching regulations which threaten the crypto industry's ability to innovate\n- We seek 8,000 ATOMS in funding to hire a European Affairs Officer as we ramp up operations in Brussels\n\n## About Adan\n\nAdan is a non-profit trade organization representing the crypto-asset industry. Our members are crypto-native companies and firms that provide industry-specific expertise (legal, consulting, marketing, etc.)\n\nFounded in France in 2020, Adan has over [195 members 6](https://adan.eu/en/association/members) in France and Europe across several industry verticals, including DeFi, payments, market making, custody, data analysis, and security, and is the largest crypto trade organization in Europe.\n\nAdan interacts with all stakeholders with interest in the crypto ecosystem: national and European Members of Parliament, the European Commission, the European Council, the European Parliament, as well as the European Central Bank, and the regulatory bodies and legislators of several European countries.\n\nOur unique positioning allows us to rally decentralized DeFi actors and more traditional listed companies and corporations.\n\n- [Adan Website](https://adan.eu)\n- [Adan on Twitter](https://twitter.com/adan_asso)\n- [Adan on LinkedIn](https://www.linkedin.com/company/adaneu/)\n\n### Mission\n\n- Educate about crypto and help create better narratives around this technology and industry\n- Foster an environment favorable to the growth of the industry\n- Accompany the implementation of French and European regulatory frameworks adapted to the specificities of the sector\n- Build constructive relations between the industry and public institutions\n\n### Team\n\n- Faustine Fleuret: President \u0026 CEO | [Twitter 8](https://twitter.com/faufleuret) / [Linkedin](https://www.linkedin.com/in/faustine-fleuret-640b67a4/)\n- Mélodie Ambroise: Head of Strategy \u0026 Institutional Relations | [Twitter 3](https://twitter.com/mambroise23) / [Linkedin](https://www.linkedin.com/in/m%C3%A9lodie-ambroise/)\n- Jules Dubourg: Administrative \u0026 Financial Manager | [Twitter 1](https://twitter.com/Jules_Dubourg) / [Linkedin 1](https://www.linkedin.com/in/jules-dubourg/)\n- Hugo Bordet: Regulatory Affairs Manager | [Twitter](https://twitter.com/HugoBordet1) / [Linkedin 1](https://www.linkedin.com/in/hugo-bordet-598241152/)\n- Dorian Ravaute: Fiscal Affairs Officer |  [Linkedin 1](https://www.linkedin.com/in/dorianravaute/)\n\n### Funding\n\nAdan is a membership organization and is funded primarily through annual fees.\n\nBeing aware that our missions concern not only the Cosmos community but the whole crypto ecosystem, we will ask for grants from other blockchains. Thus the costs will be shared between different communities.\n\nFinally, we will ask the Cosmos community once only. Our growth plan and the opening of the European market will allow us to be financially self-sufficient through membership fees - which today represent over 80% of our annual income.\n\n### Governance\n\nMembers define the yearly objectives during the annual General Assembly. They also vote on the budget, ratify the association's actions, and elect a Board of Directors responsible for representing them and controlling the implementation of these missions.\n\nSee our website for more information about Adan's governance and [Board of Directors 3](https://adan.eu/en/association/governance).\n\nAdan is a non-profit organization registered in France (Association loi de 1901).\n\n### Works \u0026 Publications\n\nAdan interacts with all stakeholders with an interest in the crypto ecosystem: national and European Members of Parliament, the European Commission, the European Council, the European Parliament, as well as the European Central Bank, and the regulatory bodies and legislators of several European countries.\n\nAdan is also involved in discussions with international bodies such as FATF, IOSO, BIS etc.\n\nIn just two short years, Adan has produced an astounding amount of writing and [publications](https://adan.eu/en/publications) to support its mission:\n\n- [A crypto-euro issued by an American giant, or how Europe is delegating its monetary sovereignty](https://adan.eu/en/tribune-les-echos-crypto-euro-en)\n- [EU framework for crypto-asset markets: the French Presidency ends with political deals on MiCA and TFR](https://adan.eu/en/press-release-policy-agreements-mica)\n- [Adan's Response to IOSCO's Retail Market Conduct Report](https://adan.eu/en/consultation/en/response-report-iosco-retail-market)\n- [Adoption of TFR in the European Parliament: the fight against financial crime should not be a fight against crypto-assets](https://adan.eu/en/press/tfr-travel-rule-vote-european-parliament-europeen-econ-libe)\n- [MiCA vote in the European Parliament: A step forward or backward for the crypto sector?](https://adan.eu/en/press-release/european-parliament-mica-compromise-crypto)\n- [Adan responds to the EBA consultation on its draft guidelines for remote onboarding customer solutions 1](https://adan.eu/en/consultation/response-guidelines-eba-onboarding-solutions)\n- [Ban of Proof-of-Work protocols: wrong answer to real global environmental challenges](https://adan.eu/en/position/pow-bitcoin-ban-eu-inappropriate-answer)\n- [Adan's position on FATF's updated guidance for a risk-based approach 1](https://adan.eu/en/position/fatf-updated-guidance-vasp)\n\n## Proposal details\n\nThe crypto industry is evolving rapidly, challenges are multiplying, and public authorities worldwide are increasingly seeking to regulate crypto innovation.\n\nTo this end, we will continue to fight the battles initiated at the French level but must accelerate on the European and international scene according to 4 priorities:\n\n- Monitor upcoming legislation and regulations, such as MiCA, TFR, and the remainder of the AML package.\n- Contribute to elaborating regulatory frameworks in preparation surrounding topics like DeFi, NFTs, the environmental impact of crypto, etc.\n- Establish strong positions on European crypto companies' issues (e.g., access to banking and insurance) and convey them to policymakers.\n- Sensibly educate on regulatory proposals which fail to capture the unique properties of crypto-economic models and risk hindering innovation (e.g., regulating DAOs as legal persons in the traditional sense).\n\nTo overcome these challenges, our team must engage in the following activities:\n\n- Carry out diligent monitoring of the legislative and regulatory agenda\n- Think up pragmatic solutions adapted to the sector within our working groups\n- Dialogue with decision-makers in European institutions (European Commission, Council of the European Union, European Parliament), European authorities (European Central Bank, European Banking Authority, etc.), and international bodies (FATF, IOSCO, BIS, etc.)\n- Create synergies with other market players with shared interests\n\nGiven the size and importance of the mission, which is constantly expanding, Adan must strengthen its team. Thus we request funding from the Cosmos Hub community pool to recruit a European Affairs Officer based in Brussels, allowing us to further increase our ties with transnational bodies.\n\n## Deliverables\n\nWe believe transparency around community funding is important for building trust and establishing a reputation. This is why we pledge to publish progress reports in 6 and 12 months such that the Cosmos Community better understands how funds are spent and may evaluate the return on its investment.\n\nThis report will be delivered to you via this forum. It will consist of different sections, such as :\n\n- actions carried out in the last months;\n- missions to be carried out in the coming months;\n- distribution of the remaining allocation.\n\n## Funding Amount\n\nTotal amount requested: 8,000 ATOM\n\nThis corresponds to roughly 100,000 EUR and covers gross salary, recruitment costs, and travel expenses for half a year.\n\n## Receipient\n\ncosmos1kdff80vxuj0zmmjauymzjs40hsfkuya79s8tm0\n\n## IPFS\n\nQmR1q2i48EJqaZSXxgggE8VaPKsZFtozVBPHKejMpJAYXx\n\n## Governance Discussion\n\nhttps://forum.cosmos.network/t/proposal-draft-adan-non-profit-fighting-for-sound-crypto-regulation/7416\n\n## Governance votes\n\nThe following items summarize the voting options and what it means for this proposal:\n\n- YES - You agree that persuading regulators to adopt sound and proportional regulation is important and thus wish to fund Adan's work\n- NO - You don't agree that persuading regulators is important and thus do not agree to fund Adan's work\n- NO WITH VETO - A 'NoWithVeto' vote indicates a proposal either (1) is deemed to be spam, i.e., irrelevant to Cosmos Hub, (2) disproportionately infringes on minority interests, or (3) violates or encourages violation of the rules of engagement as currently set out by Cosmos Hub governance. If the number of 'NoWithVeto' votes is greater than a third of the total votes, the proposal is rejected, and the deposits are burned.\n- ABSTAIN - You wish to contribute to the quorum but formally decline to vote either for or against the proposal."`
        description: String,
        /// Community pool spend proposal recipient address. Eg: `"cosmos1kdff80vxuj0zmmjauymzjs40hsfkuya79s8tm0"`
        recipient: String,
        /// Community pool spend proposal amount. Array of amounts and denoms.
        amount: Vec<DenomAmount>,
    },
    #[serde(rename = "/cosmos.gov.v1beta1.TextProposal")]
    TextProposal {
        /// Text proposal title. Eg: `"Risk and financial analysis of ATOM2.0"`
        title: String,
        /// Text proposal description. `"In depth financial analysis of ATOM2.0:\nhttps://pastebin.com/fVQ81d7H\n\nIn depth risk analysis of ATOM2.0:\nhttps://cryptpad.fr/pad/#/2/pad/view/v3QYkKeqenjgK+yPi8bDmuYv4cOBalDaei4sLta6RTg/\nhttps://pastebin.com/bgEqdKct      - backup link\n\n\nWhile many only make claims of faith, these papers make claim of empirical liablity and risk.\n\nGroups to discuss the various proposals:\nhttps://t.me/AtomPrice\nhttps://t.me/atomgov \nhttps://t.me/+uNNyjiYO38lhZDYx\n\nOpen source community lab with the goal of finding an alternative to dilution:\nhttps://forum.cosmos.network/t/atom-zero-a-open-source-non-dilutive-communitylab-for-atom2-0/7860"`
        description: String,
    },
    #[serde(rename = "/cosmos.params.v1beta1.ParameterChangeProposal")]
    ParameterChangeProposal {
        /// Parameter change proposal title. Eg: `"Adjust Blocks Per Year to 4.36M"`
        title: String,
        /// Parameter change proposal description. Eg: `"While the current inflation rate is set at 7%, the effective inflation rate is more like ~6.29%. This is because blocks have slowed down somewhat from ~6.5s to ~7.24s per block, and thus the current blocks per year value of 4855015 is too high. Here we propose to adjust the blocks per year value from 4855015 to 4360000 to bring it in line with current block times, which should realign the effective inflation rate. More details were drafted on Github (https://github.com/cosmos/governance/tree/master/proposals/2020-10-blocks-per-year) and are available on IPFS (https://ipfs.io/ipfs/QmTkzDwWqPbnAh5YiV5VwcTLnGdwSNsNTn2aDxdXBFca7D/example#/ipfs/QmTZ3R4W2odBsx6hpt7awfRTfZA67x5eQaoL6ctxBr6NyN)"`
        description: String,
        /// Array of changes wanted.
        changes: Vec<ParameterChangeProposalChange>,
    },
    #[serde(rename = "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal")]
    SoftwareUpgradeProposal {
        /// Software upgrade proposal title. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
        title: String,
        /// Software upgrade propsal description. Eg: `"Summary: Tendermint (https://tendermint.com) and B-Harvest (https://bharvest.io) have joined forces to produce and develop a Liquidity Module (https://github.com/tendermint/liquidity). This signal proposal is a Request For Comment to the ATOM community regarding the addition of this Liquidity Module into the Cosmos Hub source code.\nBy voting YES to this proposal, you will signal that you approve of having a DEX based on this specific Liquidity Module deployed on the Cosmos Hub.\nDetail of the proposal can be found at IPFS link below.\n\nCurrent Development : https://github.com/tendermint/liquidity/tree/develop\nIPFS : https://ipfs.io/ipfs/QmZpgkYLoCBnXM1S7TEdQunMmur9bAw5VTNgFQCyrqgKDd"`
        description: String,
        /// Software upgrade proposal plan.
        plan: SoftwareUpgradeProposalPlan,
    },
    #[serde(rename = "/ibc.core.client.v1.ClientUpdateProposal")]
    ClientUpdateProposal {
        /// Client update proposal title. Eg: `"Update expired client between Cosmoshub and Bostrom"`
        title: String,
        /// Client update proposal description. Eg: `"This proposal will update the expired client on channel-240 between cosmoshub-4 and the bostrom networks. In turn, this will let users transfer  from bostrom, and to transfer  from cosmoshub back to bostrom.\\n\\nBy voting **YES**, the Cosmoshub stakers, voice their support to unfreeze IBC channel-240 between Cosmoshub and Bostrom.\\n\\nBy voting **NO**, the Cosmoshub stakers voice their dissent to unfreeze IBC channel-240 between Cosmoshub and Bostrom network.\\n\\n**Details:**\\n\\nMost IBC connections between Bostrom and other Cosmos chains have been relayed, to a large extent, only by the Bro_n_Bro validator.\\n\\nOriginally, channel-240 was created with a very short trusting period of 2 days. Alas, the lack of monitoring from our side caused the expiration of client 07-tendermint-497, which in turn, led to the impossibility to transfer tokens using channel-240. Currently, there are around 710 ATOM stuck on the bostrom chain, belonging to about 20 different accounts.\\n\\nAs this might be the first case, when a channel renewal on cosmoshub-4, happens via a governance proposal, we have set up prior testing to ensure that everything will work smoothly. We also modified test-suite https://github.com/bro-n-bro/ibc-testbed (thanks to the Lum devs for the awesome repo), so everyone could simulate the client renewal using governance with this test suite.\\n\\nIn the case that this proposal goes through, client 07-tendermint-497 state will be substituted by the state of client 07-tendermint-643.\\nAlso if passed - channels 240-5 (cosmoshub-4 - bostrom) would be used, only, to recover the stuck funds. New channels would be created with a longer trusting period to ensure further stability.\\n\\nWe will be happy to answer any questions at our [Telegram community group](https://t.me/bro_n_bro_community) or on our [Discord](https://discord.com/channels/868962876721860638/870738846772514826)."`
        description: String,
        /// Client update proposal subject client ID. Eg: `"07-tendermint-497"`
        subject_client_id: String,
        /// Client update proposal substitue client ID. Eg: `"07-tendermint-643"`
        substitute_client_id: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct SoftwareUpgradeProposalPlan {
    /// Software upgrade proposal plan name. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
    pub name: String,
    /// Software upgrade proposal plan time. Eg: `"9999-12-31T00:00:00Z"`
    pub time: String,
    /// Software upgrade proposal plan height. Eg: `"0"`
    pub height: String,
    /// Software upgrade proposal plan information. Eg: `"This is information about software upgrade."`
    pub info: String,
    // Software upgrade proposal plan upgraded client state. Can be `None`.
    // TODO! We don't know what it can be.
    // upgraded_client_state: Option<>
}

#[derive(Deserialize, Debug)]
pub struct ParameterChangeProposalChange {
    /// Subspace. Eg: `"mint"`
    pub subspace: String,
    /// Key. Eg: `"BlocksPerYear"`
    pub key: String,
    /// Value. Inside quotes. Eg: `"\"4360000\""`
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct ProposalFinalTallyResult {
    /// Number of `yes` votes. Eg: `"50"`
    pub yes: String,
    /// Number of `abstain` votes. Eg: `"35"`
    pub abstain: String,
    /// Number of `no` votes. Eg: `"12"`
    pub no: String,
    /// Number of `no with veto` votes.  Eg: `"7"`
    pub no_with_veto: String,
}

#[derive(Deserialize, Debug)]
pub struct InflationParams {
    /// Mint denom. Eg: `"aevmos"`
    pub mint_denom: String,
    /// Exponential calculation.
    pub exponential_calculation: InflationParamsExponentialCalculation,
    /// Inflation distribution.
    pub inflation_distribution: InflationParamsInflationDistribution,
    /// Enable inflation. Eg: `true`
    pub enable_inflation: bool,
}

#[derive(Deserialize, Debug)]
pub struct InflationParamsInflationDistribution {
    /// Staking rewards inflation. Eg: `"0.533333334000000000"`
    pub staking_rewards: String,
    /// Usage incentives inflation. Eg: `"0.333333333000000000"`
    pub usage_incentives: String,
    /// Community pool inflation. Eg: `"0.133333333000000000"`
    pub community_pool: String,
}

#[derive(Deserialize, Debug)]
pub struct InflationParamsExponentialCalculation {
    /// Unknown. Eg: `"300000000.000000000000000000"`
    pub a: String,
    /// Unknown. Eg: `"0.500000000000000000"`
    pub r: String,
    /// Unknown. Eg: `"9375000.000000000000000000"`
    pub c: String,
    /// Bonding target. Eg: `"0.660000000000000000"`
    pub bonding_target: String,
    /// Maximum variance. Eg: `"0.000000000000000000"`
    pub max_variance: String,
}

#[derive(Deserialize, Debug)]
pub struct MintParams {
    /// Mint denom. Eg: `"uatom"`
    pub mint_denom: String,
    /// Mint inflation rate change. Eg: ` "1.000000000000000000"`
    pub inflation_rate_change: String,
    /// Maximum mint inflation. Eg: `"0.200000000000000000"`
    pub inflation_max: String,
    /// Minimum mint inflation. Eg: `"0.070000000000000000"`
    pub inflation_min: String,
    /// Goal bonded. Eg: `"0.670000000000000000"`
    pub goal_bonded: String,
    /// Blocks per year. Eg: `"4360000"`
    pub blocks_per_year: String,
}

#[derive(Deserialize, Debug)]
pub struct SlashingParams {
    /// Slashing, signed blocks window. Eg: `"10000"`
    pub signed_blocks_window: String,
    /// Slashing, minimum signed per window. Eg: `"0.050000000000000000"`
    pub min_signed_per_window: String,
    /// Slashing, downtime jail duration. Eg: `"600s"`
    pub downtime_jail_duration: String,
    /// Slash fraction double sign. Eg: `"0.050000000000000000"`
    pub slash_fraction_double_sign: String,
    /// Slash fraction downtime. Eg: `"0.000100000000000000"`
    pub slash_fraction_downtime: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListResp {
    /// Array of validators.
    pub validators: Vec<ValidatorListValidator>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidator {
    /// Operator address. Eg: `"evmosvaloper1qq95x6dhrdnrfunlth5uh24tkrfphzl9crd3xr"`
    pub operator_address: String,
    /// Consensus public key.
    pub consensus_pubkey: PublicKey,
    /// Jailed state. Eg: `false`
    pub jailed: bool,
    /// Status. Eg: `"BOND_STATUS_BONDED"`
    pub status: String,
    /// Tokens. Eg: `"145722654634775400576772"`
    pub tokens: String,
    /// Delegator shares. Eg: `"146454922655204548581706.446790192014497216"`
    pub delegator_shares: String,
    /// Description.
    pub description: ValidatorListValidatorDescription,
    /// Unbonding height. Eg: `"2580496"`
    pub unbonding_height: String,
    /// Unbonding time. Eg: `"2022-08-21T03:48:38.952541966Z"`
    pub unbonding_time: String,
    /// Validator commission.
    pub commission: ValidatorListValidatorCommission,
    /// Minimum self delegation. Eg: `"1"`
    pub min_self_delegation: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorCommission {
    /// Validator commission rates.
    pub commission_rates: ValidatorListValidatorCommissionRates,
    /// Validator commission update time. Eg: `"2022-03-02T19:00:00Z"`
    pub update_time: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorCommissionRates {
    /// Validator commission rate. Eg: `"0.050000000000000000"`
    pub rate: String,
    /// Validator maximum commission rate. Eg: `"0.200000000000000000"`
    pub max_rate: String,
    /// Validator maximum commission change rate. Eg: `"0.010000000000000000"`
    pub max_change_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorListValidatorDescription {
    /// Validator moniker. Eg: `"heisenbug"`
    pub moniker: String,
    /// Validator identity. Eg: `"367960C067E253A4"`
    pub identity: String,
    /// Validator website. Eg: `"https://heisenbug.one"`
    pub website: String,
    /// Validator security contact. Eg: `"@heisenbug_evmos"`
    pub security_contact: String,
    /// Validator details. Eg: `"reliable \u0026\u0026 secure staking"`
    pub details: String,
}

#[derive(Deserialize, Debug)]
pub struct ParamsResp<T> {
    /// The parameters.
    pub params: T,
}

#[derive(Deserialize, Debug)]
pub struct StakingParams {
    /// Unbonding time. Eg: `"1814400s"`
    pub unbonding_time: String,
    /// Maximum number of validators. Eg: `175`
    pub max_validators: usize,
    /// Maximum number of entries. Eg: `7`
    pub max_entries: usize,
    /// Historical number of entries. Eg: `10000`
    pub historical_entries: usize,
    /// Bonding denom. Eg: `"uatom"`
    pub bond_denom: String,
}

#[derive(Deserialize, Debug)]
pub struct MintingInflationResp {
    /// Minting inflation rate. Eg: `"0.131020685388983473"`
    pub inflation: String,
}

#[derive(Deserialize, Debug)]
pub struct MintingInflationRateResp {
    /// Minting inflation rate. Eg: `"91.087708112747866100"`
    pub inflation_rate: String,
}

#[derive(Deserialize, Debug)]
pub struct StakingPoolResp {
    /// Staking pool information.
    pub pool: StakingPool,
}

#[derive(Deserialize, Debug)]
pub struct StakingPool {
    /// Tokens not bonded. Eg: `"15241580330282"`
    pub not_bonded_tokens: String,
    /// Tokens bonded. Eg: `"203496656637783"`
    pub bonded_tokens: String,
}

#[derive(Deserialize, Debug)]
pub struct SupplyByDenomResp {
    /// Amount and denom.
    pub amount: DenomAmount,
}

#[derive(Deserialize, Debug)]
pub struct SupplyOfAllTokensResp {
    /// Array of amounts and denoms.
    pub supply: Vec<DenomAmount>,
    /// Paginations
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorCommisionResp {
    /// Validator commission.
    pub commission: ValidatorCommision,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorCommision {
    /// Array of amounts and demons.
    pub commission: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorRewardsResp {
    /// Validator rewards.
    pub rewards: ValidatorCommision,
}

#[derive(Deserialize, Debug)]
pub struct ValidatorRewards {
    /// Array of amounts and denoms.
    pub rewards: Vec<DenomAmount>,
}

#[derive(Deserialize, Debug)]
pub struct SlashingSigningInfo {
    pub info: Vec<SlashingSigningInfoItem>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Debug)]
pub struct SlashingSigningInfoItem {
    /// Validator address. Eg: `"evmosvalcons1qx4hehfny66jfzymzn6d5t38m0ely3cvw6zn06"`
    pub address: String,
    /// The block height slashing is started at. Eg: `"0"`
    pub start_height: String,
    /// Unknown. Eg: `"5888077"`
    pub index_offset: String,
    /// The time jailed until. Eg: `"2022-05-14T04:31:49.705643236Z"`
    pub jailed_until: String,
    /// Tombstoned state. Eg: `false`
    pub tombstoned: bool,
    /// The count of missed blocks. Eg: `"16433"`
    pub missed_blocks_counter: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum PublicKey {
    #[serde(rename = "/cosmos.crypto.secp256k1.PubKey")]
    Secp256K1 {
        /// Base 64 encoded Secp256K1 public key. Eg: `"Ap9xAyS21AGuRY4W7+Mi3JzbmULJjGATAzVeIxc98t07"`
        key: String,
    },
    #[serde(rename = "/cosmos.crypto.ed25519.PubKey")]
    Ed25519 {
        /// Base 64 encoded Ed25519 public key. Eg: `"zy/GxGwk1Pm3HiG67iani1u+MUieM98ZvSIrXC8mISE="`
        key: String,
    },
    #[serde(rename = "/ethermint.crypto.v1.ethsecp256k1.PubKey")]
    Ethsecp256k1 {
        /// Base 64 encoded Ethsecp256k1 public key. Eg: `"AqrviRnJYWdC2OMM1haDI2X6oEIev8u0oqR10Elb06+1"`
        key: String,
    },
}

/// The configuration to be used while making REST API requests.
pub struct PaginationConfig {
    /// It is set to true if results are to be returned in the descending order.
    reverse: bool,
    /// It is the number of result to not to be returned in the result page
    offset: u64,
    /// It is the total number of results to be returned in the result page
    limit: u64,
}

impl PaginationConfig {
    /// Creates a new `PaginationConfig`.
    /// ## Default:
    /// ```rs
    /// PaginationConfig {
    ///     reverse: false,
    ///     offset: 0,
    ///     limit: 10,
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            reverse: false,
            offset: 0,
            limit: 10,
        }
    }

    /// Returns `true` if `reverse` property is set to `true`.
    pub fn is_reverse(&self) -> bool {
        self.reverse
    }

    /// Returns the value `limit` property holds.
    pub fn get_limit(&self) -> u64 {
        self.limit
    }

    /// Returns the value `offset` property holds.
    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    /// Makes the response reversed.
    pub fn reverse(self) -> Self {
        Self { reverse: true, ..self }
    }

    /// Sets a limit for results to be returned in the result page
    pub fn limit(self, limit: u64) -> Self {
        Self { limit, ..self }
    }

    /// Sets an offset for padding from the first result.
    pub fn offset(self, offset: u64) -> Self {
        Self { offset, ..self }
    }
}
