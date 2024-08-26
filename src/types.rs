use bls_signatures::{Serialize as BlsSerialize, Signature};
use serde::{Deserialize, Serialize};

// TODO: verify that the Keccak256 hashes are only 32 bytes.
//       they might have more bytes.
pub type TransactionHash = [u8; 32];
pub type BlockHash = [u8; 32];

#[derive(Clone)]
pub struct QuirkleSignature {
    pub bls_signature: Signature,
}

impl std::fmt::Debug for QuirkleSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &self
                .bls_signature
                .as_bytes()
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        )
    }
}

// TODO: privatize members and use as_bytes() method instead
#[derive(Clone)]
pub struct QuirkleRoot {
    pub bytes: [u8; 32],
}

impl std::fmt::Debug for QuirkleRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &self
                .bytes
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        )
    }
}

// TODO: privatize members and use as_bytes() method instead
#[derive(Clone)]
pub struct ECDSASignature {
    pub bytes: [u8; 65],
}

impl std::fmt::Debug for ECDSASignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            &self
                .bytes
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuirkleProof {
    pub quirkle_root: QuirkleRoot,
    pub member_address: String,
    pub expires_at: u64,
    pub signature: QuirkleSignature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub signature: ECDSASignature,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "name")]
pub enum Event {
    CreateQuirkle {
        // TODO(QUI-20): this should be a vector of Keccak256 hashes
        members: Vec<String>,
        proof_ttl: u64,
    },
}

impl Serialize for QuirkleRoot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hash = format!(
            "0x{}",
            &self
                .bytes
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        );
        serializer.serialize_str(&hash)
    }
}

impl<'de> Deserialize<'de> for QuirkleRoot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        let byte_vec: Vec<u8> = (2..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect::<Result<Vec<u8>, std::num::ParseIntError>>()
            .map_err(|e| serde::de::Error::custom(e))?;

        let byte_array: [u8; 32] = byte_vec.try_into().unwrap();

        Ok(QuirkleRoot { bytes: byte_array })
    }
}

impl Serialize for QuirkleSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = &self.bls_signature.as_bytes();
        let hash = format!(
            "{}",
            bytes
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        );
        serializer.serialize_str(&hash)
    }
}

impl<'de> Deserialize<'de> for QuirkleSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        let byte_vec: Vec<u8> = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect::<Result<Vec<u8>, std::num::ParseIntError>>()
            .map_err(|e| serde::de::Error::custom(e))?;

        let byte_array: [u8; 96] = byte_vec.try_into().unwrap();

        let g2_affine = bls12_381::G2Affine::from_compressed(&byte_array).unwrap();

        Ok(QuirkleSignature {
            bls_signature: Signature::from(g2_affine),
        })
    }
}

impl Serialize for ECDSASignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = &self.bytes;
        let hash = format!(
            "{}",
            bytes
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        );
        serializer.serialize_str(&hash)
    }
}

impl<'de> Deserialize<'de> for ECDSASignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        let byte_vec: Vec<u8> = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect::<Result<Vec<u8>, std::num::ParseIntError>>()
            .map_err(|e| serde::de::Error::custom(e))?;

        let bytes: [u8; 65] = byte_vec.try_into().unwrap();

        Ok(ECDSASignature { bytes })
    }
}
