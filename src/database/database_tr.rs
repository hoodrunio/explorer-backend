use futures::StreamExt;
use mongodb::bson::{from_document, to_bson, to_document};
use mongodb::options::FindOptions;
use mongodb::IndexModel;
use mongodb::{
    bson::{doc, Document},
    Client, Collection, Database,
};
use mongodb_cursor_pagination::{FindResult, PaginatedCursor};

use crate::database::blocks::Block;
use crate::database::params::{HistoricalValidatorData, VotingPower};
use crate::database::{
    ChainDashboardInfoForDb, EvmPollForDb, EvmPollParticipantForDb, HeartbeatForDb, ListDbResult, TokenMarketPriceHistoriesForDb, TransactionForDb,
    ValidatorForDb,
};
use crate::fetch::evm::{EvmSupportedChains, PollStatus};
use crate::routes::PaginationData;

use super::ProposalVoteForDb;
use super::{params::Params, validators::Validator};

// Testnetrun explorer database.
#[derive(Clone)]
pub struct DatabaseTR {
    /// The MongoDB client that works with a MongoDB instance.
    mongo: Client,

    /// Database name and chain name are the same.
    db_name: String,
}

impl DatabaseTR {
    /// Connects to MongoDB instance at given URI and creates a client to work with that instance.
    /// # Usage
    /// ```rs
    /// let database = Database::new();
    /// ```
    pub async fn new() -> DatabaseTR {
        // Change this URI and create a database for each chain using chain names.
        let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set in .env file");

        DatabaseTR {
            mongo: (Client::with_uri_str(uri).await.expect("Cannot connect to MongoDB instance.")),
            db_name: "unexpected_db".to_string(),
        }
    }

    /// Changes the name of the database and returns a new one.
    pub fn change_name(self, db_name: &str) -> DatabaseTR {
        DatabaseTR {
            db_name: db_name.to_string(),
            ..self
        }
    }

    /// Returns the MongoDB database.
    /// # Usage
    /// ```rs
    /// let db = database.get_db();
    /// ```
    fn db(&self) -> Database {
        self.mongo.database(&self.db_name)
    }

    /// Returns the validators collection.
    /// # Usage
    /// ```rs
    /// let collection = database.validators_collection();
    /// ```
    fn validators_collection(&self) -> Collection<Validator> {
        self.db().collection("validators")
    }

    /// Returns the transactions collection.
    /// # Usage
    /// ```rs
    /// let collection = database.transactions_collection();
    /// ```
    fn transactions_collection(&self) -> Collection<TransactionForDb> {
        self.db().collection("transactions")
    }

    /// Returns the historical data collection.
    /// # Usage
    /// ```rs
    /// let collection = database.historical_data_collection();
    /// ```
    fn historical_data_collection(&self) -> Collection<HistoricalValidatorData> {
        self.db().collection("historical_data")
    }

    /// Returns the chain dashboard info.
    /// # Usage
    /// ```rs
    /// let collection = database.dashboard_info_collection();
    /// ```
    fn dashboard_info_collection(&self) -> Collection<ChainDashboardInfoForDb> {
        self.db().collection("dashboard")
    }

    /// Returns the params collection.
    /// # Usage
    /// ```rs
    /// let collection = database.blocks_collection();
    /// ```
    fn blocks_collection(&self) -> Collection<Block> {
        self.db().collection("blocks")
    }

    /// Returns the evm poll collection.
    /// # Usage
    /// ```rs
    /// let collection = database.evm_poll_collection();
    /// ```
    fn evm_poll_collection(&self) -> Collection<EvmPollForDb> {
        self.db().collection("evm_polls")
    }

    /// Returns the heartbeats collection.
    /// # Usage
    /// ```rs
    /// let collection = database.heartbeat_collection();
    /// ```
    fn heartbeat_collection(&self) -> Collection<HeartbeatForDb> {
        self.db().collection("heartbeats")
    }

    /// Returns the proposal votes collection.
    /// # Usage
    /// ```rs
    /// let collection = database.propsals_votes_collection();
    /// ```
    fn propsals_votes_collection(&self) -> Collection<ProposalVoteForDb> {
        self.db().collection("proposals_votes")
    }

    /// Returns the market price history collection.
    /// # Usage
    /// ```rs
    /// let collection = database.market_price_history();
    /// ```
    fn market_price_history(&self) -> Collection<TokenMarketPriceHistoriesForDb> {
        self.db().collection("market_price_history")
    }

    pub async fn upsert_validator(&self, validator: Validator) -> Result<(), String> {
        let doc = to_document(&validator).unwrap();
        let command = doc! {"update":"validators","updates":[{"q":{"operator_address":&validator.operator_address},"u":doc,"upsert":true}]};
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save the validator.".into()),
        }
    }

    /// Adds a new transaction to the transactions collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_transaction(transaction).await;
    /// ```
    pub async fn add_transaction(&self, transaction: TransactionForDb) -> Result<(), String> {
        match self.transactions_collection().insert_one(transaction, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save the transaction.".into()),
        }
    }

    /// Adds new propsal vote to the propsals votes collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_propsal_vote(vote).await;
    /// ```
    pub async fn add_propsal_vote(&self, proposal_vote: ProposalVoteForDb) -> Result<(), String> {
        match self.propsals_votes_collection().insert_one(proposal_vote, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save proposal vote.".into()),
        }
    }

    /// Finds a counted transaction from the transactions collection.
    /// # Usage
    /// ```rs
    /// database.find_last_count_transactions(vec<doc!{}>,10).await;
    /// ```
    pub async fn find_last_count_transactions(&self, pipeline: Option<Vec<Document>>, count: u16) -> Result<Vec<TransactionForDb>, String> {
        let mut pipeline_docs = vec![];

        let sort_pipe = doc! { "$sort": {"time": -1} };
        let limit_pipe = doc! { "$limit": count as i64 };

        pipeline_docs.push(sort_pipe);

        if let Some(pipeline) = pipeline {
            pipeline_docs.extend(pipeline);
        };

        pipeline_docs.push(limit_pipe);

        let mut results = self
            .transactions_collection()
            .aggregate(pipeline_docs, None)
            .await
            .map_err(|e| e.to_string())?;

        let mut res: Vec<TransactionForDb> = vec![];
        while let Some(result) = results.next().await {
            res.push(from_document(result.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?);
        }

        Ok(res)
    }

    /// Adds new block item to the blocks collection
    /// # Usage
    /// ```rs
    /// database.upsert_block(block).await;
    /// ```
    pub async fn upsert_block(&self, block: Block) -> Result<(), String> {
        let doc = to_document(&block).unwrap();
        let command = doc! {"update":"blocks","updates":[{"q":{"hash":&block.hash},"u":doc,"upsert":true}]};
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cannot save the block to db: {e}")),
        }
    }

    /// Finds a sorted blocks list by given document.
    /// # Usage
    /// ```rs
    /// let blocks = database.find_paginated_blocks().await;
    /// ```
    pub async fn find_paginated_blocks(&self, query: Option<Document>, config: PaginationData) -> Result<ListDbResult<Block>, String> {
        let collection = self.db().collection("blocks");

        let sort_doc = doc! { "timestamp": - 1};

        let index_doc = sort_doc.clone();
        let _ = collection.create_index(IndexModel::builder().keys(index_doc).build(), None).await;

        let find_options = FindOptions::builder()
            .sort(sort_doc)
            .limit(config.limit.map(|l| l as i64).unwrap_or_else(|| 20))
            .build();

        let results = PaginatedCursor::new(Some(find_options), config.cursor, None)
            .find(&collection, query.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(ListDbResult::from(results))
    }

    /// Finds counted blocks in the blocks collection
    /// # Usage
    /// ```rs
    /// database.find_last_count_blocks(vec<doc!{}>,10).await;
    /// ```
    pub async fn find_last_count_blocks(&self, pipeline: Option<Vec<Document>>, count: u16) -> Result<Vec<Block>, String> {
        let mut pipeline_docs = vec![];

        let sort_pipe = doc! { "$sort": {"height": -1} };
        let limit_pipe = doc! { "$limit": count as i64 };

        pipeline_docs.push(sort_pipe);

        if let Some(pipeline) = pipeline {
            pipeline_docs.extend(pipeline);
        };

        pipeline_docs.push(limit_pipe);
        let mut results = self.blocks_collection().aggregate(pipeline_docs, None).await.map_err(|e| e.to_string())?;

        let mut res: Vec<Block> = vec![];
        while let Some(result) = results.next().await {
            res.push(from_document(result.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?);
        }

        Ok(res)
    }

    /// Finds a sorted txs list by given document.
    /// # Usage
    /// ```rs
    /// let txs = database.find_paginated_txs().await;
    /// ```
    pub async fn find_paginated_txs(&self, query: Option<Document>, config: PaginationData) -> Result<ListDbResult<TransactionForDb>, String> {
        let collection = self.db().collection("transactions");
        let sort_doc = doc! {"time":-1};

        let index_doc = sort_doc.clone();
        let _ = collection.create_index(IndexModel::builder().keys(index_doc).build(), None).await;

        let find_options = FindOptions::builder()
            .sort(sort_doc)
            .limit(config.limit.map(|l| l as i64).unwrap_or_else(|| 20))
            .build();

        let results = PaginatedCursor::new(Some(find_options), config.cursor, None)
            .find(&collection, query.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(ListDbResult::from(results))
    }

    /// Finds a validator by given document.
    /// # Usage
    /// ```rs
    /// let validator = database.find_validator(doc!("operator_address": address)).await;
    /// ```
    pub async fn find_validator(&self, doc: Document) -> Result<Validator, String> {
        match self.validators_collection().find_one(doc, None).await {
            Ok(potential_validator) => match potential_validator {
                Some(validator) => Ok(validator),
                None => Err("No validator is found.".into()),
            },
            Err(_) => Err("Cannot make request to DB.".into()),
        }
    }

    /// Finds a sorted validator list by given document.
    /// # Usage
    /// ```rs
    /// let validator = database.find_paginated_validators(doc!{"$match":{"operator_address":{"$exists":true}}}).await;
    /// ```
    pub async fn find_paginated_validators(&self, query: Option<Document>, config: PaginationData) -> Result<ListDbResult<ValidatorForDb>, String> {
        let collection = self.db().collection("validators");
        let sort_doc = doc! {"rank":1};

        let index_doc = sort_doc.clone();
        let _ = collection.create_index(IndexModel::builder().keys(index_doc).build(), None).await;

        let find_options = FindOptions::builder()
            .sort(sort_doc)
            .limit(config.limit.map(|l| l as i64).unwrap_or_else(|| 20))
            .build();

        let results = PaginatedCursor::new(Some(find_options), config.cursor, None)
            .find(&collection, query.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(ListDbResult::from(results))
    }
    /// Finds a sorted validator list by given document.
    /// # Usage
    /// ```rs
    /// let validator = database.find_validators(doc!{"$match":{"operator_address":{"$exists":true}}}).await;
    /// ```
    pub async fn find_validators(&self, pipe: Option<Document>) -> Result<Vec<Validator>, String> {
        let default_filter = doc! {"$match":{"operator_address":{"$exists":true}}};
        //default filter necessary when using aggregate
        let mut pipeline: Vec<Document> = vec![default_filter];
        let filter = pipe.clone();
        match filter {
            None => {}
            Some(val) => pipeline.push(val),
        };

        let mut results = self.validators_collection().aggregate(pipeline, None).await.map_err(|e| e.to_string())?;

        let mut res: Vec<Validator> = vec![];
        while let Some(result) = results.next().await {
            res.push(from_document(result.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?);
        }

        Ok(res)
    }
    /// Finds a validator by operator address.
    /// # Usage
    /// ```rs
    /// let validator = database.find_validator_by_operator_addr(operator_address).await;
    /// ```
    pub async fn find_validator_by_operator_addr(&self, operator_address: &str) -> Result<Validator, String> {
        self.find_validator(doc! {"operator_address": operator_address}).await
    }

    /// Finds a validator by hex address.
    /// # Usage
    /// ```rs
    /// let validator = database.find_validator_by_hex_addr(hex_address).await;
    /// ```
    pub async fn find_validator_by_hex_addr(&self, hex_address: &str) -> Result<Validator, String> {
        self.find_validator(doc! {"hex_address": hex_address.to_ascii_uppercase()}).await
    }

    /// Updates validator on to the validators collection
    /// # Usage
    /// ```rs
    /// database.update_validator(doc,doc).await;
    /// ```
    pub async fn update_validator(&self, query: Document, update: Document) -> Result<(), String> {
        match self.validators_collection().update_one(query, update, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Can not update validator poll {}", e)),
        }
    }

    /// Updates validator supported chains on to the validators collection
    /// # Usage
    /// ```rs
    /// database.update_validator(doc,doc).await;
    /// ```
    pub async fn update_validator_supported_chains(&self, operator_address: &String, chains: Vec<String>) -> Result<(), String> {
        let query = doc! {"operator_address": operator_address};
        let bson_doc = to_bson(&chains).unwrap();
        let update_query = doc! {"$set": {"supported_evm_chains": bson_doc}};
        match self.update_validator(query, update_query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Can not update validator supported chains {}", e)),
        }
    }

    /// Finds evm_polls with pagination option
    /// # Usage
    /// ```rs
    /// database.find_paginated_evm_polls(evm_poll).await;
    /// ```
    pub async fn find_paginated_evm_polls(&self, pipe: Option<Document>, config: PaginationData) -> Result<ListDbResult<EvmPollForDb>, String> {
        let collection = self.db().collection("evm_polls");
        let sort_doc = doc! { "timestamp": -1};

        let index_doc = sort_doc.clone();
        let _ = collection.create_index(IndexModel::builder().keys(index_doc).build(), None).await;

        let options = FindOptions::builder().limit(config.limit.map(|l| l as i64)).sort(sort_doc).build();

        let results: FindResult<EvmPollForDb> = PaginatedCursor::new(Some(options), config.cursor, config.direction.map(|d| d.into()))
            .find(&collection, pipe.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(ListDbResult::from(results))
    }
    /// Finds evm_polls with pagination option
    /// # Usage
    /// ```rs
    /// database.find_paginated_evm_polls(evm_poll).await;
    /// ```
    pub async fn find_validator_supported_chains(&self, operator_address: &String) -> Result<EvmSupportedChains, String> {
        let pipeline: Vec<Document> = vec![doc! {"$match":{"operator_address": operator_address}}];

        let mut results = self.validators_collection().aggregate(pipeline, None).await.map_err(|e| e.to_string())?;

        let mut res: Vec<String> = vec![];
        while let Some(result) = results.next().await {
            let val = from_document::<Validator>(result.map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            res = val.supported_evm_chains.unwrap_or(vec![]);
        }

        Ok(res)
    }

    /// Add new evm_poll item to the evm_polls collection
    /// # Usage
    /// ```rs
    /// database.upsert_block(evm_poll).await;
    /// ```
    pub async fn upsert_evm_poll(&self, poll: EvmPollForDb) -> Result<(), String> {
        let doc = to_document(&poll).unwrap();
        let command = doc! {"update":"evm_polls","updates":[{"q":{"poll_id":&poll.poll_id},"u":doc,"upsert":true}]};
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save the poll id to db.".into()),
        }
    }

    /// Finds a validator by given document.
    /// # Usage
    /// ```rs
    /// let evm_poll = database.find_validator(doc!("operator_address": address)).await;
    /// ```
    pub async fn find_evm_poll(&self, doc: Document) -> Result<EvmPollForDb, String> {
        match self.evm_poll_collection().find_one(doc, None).await {
            Ok(potential_validator) => match potential_validator {
                Some(poll) => Ok(poll),
                None => Err("No poll is found.".into()),
            },
            Err(_) => Err("Cannot make request to DB.".into()),
        }
    }

    /// Add new evm_poll item to the evm_polls collection
    /// # Usage
    /// ```rs
    /// database.upsert_block(evm_poll).await;
    /// ```
    pub async fn update_evm_poll(&self, query: Document, update: Document) -> Result<(), String> {
        match self.evm_poll_collection().update_one(query, update, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Can not update evm poll {}", e)),
        }
    }

    pub async fn upsert_evm_poll_participant(&self, poll_participant: EvmPollParticipantForDb) -> Result<(), String> {
        let doc = to_document(&poll_participant).unwrap();
        let command = doc! {"update":"evm_poll_participants","updates":[{"q":{"poll_id":&poll_participant.poll_id,"operator_address":&poll_participant.operator_address},"u":doc,"upsert":true}]};
        if (self.db().run_command(command, None).await).is_err() {
            return Err("Cannot save the evm poll participant.".into());
        };

        Ok(())
    }

    /// Finds find_paginated_evm_poll_participants with pagination option
    /// # Usage
    /// ```rs
    /// database.find_paginated_evm_poll_participants(pipe,config).await;
    /// ```
    pub async fn find_paginated_evm_poll_participants(
        &self,
        pipe: Option<Document>,
        config: PaginationData,
    ) -> Result<ListDbResult<EvmPollParticipantForDb>, String> {
        let collection = self.db().collection("evm_poll_participants");
        let sort_doc = doc! { "poll_id": -1};

        let index_doc = sort_doc.clone();
        let _ = collection.create_index(IndexModel::builder().keys(index_doc).build(), None).await;

        let options = FindOptions::builder().limit(config.limit.map(|l| l as i64)).sort(sort_doc).build();

        let results: FindResult<EvmPollParticipantForDb> = PaginatedCursor::new(Some(options), config.cursor, config.direction.map(|d| d.into()))
            .find(&collection, pipe.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(ListDbResult::from(results))
    }

    /// Updates evm_poll item status on to the evm_polls collection
    /// # Usage
    /// ```rs
    /// database.update_evm_poll_status(3890,PollStatus::YES).await;
    /// ```
    pub async fn update_evm_poll_status(&self, pool_id: &String, poll_status: &PollStatus) -> Result<(), String> {
        let query = doc! {"poll_id": pool_id};
        let bson_doc = to_bson(poll_status).unwrap();
        let update_query = doc! {"$set": {"status": bson_doc}};
        match self.update_evm_poll(query, update_query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Adds a new heartbeat to the heartbeats collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_heartbeat(heartbeat).await;
    /// ```
    pub async fn upsert_heartbeat(&self, heartbeat: HeartbeatForDb) -> Result<(), String> {
        let doc = to_document(&heartbeat).unwrap();
        let command = doc! {"update":"heartbeats","updates":[{"q":{"id":&heartbeat.id},"u":doc,"upsert":true}]};
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot upsert the hearbeat.".into()),
        }
    }

    /// Adds a new heartbeats to the heartbeats collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_heartbeat_many(heartbeat).await;
    /// ```
    pub async fn add_heartbeat_many(&self, heartbeats: Vec<HeartbeatForDb>) -> Result<(), String> {
        match self.heartbeat_collection().insert_many(heartbeats, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save the heartbeat.".into()),
        }
    }

    /// Finds a sorted hearbeats list by given document.
    /// # Usage
    /// ```rs
    /// let hearbeats = database.find_heartbeats(doc!{"$match":{"voter_address":"axelar1k3h51l35g5hb3lh4kjg34"}}).await;
    /// ```
    pub async fn find_paginated_heartbeats(
        &self,
        filter: Option<Document>,
        config: Option<PaginationData>,
    ) -> Result<ListDbResult<HeartbeatForDb>, String> {
        let collection = self.db().collection("heartbeats");
        let sort_doc = doc! { "_id": -1};

        let index_doc = sort_doc.clone();
        let _ = collection.create_index(IndexModel::builder().keys(index_doc).build(), None).await;

        let config = config.unwrap_or_default();
        let options = FindOptions::builder()
            .limit(config.limit.map(|l| l as i64).unwrap_or_else(|| 20))
            .sort(sort_doc)
            .build();

        let results: FindResult<HeartbeatForDb> = PaginatedCursor::new(Some(options), config.cursor, config.direction.map(|d| d.into()))
            .find(&collection, filter.as_ref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(ListDbResult::from(results))
    }

    /// Finds a validator by given document.
    /// # Usage
    /// ```rs
    /// database.upsert_params(params).await;
    /// ```
    pub async fn upsert_params(&self, params: Params) -> Result<(), String> {
        let doc = to_document(&params).unwrap();
        let command = doc! {"update":"params","updates":[{"q":{"staking":{"$exists":true}},"u":doc,"upsert":true}]};
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cannot save the params: {e}")),
        }
    }

    /// Upsert a voting power data to historical data collection.
    /// # Usage
    /// ```rs
    /// database.upsert_voting_power_data(operator_address, voting_power_data).await;
    /// ```
    pub async fn upsert_voting_power_data(&self, operator_address: &str, voting_power_data: VotingPower) -> Result<(), String> {
        let doc = to_document(&voting_power_data).unwrap();
        let command = doc! {
            "update":"historical_data",
            "updates":[{
                "q":{"operator_address":operator_address},
                "u":{"$push":{"voting_power_data":doc}},
                "upsert":true}]
        };
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cannot save the params: {e}")),
        }
    }

    /// # Usage
    /// ```rs
    /// database.insert_market_price_history(token_market_price_histories).await;
    /// ```
    pub async fn insert_market_price_history(&self, token_market_price_histories: TokenMarketPriceHistoriesForDb) -> Result<(), String> {
        let doc = to_document(&token_market_price_histories).unwrap();
        let command = doc! {
            "update":"market_price_history",
            "updates":[{
                "q":{"token": token_market_price_histories.token.clone()},
                "u":{"$set":doc},
                "upsert":true}
                ]
        };
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cannot save the market_price_history: {e}")),
        }
    }

    /// # Usage
    /// ```rs
    /// database.insertfind_market_history_market_price_history(token_market_price_histories).await;
    /// ```
    pub async fn find_market_history(&self, token: String) -> Result<TokenMarketPriceHistoriesForDb, String> {
        let filter = doc! {"token":token};

        match self.market_price_history().find_one(filter, None).await {
            Ok(history) => match history {
                Some(history) => Ok(history),
                None => Err("No validator is found.".into()),
            },
            Err(e) => return Err(format!("Cannot make request to DB: {e}")),
        }
    }

    /// Finds a historical data by given document.
    /// # Usage
    /// ```rs
    /// database.find_historical_data(doc).await;
    /// ```
    pub async fn find_historical_data(&self, doc: Document) -> Result<HistoricalValidatorData, String> {
        match self.historical_data_collection().find_one(doc, None).await {
            Ok(potential_h_data) => match potential_h_data {
                Some(historical_data) => Ok(historical_data),
                None => Err("No validator is found.".into()),
            },
            Err(_) => Err("Cannot make request to DB.".into()),
        }
    }

    /// Finds a historical data by given operator_address.
    /// # Usage
    /// ```rs
    /// database.find_historical_data_by_operator_address(operator_address).await;
    /// ```
    pub async fn find_historical_data_by_operator_address(&self, operator_address: &str) -> Result<HistoricalValidatorData, String> {
        self.find_historical_data(doc! {"operator_address":operator_address}).await
    }

    /// Finds a chain dashboard info data.
    /// # Usage
    /// ```rs
    /// database.find_chain_dashboard_info(operator_address).await;
    /// ```
    pub async fn find_chain_dashboard_info(&self) -> Result<ChainDashboardInfoForDb, String> {
        match self.dashboard_info_collection().find_one(doc! {}, None).await {
            Ok(potential_dashboard) => match potential_dashboard {
                Some(dashboard) => Ok(dashboard),
                None => Err("No dashboard is found.".into()),
            },
            Err(_) => Err("Cannot make request to DB.".into()),
        }
    }

    /// Finds a chain dashboard info data.
    /// # Usage
    /// ```rs
    /// database.find_chain_dashboard_info(operator_address).await;
    /// ```
    pub async fn upsert_chain_dashboard_info(&self, info: ChainDashboardInfoForDb) -> Result<(), String> {
        let doc = to_document(&info).unwrap();
        let command = doc! {"update":"dashboard","updates":[{"q":{"total_supply":{"$exists":true}},"u":doc,"upsert":true}]};
        match self.db().run_command(command, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Cannot save the dashboard: {e}")),
        }
    }
}
