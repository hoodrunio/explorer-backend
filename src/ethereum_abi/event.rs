use anyhow::{anyhow, Result};
use ethereum_types::H256;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

use crate::ethereum_abi::{DecodedParams, Param, Type, Value};

/// Contract event definition.
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Event {
    /// Event name.
    pub name: String,
    /// Event inputs.
    pub inputs: Vec<Param>,
    /// Whether the event is anonymous or not.
    pub anonymous: bool,
}

impl Event {
    /// Returns the event's signature.
    pub fn signature(&self) -> String {
        format!(
            "{}({})",
            self.name,
            self.inputs
                .iter()
                .map(|param| param.type_.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    /// Compute the event's topic hash
    pub fn topic(&self) -> H256 {
        use tiny_keccak::{Hasher, Keccak};

        let mut keccak_out = [0u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(self.signature().as_bytes());
        hasher.finalize(&mut keccak_out);

        H256::from_slice(&keccak_out)
    }

    /// Decode event params from a log's topics and data.
    pub fn decode_data_from_slice(
        &self,
        mut topics: &[H256],
        data: &[u8],
    ) -> Result<DecodedParams> {
        // strip event topic from the topics array
        // so that we end up with only the values we
        // need to decode
        if !self.anonymous {
            topics = topics
                .get(1..)
                .ok_or_else(|| anyhow!("missing event topic"))?;
        }

        let mut topics_values = VecDeque::from(topics.to_vec());

        let mut data_values = VecDeque::from(Value::decode_from_slice(
            data,
            &self
                .inputs
                .iter()
                .filter(|input| !input.indexed.unwrap_or(false))
                .map(|input| input.type_.clone())
                .collect::<Vec<_>>(),
        )?);

        let mut decoded = vec![];
        for input in self.inputs.iter().cloned() {
            let decoded_value = if input.indexed.unwrap_or(false) {
                let val = topics_values
                    .pop_front()
                    .ok_or_else(|| anyhow!("insufficient topics entries"))?;

                let bytes = val.to_fixed_bytes().to_vec();

                if Self::is_encoded_to_keccak(&input.type_) {
                    Ok(Value::FixedBytes(bytes))
                } else {
                    Value::decode_from_slice(&bytes, &[input.type_.clone()])?
                        .first()
                        .ok_or_else(|| anyhow!("no value decoded from topics entry"))
                        .map(Clone::clone)
                }
            } else {
                data_values
                    .pop_front()
                    .ok_or_else(|| anyhow!("insufficient data values"))
            };

            decoded.push((input, decoded_value?));
        }

        Ok(DecodedParams::from(decoded))
    }

    fn is_encoded_to_keccak(ty: &Type) -> bool {
        matches!(
            ty,
            Type::FixedArray(_, _) | Type::Array(_) | Type::Bytes | Type::String | Type::Tuple(_)
        )
    }
}
