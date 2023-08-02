use super::{dump_as_json, dump_data, dump_vk, from_json_file, serialize_instance, Proof};
use crate::io::serialize_fr_vec;
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use snark_verifier_sdk::encode_calldata;

const ACC_LEN: usize = 12;
const PI_LEN: usize = 32;

const ACC_BYTES: usize = ACC_LEN * 32;
const PI_BYTES: usize = PI_LEN * 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchProof {
    #[serde(flatten)]
    raw: Proof,
}

impl From<Proof> for BatchProof {
    fn from(proof: Proof) -> Self {
        let instances = proof.instances();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].len(), ACC_LEN + PI_LEN);

        let vk = proof.vk;
        let proof = proof
            .proof
            .into_iter()
            .chain(
                serialize_fr_vec(&instances[0][..ACC_LEN])
                    .into_iter()
                    .flatten(),
            )
            .collect();

        let instances = serialize_instance(&instances[0][ACC_LEN..]);

        Self {
            raw: Proof {
                proof,
                instances,
                vk,
            },
        }
    }
}

impl BatchProof {
    pub fn from_json_file(dir: &str, name: &str) -> Result<Self> {
        from_json_file(dir, &dump_filename(name))
    }

    pub fn dump(&self, dir: &str, name: &str) -> Result<()> {
        let filename = dump_filename(name);

        dump_data(dir, &format!("pi_{filename}.data"), &self.raw.instances);
        dump_data(dir, &format!("proof_{filename}.data"), &self.raw.proof);

        dump_vk(dir, &filename, &self.raw.vk);

        dump_as_json(dir, &filename, &self)
    }

    pub fn proof_to_verify(self) -> Proof {
        assert!(self.raw.proof.len() > ACC_BYTES);
        assert_eq!(self.raw.instances.len(), PI_BYTES);

        let proof_len = self.raw.proof.len() - ACC_BYTES;

        let mut proof = self.raw.proof;
        let mut instances = proof.split_off(proof_len);
        instances.extend(self.raw.instances);

        let vk = self.raw.vk;

        Proof {
            proof,
            instances,
            vk,
        }
    }

    // Only used for debugging.
    pub fn assert_calldata(&self) {
        let proof = self.clone().proof_to_verify();
        let expected_calldata = encode_calldata(&proof.instances(), &proof.proof);

        let mut result_calldata = self.raw.instances.clone();
        result_calldata.extend(self.raw.proof.clone());

        assert_eq!(result_calldata, expected_calldata);
    }
}

fn dump_filename(name: &str) -> String {
    format!("batch_{name}")
}
