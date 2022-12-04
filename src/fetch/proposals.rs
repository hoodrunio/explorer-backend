use chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::others::{DenomAmount, Pagination, PaginationConfig};
use crate::{
    chain::Chain,
    routes::{calc_pages, OutRestResponse},
};

impl Chain {
    /// Returns all the proposals in voting period.
    pub async fn get_proposals_by_status(&self, status: &str, config: PaginationConfig) -> Result<OutRestResponse<Vec<ProposalItem>>, String> {
        let mut query = vec![];

        query.push(("proposal_status", status.to_string()));
        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ProposalsResp>("/cosmos/gov/v1beta1/proposals", &query).await?;

        let mut proposals = vec![];

        for proposal in resp.proposals {
            proposals.push(proposal.try_into()?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(proposals, pages))
    }

    /// Returns all the proposals unspecified.
    pub async fn get_proposals_unspecified(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ProposalItem>>, String> {
        self.get_proposals_by_status("1", config).await
    }

    /// Returns all the proposals in voting period.
    pub async fn get_proposals_in_voting_period(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ProposalItem>>, String> {
        self.get_proposals_by_status("2", config).await
    }

    /// Returns all the proposals passed.
    pub async fn get_proposals_passed(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ProposalItem>>, String> {
        self.get_proposals_by_status("3", config).await
    }

    /// Returns all the proposals rejected.
    pub async fn get_proposals_rejected(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ProposalItem>>, String> {
        self.get_proposals_by_status("4", config).await
    }

    /// Returns all the proposals failed.
    pub async fn get_proposals_failed(&self, config: PaginationConfig) -> Result<OutRestResponse<Vec<ProposalItem>>, String> {
        self.get_proposals_by_status("5", config).await
    }

    /// Returns the details of given proposal.
    pub async fn get_proposal_details(&self, proposal_id: u64) -> Result<OutRestResponse<InternalProposal>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}");

        let resp = self.rest_api_request::<ProposalsDetailsResp>(&path, &[]).await?;

        let proposal = InternalProposal {
            proposal_id: resp
                .proposal
                .proposal_id
                .parse()
                .map_err(|_| format!("Proposal ID cannot be parsed, '{}'.", resp.proposal.proposal_id))?,
            content: InternalProposalContent::try_from_with(resp.proposal.content, |a| self.calc_amount_u128_to_f64(a))?,
            status: match resp.proposal.status.as_ref() {
                "PROPOSAL_STATUS_PASSED" => "Passed",
                "PROPOSAL_STATUS_REJECTED" => "Rejected",
                "PROPOSAL_STATUS_FAILED" => "Failed",
                "PROPOSAL_STATUS_VOTING_PERIOD" => "Voting",
                _ => "Unknown",
            }
            .to_string(),
            final_tally_result: InternalProposalFinalTallyResult::try_from_with(resp.proposal.final_tally_result, |a| {
                self.calc_amount_u128_to_f64(a)
            })?,
            submit_time: DateTime::parse_from_rfc3339(&resp.proposal.submit_time)
                .map_err(|_| format!("Cannot parse proposal submit time, '{}'", resp.proposal.submit_time))?
                .timestamp_millis(),
            deposit_end_time: DateTime::parse_from_rfc3339(&resp.proposal.deposit_end_time)
                .map_err(|_| format!("Cannot parse proposal deposit end time, '{}'", resp.proposal.deposit_end_time))?
                .timestamp_millis(),
            total_deposit: resp
                .proposal
                .total_deposit
                .get(0)
                .and_then(|td| td.amount.parse::<f64>().map(|amount| self.calc_amount_f64_to_f64(amount)).ok())
                .unwrap_or(0.0),
            voting_start_time: DateTime::parse_from_rfc3339(&resp.proposal.voting_start_time)
                .map_err(|_| format!("Cannot parse proposal voting start time, '{}'", resp.proposal.voting_start_time))?
                .timestamp_millis(),
            voting_end_time: DateTime::parse_from_rfc3339(&resp.proposal.voting_end_time)
                .map_err(|_| format!("Cannot parse proposal voting end time, '{}'", resp.proposal.voting_end_time))?
                .timestamp_millis(),
        };

        // We specify page as 0. Because that means there is no need for pagination.
        Ok(OutRestResponse::new(proposal, 0))
    }

    /// Returns the deposits of given proposal.
    pub async fn get_proposal_deposits(
        &self,
        proposal_id: u64,
        config: PaginationConfig,
    ) -> Result<OutRestResponse<Vec<InternalProposalDeposit>>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ProposalDepositsResp>(&path, &query).await?;

        let mut proposal_deposits = vec![];

        for proposal_deposit in resp.deposits {
            proposal_deposits.push(InternalProposalDeposit::new(proposal_deposit, self)?);
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(proposal_deposits, pages))
    }

    /// Returns the deposit of given proposal by given depositor.
    pub async fn get_proposal_deposit_by_depositor(
        &self,
        proposal_id: u64,
        depositor: &str,
    ) -> Result<OutRestResponse<InternalProposalDeposit>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits/{depositor}");

        let resp = self.rest_api_request::<ProposalDepositByDepositorResp>(&path, &[]).await?;

        let deposit = InternalProposalDeposit::new(resp.deposit, self)?;

        Ok(OutRestResponse::new(deposit, 0))
    }

    /// Returns the tally of given proposal.
    pub async fn get_proposal_tally(&self, proposal_id: u64) -> Result<OutRestResponse<InternalProposalFinalTallyResult>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/tally");

        let resp = self.rest_api_request::<ProposalTallyResp>(&path, &[]).await?;

        Ok(OutRestResponse::new(
            InternalProposalFinalTallyResult::try_from_with(resp.tally, |a| self.calc_amount_u128_to_f64(a))?,
            0,
        ))
    }

    /// Returns the votes of given proposal.
    pub async fn get_proposal_votes(&self, proposal_id: u64, config: PaginationConfig) -> Result<OutRestResponse<Vec<InternalProposalVote>>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes");

        let mut query = vec![];

        query.push(("pagination.reverse", format!("{}", config.is_reverse())));
        query.push(("pagination.limit", format!("{}", config.get_limit())));
        query.push(("pagination.count_total", "true".to_string()));
        query.push(("pagination.offset", format!("{}", config.get_offset())));

        let resp = self.rest_api_request::<ProposalVotesResp>(&path, &query).await?;

        let mut proposal_votes = vec![];

        for proposal_vote in resp.votes {
            match proposal_vote.try_into() {
                Ok(proposal_vote) => proposal_votes.push(proposal_vote),
                Err(error) => eprintln!("{}", error),
            }
        }

        let pages = calc_pages(resp.pagination, config)?;

        Ok(OutRestResponse::new(proposal_votes, pages))
    }

    /// Returns the vote of given proposal by given voter.
    pub async fn get_proposal_vote_by_voter(&self, proposal_id: u64, voter: &str) -> Result<OutRestResponse<InternalProposalVote>, String> {
        let path = format!("/cosmos/gov/v1beta1/proposals/{proposal_id}/votes/{voter}");

        let resp = self.rest_api_request::<ProposalVoteByVoterResp>(&path, &[]).await?;

        let proposal_vote = resp.vote.try_into()?;

        Ok(OutRestResponse::new(proposal_vote, 0))
    }
}

impl TryFrom<BasicProposal> for ProposalItem {
    type Error = String;
    fn try_from(proposal: BasicProposal) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id: proposal
                .proposal_id
                .parse()
                .map_err(|_| format!("Proposal ID cannot be parsed, '{}'.", proposal.proposal_id))?,
            title: proposal.content.title,
            description: proposal.content.description,
            time: DateTime::parse_from_rfc3339(&proposal.voting_start_time)
                .map_err(|_| format!("Cannot parse proposal voting time datetime, '{}'.", proposal.voting_start_time))?
                .timestamp_millis(),
            status: match proposal.status.as_ref() {
                "PROPOSAL_STATUS_PASSED" => "Passed",
                "PROPOSAL_STATUS_REJECTED" => "Rejected",
                "PROPOSAL_STATUS_FAILED" => "Failed",
                "PROPOSAL_STATUS_VOTING_PERIOD" => "Voting",
                _ => "Unknown",
            }
            .into(),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalVoteByVoterResp {
    /// Proposal vote.
    pub vote: ProposalVote,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalVotesResp {
    /// Array of proposal votes.
    pub votes: Vec<ProposalVote>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalVote {
    /// Proposal ID. Eg: `34`
    pub proposal_id: u32,
    /// Proposal voter. Eg: `""`
    pub voter: String,
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Array of proposal options.
    pub options: Vec<ProposalOption>,
}

impl TryFrom<ProposalVote> for InternalProposalVote {
    type Error = String;
    fn try_from(value: ProposalVote) -> Result<Self, Self::Error> {
        Ok(Self {
            proposal_id: value
                .proposal_id
                .parse()
                .map_err(|_| format!("Cannot parse proposal id, '{}'.", value.proposal_id))?,
            voter: value.voter,
            option: value.option,
            options: value.options,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalOption {
    /// Proposal vote option. Eg: `"VOTE_OPTION_UNSPECIFIED"`
    pub option: String,
    /// Proposal vote option weight. Eg: `""`
    pub weight: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalTallyResp {
    /// Proposal tally.
    pub tally: ProposalFinalTallyResult,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalDepositByDepositorResp {
    /// Proposal deposit.
    pub deposit: ProposalDeposit,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalDepositsResp {
    /// Proposal deposits.
    pub deposits: Vec<ProposalDeposit>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalDeposit {
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Amounts and denom deposited.
    pub amount: Vec<DenomAmount>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalDeposit {
    /// Proposal depositor. Eg: `""`
    pub depositor: String,
    /// Amount deposited.
    pub amount: f64,
}

impl InternalProposalDeposit {
    fn new(value: ProposalDeposit, chain: &Chain) -> Result<Self, String> {
        Ok(Self {
            depositor: value.depositor,
            amount: match value.amount.get(0) {
                Some(da) => chain.calc_amount_u128_to_f64(
                    da.amount
                        .parse::<u128>()
                        .map_err(|_| "Cannot parse proposal deposit amount.".to_string())?,
                ),
                None => return Err("There is no proposal deposit.".to_string()),
            },
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalsDetailsResp {
    /// Proposal details.
    pub proposal: Proposal,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProposalsResp {
    /// Array of proposals.
    pub proposals: Vec<BasicProposal>,
    /// Pagination.
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BasicProposal {
    /// Proposal ID. Eg: `"79"`
    pub proposal_id: String,
    /// Proposal content.
    pub content: BasicPropsalContent,
    /// Proposal status. Eg: `"PROPOSAL_STATUS_VOTING_PERIOD"`
    pub status: String,
    /// Proposal voting start time. Eg: `"2022-11-15T22:09:29.130698116Z"`
    pub voting_start_time: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BasicPropsalContent {
    /// Client update proposal title. Eg: `"Update expired client between Cosmoshub and Bostrom"`
    title: String,
    /// Client update proposal description. Eg: `"This proposal will update the expired client on channel-240 between cosmoshub-4 and the bostrom networks. In turn, this will let users transfer  from bostrom, and to transfer  from cosmoshub back to bostrom.\\n\\nBy voting **YES**, the Cosmoshub stakers, voice their support to unfreeze IBC channel-240 between Cosmoshub and Bostrom.\\n\\nBy voting **NO**, the Cosmoshub stakers voice their dissent to unfreeze IBC channel-240 between Cosmoshub and Bostrom network.\\n\\n**Details:**\\n\\nMost IBC connections between Bostrom and other Cosmos chains have been relayed, to a large extent, only by the Bro_n_Bro validator.\\n\\nOriginally, channel-240 was created with a very short trusting period of 2 days. Alas, the lack of monitoring from our side caused the expiration of client 07-tendermint-497, which in turn, led to the impossibility to transfer tokens using channel-240. Currently, there are around 710 ATOM stuck on the bostrom chain, belonging to about 20 different accounts.\\n\\nAs this might be the first case, when a channel renewal on cosmoshub-4, happens via a governance proposal, we have set up prior testing to ensure that everything will work smoothly. We also modified test-suite https://github.com/bro-n-bro/ibc-testbed (thanks to the Lum devs for the awesome repo), so everyone could simulate the client renewal using governance with this test suite.\\n\\nIn the case that this proposal goes through, client 07-tendermint-497 state will be substituted by the state of client 07-tendermint-643.\\nAlso if passed - channels 240-5 (cosmoshub-4 - bostrom) would be used, only, to recover the stuck funds. New channels would be created with a longer trusting period to ensure further stability.\\n\\nWe will be happy to answer any questions at our [Telegram community group](https://t.me/bro_n_bro_community) or on our [Discord](https://discord.com/channels/868962876721860638/870738846772514826)."`
    description: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Proposal {
    /// Proposal ID. Eg: `"79"`
    pub proposal_id: String,
    /// Proposal content.
    pub content: ProposalContentWithUnknown,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposal {
    /// Proposal ID. Eg: `79`
    pub proposal_id: u32,
    /// Proposal content.
    pub content: InternalProposalContent,
    /// Proposal status. Eg: `"Passed"`
    pub status: String,
    /// Proposal final tally result.
    pub final_tally_result: InternalProposalFinalTallyResult,
    /// Proposal submit timestamp in milliseconds.
    pub submit_time: i64,
    /// Proposal deposit deadline timestamp in milliseconds.
    pub deposit_end_time: i64,
    /// Proposal total deposit in the native coin of the chain..
    pub total_deposit: f64,
    /// Proposal voting start timestamp in milliseconds.
    pub voting_start_time: i64,
    /// Proposal voting start timestamp in milliseconds.
    pub voting_end_time: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "@type")]
pub enum ProposalContent {
    #[serde(rename = "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal")]
    CommunityPoolSpend {
        /// Community pool spend proposal title. Eg: `"Adan: non-profit fighting for sound crypto regulation"`
        title: String,
        /// Community pool spend proposal description. Eg: `"# Adan: non-profit fighting for sound crypto regulation\n\n## Summary\n\n- Adan is a non-profit organization representing the crypto-asset industry in Europe\n- Since 2020, Adan has worked relentlessly to fight disproportional and overreaching regulations which threaten the crypto industry's ability to innovate\n- We seek 8,000 ATOMS in funding to hire a European Affairs Officer as we ramp up operations in Brussels\n\n## About Adan\n\nAdan is a non-profit trade organization representing the crypto-asset industry. Our members are crypto-native companies and firms that provide industry-specific expertise (legal, consulting, marketing, etc.)\n\nFounded in France in 2020, Adan has over [195 members 6](https://adan.eu/en/association/members) in France and Europe across several industry verticals, including DeFi, payments, market making, custody, data analysis, and security, and is the largest crypto trade organization in Europe.\n\nAdan interacts with all stakeholders with interest in the crypto ecosystem: national and European Members of Parliament, the European Commission, the European Council, the European Parliament, as well as the European Central Bank, and the regulatory bodies and legislators of several European countries.\n\nOur unique positioning allows us to rally decentralized DeFi actors and more traditional listed companies and corporations.\n\n- [Adan Website](https://adan.eu)\n- [Adan on Twitter](https://twitter.com/adan_asso)\n- [Adan on LinkedIn](https://www.linkedin.com/company/adaneu/)\n\n### Mission\n\n- Educate about crypto and help create better narratives around this technology and industry\n- Foster an environment favorable to the growth of the industry\n- Accompany the implementation of French and European regulatory frameworks adapted to the specificities of the sector\n- Build constructive relations between the industry and public institutions\n\n### Team\n\n- Faustine Fleuret: President \u0026 CEO | [Twitter 8](https://twitter.com/faufleuret) / [Linkedin](https://www.linkedin.com/in/faustine-fleuret-640b67a4/)\n- Mélodie Ambroise: Head of Strategy \u0026 Institutional Relations | [Twitter 3](https://twitter.com/mambroise23) / [Linkedin](https://www.linkedin.com/in/m%C3%A9lodie-ambroise/)\n- Jules Dubourg: Administrative \u0026 Financial Manager | [Twitter 1](https://twitter.com/Jules_Dubourg) / [Linkedin 1](https://www.linkedin.com/in/jules-dubourg/)\n- Hugo Bordet: Regulatory Affairs Manager | [Twitter](https://twitter.com/HugoBordet1) / [Linkedin 1](https://www.linkedin.com/in/hugo-bordet-598241152/)\n- Dorian Ravaute: Fiscal Affairs Officer |  [Linkedin 1](https://www.linkedin.com/in/dorianravaute/)\n\n### Funding\n\nAdan is a membership organization and is funded primarily through annual fees.\n\nBeing aware that our missions concern not only the Cosmos community but the whole crypto ecosystem, we will ask for grants from other blockchains. Thus the costs will be shared between different communities.\n\nFinally, we will ask the Cosmos community once only. Our growth plan and the opening of the European market will allow us to be financially self-sufficient through membership fees - which today represent over 80% of our annual income.\n\n### Governance\n\nMembers define the yearly objectives during the annual General Assembly. They also vote on the budget, ratify the association's actions, and elect a Board of Directors responsible for representing them and controlling the implementation of these missions.\n\nSee our website for more information about Adan's governance and [Board of Directors 3](https://adan.eu/en/association/governance).\n\nAdan is a non-profit organization registered in France (Association loi de 1901).\n\n### Works \u0026 Publications\n\nAdan interacts with all stakeholders with an interest in the crypto ecosystem: national and European Members of Parliament, the European Commission, the European Council, the European Parliament, as well as the European Central Bank, and the regulatory bodies and legislators of several European countries.\n\nAdan is also involved in discussions with international bodies such as FATF, IOSO, BIS etc.\n\nIn just two short years, Adan has produced an astounding amount of writing and [publications](https://adan.eu/en/publications) to support its mission:\n\n- [A crypto-euro issued by an American giant, or how Europe is delegating its monetary sovereignty](https://adan.eu/en/tribune-les-echos-crypto-euro-en)\n- [EU framework for crypto-asset markets: the French Presidency ends with political deals on MiCA and TFR](https://adan.eu/en/press-release-policy-agreements-mica)\n- [Adan's Response to IOSCO's Retail Market Conduct Report](https://adan.eu/en/consultation/en/response-report-iosco-retail-market)\n- [Adoption of TFR in the European Parliament: the fight against financial crime should not be a fight against crypto-assets](https://adan.eu/en/press/tfr-travel-rule-vote-european-parliament-europeen-econ-libe)\n- [MiCA vote in the European Parliament: A step forward or backward for the crypto sector?](https://adan.eu/en/press-release/european-parliament-mica-compromise-crypto)\n- [Adan responds to the EBA consultation on its draft guidelines for remote onboarding customer solutions 1](https://adan.eu/en/consultation/response-guidelines-eba-onboarding-solutions)\n- [Ban of Proof-of-Work protocols: wrong answer to real global environmental challenges](https://adan.eu/en/position/pow-bitcoin-ban-eu-inappropriate-answer)\n- [Adan's position on FATF's updated guidance for a risk-based approach 1](https://adan.eu/en/position/fatf-updated-guidance-vasp)\n\n## Proposal details\n\nThe crypto industry is evolving rapidly, challenges are multiplying, and public authorities worldwide are increasingly seeking to regulate crypto innovation.\n\nTo this end, we will continue to fight the battles initiated at the French level but must accelerate on the European and international scene according to 4 priorities:\n\n- Monitor upcoming legislation and regulations, such as MiCA, TFR, and the remainder of the AML package.\n- Contribute to elaborating regulatory frameworks in preparation surrounding topics like DeFi, NFTs, the environmental impact of crypto, etc.\n- Establish strong positions on European crypto companies' issues (e.g., access to banking and insurance) and convey them to policymakers.\n- Sensibly educate on regulatory proposals which fail to capture the unique properties of crypto-economic models and risk hindering innovation (e.g., regulating DAOs as legal persons in the traditional sense).\n\nTo overcome these challenges, our team must engage in the following activities:\n\n- Carry out diligent monitoring of the legislative and regulatory agenda\n- Think up pragmatic solutions adapted to the sector within our working groups\n- Dialogue with decision-makers in European institutions (European Commission, Council of the European Union, European Parliament), European authorities (European Central Bank, European Banking Authority, etc.), and international bodies (FATF, IOSCO, BIS, etc.)\n- Create synergies with other market players with shared interests\n\nGiven the size and importance of the mission, which is constantly expanding, Adan must strengthen its team. Thus we request funding from the Cosmos Hub community pool to recruit a European Affairs Officer based in Brussels, allowing us to further increase our ties with transnational bodies.\n\n## Deliverables\n\nWe believe transparency around community funding is important for building trust and establishing a reputation. This is why we pledge to publish progress reports in 6 and 12 months such that the Cosmos Community better understands how funds are spent and may evaluate the return on its investment.\n\nThis report will be delivered to you via this forum. It will consist of different sections, such as :\n\n- actions carried out in the last months;\n- missions to be carried out in the coming months;\n- distribution of the remaining allocation.\n\n## Funding Amount\n\nTotal amount requested: 8,000 ATOM\n\nThis corresponds to roughly 100,000 EUR and covers gross salary, recruitment costs, and travel expenses for half a year.\n\n## Receipient\n\ncosmos1kdff80vxuj0zmmjauymzjs40hsfkuya79s8tm0\n\n## IPFS\n\nQmR1q2i48EJqaZSXxgggE8VaPKsZFtozVBPHKejMpJAYXx\n\n## Governance Discussion\n\nhttps://forum.cosmos.network/t/proposal-draft-adan-non-profit-fighting-for-sound-crypto-regulation/7416\n\n## Governance votes\n\nThe following items summarize the voting options and what it means for this proposal:\n\n- YES - You agree that persuading regulators to adopt sound and proportional regulation is important and thus wish to fund Adan's work\n- NO - You don't agree that persuading regulators is important and thus do not agree to fund Adan's work\n- NO WITH VETO - A 'NoWithVeto' vote indicates a proposal either (1) is deemed to be spam, i.e., irrelevant to Cosmos Hub, (2) disproportionately infringes on minority interests, or (3) violates or encourages violation of the rules of engagement as currently set out by Cosmos Hub governance. If the number of 'NoWithVeto' votes is greater than a third of the total votes, the proposal is rejected, and the deposits are burned.\n- ABSTAIN - You wish to contribute to the quorum but formally decline to vote either for or against the proposal."`
        description: String,
        /// Community pool spend proposal recipient address. Eg: `"cosmos1kdff80vxuj0zmmjauymzjs40hsfkuya79s8tm0"`
        recipient: String,
        /// Community pool spend proposal amount. Array of amounts and denoms.
        amount: Vec<DenomAmount>,
    },
    #[serde(rename = "/cosmos.params.v1beta1.ParameterChangeProposal")]
    ParameterChange {
        /// Parameter change proposal title. Eg: `"Adjust Blocks Per Year to 4.36M"`
        title: String,
        /// Parameter change proposal description. Eg: `"While the current inflation rate is set at 7%, the effective inflation rate is more like ~6.29%. This is because blocks have slowed down somewhat from ~6.5s to ~7.24s per block, and thus the current blocks per year value of 4855015 is too high. Here we propose to adjust the blocks per year value from 4855015 to 4360000 to bring it in line with current block times, which should realign the effective inflation rate. More details were drafted on Github (https://github.com/cosmos/governance/tree/master/proposals/2020-10-blocks-per-year) and are available on IPFS (https://ipfs.io/ipfs/QmTkzDwWqPbnAh5YiV5VwcTLnGdwSNsNTn2aDxdXBFca7D/example#/ipfs/QmTZ3R4W2odBsx6hpt7awfRTfZA67x5eQaoL6ctxBr6NyN)"`
        description: String,
        /// Array of changes wanted.
        changes: Vec<ParameterChangeProposalChange>,
    },
    #[serde(rename = "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal")]
    SoftwareUpgrade {
        /// Software upgrade proposal title. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
        title: String,
        /// Software upgrade propsal description. Eg: `"Summary: Tendermint (https://tendermint.com) and B-Harvest (https://bharvest.io) have joined forces to produce and develop a Liquidity Module (https://github.com/tendermint/liquidity). This signal proposal is a Request For Comment to the ATOM community regarding the addition of this Liquidity Module into the Cosmos Hub source code.\nBy voting YES to this proposal, you will signal that you approve of having a DEX based on this specific Liquidity Module deployed on the Cosmos Hub.\nDetail of the proposal can be found at IPFS link below.\n\nCurrent Development : https://github.com/tendermint/liquidity/tree/develop\nIPFS : https://ipfs.io/ipfs/QmZpgkYLoCBnXM1S7TEdQunMmur9bAw5VTNgFQCyrqgKDd"`
        description: String,
        /// Software upgrade proposal plan.
        plan: SoftwareUpgradeProposalPlan,
    },
    #[serde(rename = "/ibc.core.client.v1.ClientUpdateProposal")]
    ClientUpdate {
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

impl InternalProposalContent {
    fn try_from_with(value: ProposalContentWithUnknown, self_calc_u128_to_f64: impl Fn(u128) -> f64) -> Result<Self, String> {
        Ok(match value {
            ProposalContentWithUnknown::KnownProposal(proposal_content) => match proposal_content {
                ProposalContent::ClientUpdate {
                    title,
                    description,
                    subject_client_id,
                    substitute_client_id,
                } => Self::ClientUpdate {
                    r#type: "ClientUpdateProposal".into(),
                    title,
                    description,
                    subject_client_id,
                    substitute_client_id,
                },
                ProposalContent::CommunityPoolSpend {
                    title,
                    description,
                    recipient,
                    amount,
                } => Self::CommunityPoolSpend {
                    r#type: "CommunityPoolSpendProposal".into(),
                    title,
                    description,
                    recipient,
                    amount: amount
                        .get(0)
                        .map(|da| da.amount.to_string())
                        .and_then(|amount| amount.parse::<u128>().ok().map(|amount| self_calc_u128_to_f64(amount)))
                        .unwrap_or(0.0),
                },
                ProposalContent::ParameterChange { title, description, changes } => Self::ParameterChange {
                    r#type: "ParameterChangeProposal".into(),
                    title,
                    description,
                    changes,
                },
                ProposalContent::SoftwareUpgrade { title, description, plan } => Self::SoftwareUpgrade {
                    r#type: "SoftwareUpgradeProposal".into(),
                    title,
                    description,
                    plan: plan.try_into()?,
                },
            },
            ProposalContentWithUnknown::UnknownProposal { r#type, title, description } => Self::Unknown {
                r#type: r#type.split('.').last().unwrap_or("Unknown").to_string(),
                title,
                description,
            },
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum ProposalContentWithUnknown {
    KnownProposal(ProposalContent),
    UnknownProposal {
        #[serde(rename = "@type")]
        r#type: String,
        #[serde(default = "default_proposal_title")]
        title: String,
        #[serde(default = "default_proposal_description")]
        description: String,
    },
}

fn default_proposal_title() -> String {
    String::from("Unknown proposal.")
}

fn default_proposal_description() -> String {
    String::from("This proposal has no description.")
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum InternalProposalContent {
    CommunityPoolSpend {
        r#type: String,
        title: String,
        description: String,
        recipient: String,
        amount: f64,
    },
    ParameterChange {
        r#type: String,
        title: String,
        description: String,
        changes: Vec<ParameterChangeProposalChange>,
    },
    SoftwareUpgrade {
        r#type: String,
        title: String,
        description: String,
        plan: InternalSoftwareUpgradeProposalPlan,
    },
    ClientUpdate {
        r#type: String,
        title: String,
        description: String,
        subject_client_id: String,
        substitute_client_id: String,
    },
    Unknown {
        r#type: String,
        title: String,
        description: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalProposalFinalTallyResult {
    /// Number of `yes` votes. Eg: `50`
    pub yes: f64,
    /// Number of `abstain` votes. Eg: `35`
    pub abstain: f64,
    /// Number of `no` votes. Eg: `12`
    pub no: f64,
    /// Number of `no with veto` votes.  Eg: `7`
    pub no_with_veto: f64,
}

impl InternalProposalFinalTallyResult {
    fn try_from_with(value: ProposalFinalTallyResult, self_u128_to_f64: impl Fn(u128) -> f64) -> Result<Self, String> {
        Ok(Self {
            yes: self_u128_to_f64(
                value
                    .yes
                    .parse()
                    .map_err(|_| format!("Cannot parse 'yes' votes count, '{}'.", value.yes))?,
            ),
            abstain: self_u128_to_f64(
                value
                    .abstain
                    .parse()
                    .map_err(|_| format!("Cannot parse 'abstain' votes count, '{}'.", value.abstain))?,
            ),
            no: self_u128_to_f64(value.no.parse().map_err(|_| format!("Cannot parse 'no' votes count, '{}'.", value.no))?),
            no_with_veto: self_u128_to_f64(
                value
                    .no_with_veto
                    .parse()
                    .map_err(|_| format!("Cannot parse 'no' votes with veto count, '{}'.", value.no_with_veto))?,
            ),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SoftwareUpgradeProposalPlan {
    /// Software upgrade proposal plan name. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
    pub name: String,
    /// Software upgrade proposal plan time. Eg: `"9999-12-31T00:00:00Z"`
    pub time: String,
    /// Software upgrade proposal plan height. Eg: `"0"`
    pub height: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InternalSoftwareUpgradeProposalPlan {
    /// Software upgrade proposal plan name. Eg: `"Signal Proposal to Adopt the Liquidity Module onto the Cosmos Hub"`
    pub name: String,
    /// Software upgrade proposal plan timestamp in milliseconds.
    pub time: i64,
    /// Software upgrade proposal plan height. Eg: `0`
    pub height: u64,
}

impl TryFrom<SoftwareUpgradeProposalPlan> for InternalSoftwareUpgradeProposalPlan {
    type Error = String;
    fn try_from(value: SoftwareUpgradeProposalPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            time: DateTime::parse_from_rfc3339(&value.time)
                .map_err(|_| format!("Cannot parse software upgrade proposal plan datetime, '{}'.", value.time))?
                .timestamp_millis(),
            height: value
                .height
                .parse()
                .map_err(|_| format!("Cannot parse software upgrade proposal plan height, '{}'.", value.height))?,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ParameterChangeProposalChange {
    /// Subspace. Eg: `"mint"`
    pub subspace: String,
    /// Key. Eg: `"BlocksPerYear"`
    pub key: String,
    /// Value. Inside quotes. Eg: `"\"4360000\""`
    pub value: String,
}

#[derive(Serialize, Debug)]
pub struct ProposalItem {
    /// Proposal ID.
    pub proposal_id: u32,
    /// Proposal Title.
    pub title: String,
    /// Proposal ID. Eg: `79`
    pub description: String,
    /// Voting start timestamp in milliseconds.
    pub time: i64,
    /// Proposal status.
    pub status: String,
}
