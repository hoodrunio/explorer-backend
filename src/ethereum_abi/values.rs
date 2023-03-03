use anyhow::{anyhow, Result};
use ethereum_types::{H160, U256};
use serde::{Deserialize, Serialize};

use crate::ethereum_abi::types::Type;

/// ABI decoded value.
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub enum Value {
    /// Unsigned int value (uint<M>).
    Uint(U256, usize),
    /// Signed int value (int<M>).
    Int(U256, usize),
    /// Address value (address).
    Address(H160),
    /// Bool value (bool).
    Bool(bool),
    /// Fixed size bytes value (bytes<M>).
    FixedBytes(Vec<u8>),
    /// Fixed size array value (T\[k\]).
    FixedArray(Vec<Value>, Type),
    /// UTF-8 string value (string).
    String(String),
    /// Dynamic size bytes value (bytes).
    Bytes(Vec<u8>),
    /// Dynamic size array value (T[]).
    Array(Vec<Value>, Type),
    /// Tuple value (tuple(T1, T2, ..., Tn)).
    ///
    /// This variant's vector items have the form (name, value).
    Tuple(Vec<(String, Value)>),
}

impl Value {
    /// Decodes values from bytes using the given type hint.
    pub fn decode_from_slice(bs: &[u8], tys: &[Type]) -> Result<Vec<Value>> {
        tys.iter()
            .try_fold((vec![], 0), |(mut values, at), ty| {
                let (value, consumed) = Self::decode(bs, ty, 0, at)?;
                values.push(value);

                Ok((values, at + consumed))
            })
            .map(|(values, _)| values)
    }

    /// Encodes values into bytes.
    pub fn encode(values: &[Self]) -> Vec<u8> {
        let mut buf = vec![];
        let mut alloc_queue = std::collections::VecDeque::new();

        for value in values {
            match value {
                Value::Uint(i, _) | Value::Int(i, _) => {
                    let start = buf.len();
                    buf.resize(buf.len() + 32, 0);

                    i.to_big_endian(&mut buf[start..(start + 32)]);
                }

                Value::Address(addr) => {
                    let start = buf.len();
                    buf.resize(buf.len() + 32, 0);

                    // big-endian, as if it were a uint160.
                    buf[(start + 12)..(start + 32)].copy_from_slice(addr.as_fixed_bytes());
                }

                Value::Bool(b) => {
                    let start = buf.len();
                    buf.resize(buf.len() + 32, 0);

                    if *b {
                        buf[start + 31] = 1;
                    }
                }

                Value::FixedBytes(bytes) => {
                    let start = buf.len();
                    buf.resize(buf.len() + 32, 0);

                    buf[start..(start + bytes.len())].copy_from_slice(bytes);
                }

                Value::FixedArray(values, _) => {
                    if value.type_of().is_dynamic() {
                        alloc_queue.push_back((buf.len(), value));
                        buf.resize(buf.len() + 32, 0);
                    } else {
                        buf.extend(Self::encode(values));
                    }
                }

                Value::Tuple(values) => {
                    if value.type_of().is_dynamic() {
                        alloc_queue.push_back((buf.len(), value));
                        buf.resize(buf.len() + 32, 0);
                    } else {
                        let values: Vec<_> =
                            values.iter().cloned().map(|(_, value)| value).collect();

                        buf.extend(Self::encode(&values));
                    }
                }

                Value::String(_) | Value::Bytes(_) | Value::Array(_, _) => {
                    alloc_queue.push_back((buf.len(), value));
                    buf.resize(buf.len() + 32, 0);
                }
            };
        }

        let mut alloc_offset = buf.len();

        while let Some((at, value)) = alloc_queue.pop_front() {
            U256::from(alloc_offset).to_big_endian(&mut buf[at..(at + 32)]);

            match value {
                Value::String(s) => {
                    alloc_offset = Self::encode_bytes(&mut buf, s.as_bytes(), alloc_offset);
                }

                Value::Bytes(bytes) => {
                    alloc_offset = Self::encode_bytes(&mut buf, bytes, alloc_offset);
                }

                Value::Array(values, _) => {
                    buf.resize(buf.len() + 32, 0);

                    // write array length
                    U256::from(values.len())
                        .to_big_endian(&mut buf[alloc_offset..(alloc_offset + 32)]);
                    alloc_offset += 32;

                    // write array values
                    let bytes = Self::encode(values);
                    alloc_offset += bytes.len();
                    buf.extend(bytes);
                }

                Value::FixedArray(values, _) => {
                    // write array values
                    let bytes = Self::encode(values);
                    alloc_offset += bytes.len();
                    buf.extend(bytes);
                }

                Value::Tuple(values) => {
                    // write tuple values
                    let values: Vec<_> = values.iter().cloned().map(|(_, value)| value).collect();

                    let bytes = Self::encode(&values);
                    alloc_offset += bytes.len();
                    buf.extend(bytes);
                }

                _ => panic!("value of fixed size type {:?} in dynamic alloc area", value),
            };
        }

        buf
    }

    /// Returns the type of the given value.
    pub fn type_of(&self) -> Type {
        match self {
            Value::Uint(_, size) => Type::Uint(*size),
            Value::Int(_, size) => Type::Int(*size),
            Value::Address(_) => Type::Address,
            Value::Bool(_) => Type::Bool,
            Value::FixedBytes(bytes) => Type::FixedBytes(bytes.len()),
            Value::FixedArray(values, ty) => Type::FixedArray(Box::new(ty.clone()), values.len()),
            Value::String(_) => Type::String,
            Value::Bytes(_) => Type::Bytes,
            Value::Array(_, ty) => Type::Array(Box::new(ty.clone())),
            Value::Tuple(values) => Type::Tuple(
                values
                    .iter()
                    .map(|(name, value)| (name.clone(), value.type_of()))
                    .collect(),
            ),
        }
    }

    fn decode(bs: &[u8], ty: &Type, base_addr: usize, at: usize) -> Result<(Value, usize)> {
        match ty {
            Type::Uint(size) => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding uint{}", size))?;

                let uint = U256::from_big_endian(slice);

                Ok((Value::Uint(uint, *size), 32))
            }

            Type::Int(size) => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding int{}", size))?;

                let uint = U256::from_big_endian(slice);

                Ok((Value::Int(uint, *size), 32))
            }

            Type::Address => {
                let at = base_addr + at;
                let slice = bs
                    .get((at + 12)..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding address"))?;

                // big-endian, same as if it were a uint160.
                let addr = H160::from_slice(slice);

                Ok((Value::Address(addr), 32))
            }

            Type::Bool => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bool"))?;

                let b = U256::from_big_endian(slice) == U256::one();

                Ok((Value::Bool(b), 32))
            }

            Type::FixedBytes(size) => {
                let at = base_addr + at;
                let bv = bs
                    .get(at..(at + size))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bytes{}", size))?
                    .to_vec();

                Ok((Value::FixedBytes(bv), Self::padded32_size(*size)))
            }

            Type::FixedArray(ty, size) => {
                let (base_addr, at) = if ty.is_dynamic() {
                    // For fixed arrays of types that are dynamic, we just jump
                    // to the offset location and decode from there.
                    let slice = bs.get(at..(at + 32)).ok_or_else(|| {
                        anyhow!("reached end of input while decoding {}[{}]", ty, size)
                    })?;
                    let offset = U256::from_big_endian(slice).as_usize();

                    (base_addr + offset, 0)
                } else {
                    // There's no need to change the addressing because fixed arrays
                    // will consume input by calling decode recursively and addressing
                    // will be computed correctly inside those calls.
                    (base_addr, at)
                };

                (0..(*size))
                    .try_fold((vec![], 0), |(mut values, total_consumed), _| {
                        let (value, consumed) =
                            Self::decode(bs, ty, base_addr, at + total_consumed)?;

                        values.push(value);

                        Ok((values, total_consumed + consumed))
                    })
                    .map(|(values, consumed)| {
                        let consumed = if ty.is_dynamic() { 32 } else { consumed };

                        (Value::FixedArray(values, *ty.clone()), consumed)
                    })
            }

            Type::String => {
                let (bytes_value, consumed) = Self::decode(bs, &Type::Bytes, base_addr, at)?;

                let bytes = if let Value::Bytes(bytes) = bytes_value {
                    bytes
                } else {
                    // should always be Value::Bytes
                    unreachable!();
                };

                let s = String::from_utf8(bytes)?;

                Ok((Value::String(s), consumed))
            }

            Type::Bytes => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bytes offset"))?;
                let offset = U256::from_big_endian(slice).as_usize();

                let at = base_addr + offset;

                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bytes length"))?;
                let bytes_len = U256::from_big_endian(slice).as_usize();

                let at = at + 32;
                let bytes = bs
                    .get(at..(at + bytes_len))
                    .ok_or_else(|| anyhow!("reached end of input while decoding bytes"))?
                    .to_vec();

                // consumes only the first 32 bytes, i.e. the offset pointer
                Ok((Value::Bytes(bytes), 32))
            }

            Type::Array(ty) => {
                let at = base_addr + at;
                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding array offset"))?;
                let offset = U256::from_big_endian(slice).as_usize();

                let at = base_addr + offset;
                let slice = bs
                    .get(at..(at + 32))
                    .ok_or_else(|| anyhow!("reached end of input while decoding array length"))?;
                let array_len = U256::from_big_endian(slice).as_usize();

                let at = at + 32;

                (0..array_len)
                    .try_fold((vec![], 0), |(mut values, total_consumed), _| {
                        let (value, consumed) = Self::decode(bs, ty, at, total_consumed)?;

                        values.push(value);

                        Ok((values, total_consumed + consumed))
                    })
                    .map(|(values, _)| (Value::Array(values, *ty.clone()), 32))
            }

            Type::Tuple(tys) => {
                // Tuples follow the same logic as fixed arrays.
                let (base_addr, at) = if ty.is_dynamic() {
                    let slice = bs.get(at..(at + 32)).ok_or_else(|| {
                        anyhow!("reached end of input while decoding tuple offset")
                    })?;
                    let offset = U256::from_big_endian(slice).as_usize();

                    (base_addr + offset, 0)
                } else {
                    (base_addr, at)
                };

                tys.iter()
                    .cloned()
                    .try_fold((vec![], 0), |(mut values, total_consumed), (name, ty)| {
                        let (value, consumed) =
                            Self::decode(bs, &ty, base_addr, at + total_consumed)?;

                        values.push((name, value));

                        Ok((values, total_consumed + consumed))
                    })
                    .map(|(values, consumed)| {
                        let consumed = if ty.is_dynamic() { 32 } else { consumed };

                        (Value::Tuple(values), consumed)
                    })
            }
        }
    }

    fn encode_bytes(buf: &mut Vec<u8>, bytes: &[u8], mut alloc_offset: usize) -> usize {
        let padded_bytes_len = Self::padded32_size(bytes.len());
        buf.resize(buf.len() + 32 + padded_bytes_len, 0);

        // write bytes size
        U256::from(bytes.len()).to_big_endian(&mut buf[alloc_offset..(alloc_offset + 32)]);
        alloc_offset += 32;

        // write bytes
        buf[alloc_offset..(alloc_offset + bytes.len())].copy_from_slice(bytes);

        alloc_offset + padded_bytes_len
    }

    // Computes the padded size for a given size, e.g.:
    // padded32_size(20) == 32
    // padded32_size(32) == 32
    // padded32_size(40) == 64
    fn padded32_size(size: usize) -> usize {
        let r = size % 32;

        if r == 0 {
            size
        } else {
            size + 32 - r
        }
    }
}
