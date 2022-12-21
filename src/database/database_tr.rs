use mongodb::{
    bson::{doc, Document},
    Client, Collection, Database,
};

use super::{chains::Chain, params::Params, validators::Validator};

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
        let uri = "mongodb://127.0.0.1:27017";

        DatabaseTR {
            mongo: (Client::with_uri_str(uri).await.expect("Cannot connect to MongoDB instance.")),
            db_name: "unexpected_db".to_string(),
        }
    }

    /// Changes the name of the database and returns a new one.
    pub fn change_name(self, db_name: &str) -> DatabaseTR {
        DatabaseTR { db_name: db_name.to_string(), ..self }
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

    /// Returns the chains collection.
    /// # Usage
    /// ```rs
    /// let collection = database.chains_collection();
    /// ```
    fn chains_collection(&self) -> Collection<Chain> {
        self.db().collection("chains")
    }

    /// Returns the params collection.
    /// # Usage
    /// ```rs
    /// let collection = database.chains_collection();
    /// ```
    fn params_collection(&self) -> Collection<Params> {
        // PARAMS is not an array, so it should not add a new item to an array.
        // It should modify the same data.
        // Once it's done, create a cron_job for params.
        todo!();
        // self.db().collection("params");
    }

    /// Adds a new validator to the validators collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_validator(validator).await;
    /// ```
    pub async fn add_validator(&self, validator: Validator) -> Result<(), String> {
        match self.validators_collection().insert_one(validator, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save the validator.".into()),
        }
    }

    /// Adds new validators to the validators collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_validators(validators).await;
    /// ```
    pub async fn add_validators(&self, validators: Vec<Validator>) -> Result<(), String> {
        match self.validators_collection().insert_many(validators, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save validators.".into()),
        }
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
        self.find_validator(doc! {"hex_address": hex_address}).await
    }

    /// Adds a new chain to the chains collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_chain(chain).await;
    /// ```
    async fn add_chain(&self, chain: Chain) -> Result<(), String> {
        match self.chains_collection().insert_one(chain, None).await {
            Ok(_) => Ok(()),
            Err(_) => Err("Cannot save the chain.".into()),
        }
    }

    /// Finds a validator by given document.
    /// # Usage
    /// ```rs
    /// let validator = database.find_validator(doc!("operator_address": address)).await;
    /// ```
    async fn find_chain(&self, name: &str) -> Result<Chain, String> {
        match self.chains_collection().find_one(doc! {"name":name }, None).await {
            Ok(potential_chain) => match potential_chain {
                Some(chain) => Ok(chain),
                None => Err("No chain is found.".into()),
            },
            Err(_) => Err("Cannot make request to DB.".into()),
        }
    }

    // Updates params collection of the database.
    // # Usage
    // ```rs
    // database.add_params(params).await;
    // ```
    //  async fn add_params(&self, params: Params) -> Result<(), String> {
    //      match self.chains_collection().insert_one(params, None).await {
    //          Ok(_) => Ok(()),
    //          Err(_) => Err("Cannot save the chain.".into()),
    //      }
    //  }
}
