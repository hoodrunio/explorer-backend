use mongodb::{
    bson::{doc, Document},
    error::Error,
    Client, Collection, Database,
};

use super::{chains::Chain, validators::Validator};

// Testnetrun explorer database.
#[derive(Clone)]
pub struct DatabaseTR {
    /// The MongoDB client that works with a MongoDB instance.
    mongo: Client,
}

mod consts {
    /// The URI for the MongoDB instance.
    pub const MONGO_DB_URI: &str = "mongodb://db.example.com:12345";

    /// The name of the MongoDB database.
    pub const DATABASE_NAME: &str = "testnetrun";

    /// Validators database name.
    pub const VALIDATORS_COLLECTION_NAME: &str = "validators";

    /// Chains database name.
    pub const CHAINS_COLLECTION_NAME: &str = "chains";
}

impl DatabaseTR {
    /// Connects to MongoDB instance at given URI and creates a client to work with that instance.
    /// # Usage
    /// ```rs
    /// let database = Database::new();
    /// ```
    pub async fn new() -> DatabaseTR {
        DatabaseTR {
            mongo: (Client::with_uri_str(consts::MONGO_DB_URI)
                .await
                .expect("Cannot connect to MongoDB instance.")),
        }
    }

    /// Returns the MongoDB database.
    /// # Usage
    /// ```rs
    /// let db = database.get_db();
    /// ```
    fn db(&self) -> Database {
        self.mongo.database(consts::DATABASE_NAME)
    }

    /// Returns the validators collection.
    /// # Usage
    /// ```rs
    /// let collection = database.validators_collection();
    /// ```
    fn validators_collection(&self) -> Collection<Validator> {
        self.db().collection(consts::VALIDATORS_COLLECTION_NAME)
    }

    /// Returns the chains collection.
    /// # Usage
    /// ```rs
    /// let collection = database.chains_collection();
    /// ```
    fn chains_collection(&self) -> Collection<Chain> {
        self.db().collection(consts::CHAINS_COLLECTION_NAME)
    }

    /// Adds a new validator to the validators collection of the database.
    /// # Usage
    /// ```rs
    /// database.add_validator(validator).await;
    /// ```
    async fn add_validator(&self, validator: Validator) -> Result<(), String> {
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
    async fn add_validators(&self, validators: Vec<Validator>) -> Result<(), String> {
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
}
