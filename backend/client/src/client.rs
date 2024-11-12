use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomConfig};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_std::rand::thread_rng;
use dirs::home_dir;
use num::BigUint;
use shared_crypto::intent::Intent;
use std::vec;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{
    rpc_types::{
        SuiMoveValue, SuiObjectDataOptions, SuiObjectResponseQuery, SuiParsedData,
        SuiParsedMoveObject, SuiTransactionBlockResponseOptions,
    },
    types::{
        base_types::{ObjectID, ObjectRef, SuiAddress},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        quorum_driver_types::ExecuteTransactionRequestType,
        transaction::{Command, ObjectArg, ProgrammableTransaction, Transaction, TransactionData},
        Identifier,
    },
    SuiClient, SuiClientBuilder,
};
use tracing::info;

use crate::ClientHandlers;

pub struct Client {
    sui: SuiClient,
    package_id: ObjectID,
    group_id: ObjectID,
    keystore: FileBasedKeystore,
    address: SuiAddress,
}

impl Client {
    pub async fn new(package_id: String, group_id: String, keystore_path_relative: String) -> Self {
        let sui = SuiClientBuilder::default().build_devnet().await.unwrap();
        let package_id = package_id.parse().unwrap();
        let group_id = group_id.parse().unwrap();

        let mut keystore_path_absolute = home_dir().unwrap();
        keystore_path_absolute.push(keystore_path_relative);

        let keystore = FileBasedKeystore::new(&keystore_path_absolute.into()).unwrap();
        let address = keystore.addresses()[0];

        Self {
            sui,
            package_id,
            group_id,
            keystore,
            address,
        }
    }

    async fn get_gas_payment(&self) -> Vec<ObjectRef> {
        let owned_objects = self
            .sui
            .read_api()
            .get_owned_objects(
                self.address,
                Some(SuiObjectResponseQuery::new_with_options(
                    SuiObjectDataOptions::new().with_type(),
                )),
                None,
                None,
            )
            .await
            .unwrap();

        let coin = owned_objects
            .data
            .iter()
            .find(|obj| obj.data.as_ref().unwrap().is_gas_coin())
            .unwrap()
            .data
            .as_ref()
            .unwrap();

        vec![coin.object_ref()]
    }

    async fn submit_transaction(&self, pt: ProgrammableTransaction) {
        // Create transaction data.
        let tx_data = TransactionData::new_programmable(
            self.address,
            self.get_gas_payment().await,
            pt,
            5_000_000,
            self.sui.read_api().get_reference_gas_price().await.unwrap(),
        );

        // Sign the transaction.
        let signature = self
            .keystore
            .sign_secure(&self.address, &tx_data, Intent::sui_transaction())
            .unwrap();

        // Submit the transaction.
        let tx_receipt = self
            .sui
            .quorum_driver_api()
            .execute_transaction_block(
                Transaction::from_data(tx_data, vec![signature]),
                SuiTransactionBlockResponseOptions::full_content(),
                Some(ExecuteTransactionRequestType::WaitForLocalExecution),
            )
            .await
            .unwrap();

        info!("Transaction sent: {:?}", tx_receipt.digest);
    }

    async fn generate_groth16_proof(
        &self,
        secret_bytes: String,
        message_bytes: String,
        scope_bytes: String,
    ) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        // Load the WASM and R1CS for witness and proof generation
        let cfg = CircomConfig::<Fr>::new(
            "./circuit/main_js/semaphore.wasm",
            "./circuit/semaphore.r1cs",
        )
        .unwrap();
        let mut builder = CircomBuilder::new(cfg);

        // Prepare the inclusion proof input.

        let identity_commitments = self.get_identity_commitments().await;
        identity_commitments
            .iter()
            .for_each(|identity_commitment_bytes| {
                builder.push_input(
                    "members",
                    BigUint::parse_bytes(identity_commitment_bytes, 10).unwrap(),
                );
            });
        builder.push_input(
            "secret",
            BigUint::parse_bytes(&secret_bytes.as_bytes(), 10).unwrap(),
        );
        builder.push_input(
            "message",
            BigUint::parse_bytes(&message_bytes.as_bytes(), 10).unwrap(),
        );
        builder.push_input(
            "scope",
            BigUint::parse_bytes(&scope_bytes.as_bytes(), 10).unwrap(),
        );

        let circuit = builder.setup();

        // Generate a random proving key. WARNING: This is not secure. A proving key generated from a ceremony should be used in production.
        let mut rng = thread_rng();
        let pk =
            Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, &mut rng).unwrap();

        let circuit = builder.build().unwrap();
        let public_inputs = circuit.get_public_inputs().unwrap();

        // Generate proof.
        let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng).unwrap();

        // Verify the proof.
        let pvk = Groth16::<Bn254>::process_vk(&pk.vk).unwrap();
        let verified =
            Groth16::<Bn254>::verify_with_processed_vk(&pvk, &public_inputs, &proof).unwrap();
        assert!(verified);

        // Serialize verifying key, proof and public inputs.
        let mut verifying_key_bytes = Vec::new();
        let mut proof_bytes = Vec::new();
        let mut public_inputs_bytes = Vec::new();
        pk.vk
            .serialize_compressed(&mut verifying_key_bytes)
            .unwrap();
        proof.serialize_compressed(&mut proof_bytes).unwrap();
        public_inputs.iter().for_each(|input| {
            input
                .serialize_compressed(&mut public_inputs_bytes)
                .unwrap();
        });

        // Return verifying key, proof and public inputs.
        (verifying_key_bytes, proof_bytes, public_inputs_bytes)
    }

    async fn get_identity_commitments(&self) -> Vec<Vec<u8>> {
        let group = self
            .sui
            .read_api()
            .get_object_with_options(
                self.group_id,
                SuiObjectDataOptions {
                    show_type: false,
                    show_owner: false,
                    show_previous_transaction: false,
                    show_display: false,
                    show_content: true,
                    show_bcs: false,
                    show_storage_rebate: false,
                },
            )
            .await
            .unwrap();

        let members = match group.data.and_then(|d| d.content) {
            Some(SuiParsedData::MoveObject(SuiParsedMoveObject { fields, .. })) => {
                match fields.field_value("members").unwrap() {
                    SuiMoveValue::Vector(vec) => vec,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };

        let mut identity_commitments: Vec<Vec<u8>> = vec![b"0".to_vec(); 50];

        for (i, member) in members.iter().enumerate() {
            if let SuiMoveValue::Struct(s) = member {
                if let SuiMoveValue::Vector(vec) = s.field_value("identity_commitment").unwrap() {
                    vec.iter().for_each(|v| {
                        if let SuiMoveValue::Number(num) = v {
                            identity_commitments[i].push(*num as u8);
                        }
                    });
                }
            };
        }

        identity_commitments
    }
}

#[async_trait::async_trait]
impl ClientHandlers for Client {
    async fn add_member(&self, identity_commitment: String) {
        // Generate a Programmable Transaction Block.
        let mut ptb = ProgrammableTransactionBuilder::new();

        // Prepare inputs.
        let group_object = self
            .sui
            .read_api()
            .get_object_with_options(self.group_id, SuiObjectDataOptions::default())
            .await
            .unwrap()
            .data
            .unwrap();
        let group_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(group_object.object_ref()))
            .unwrap();
        let identity_commitment_input = ptb.pure(identity_commitment).unwrap();

        ptb.command(Command::move_call(
            self.package_id,
            Identifier::new("semaphore").unwrap(),
            Identifier::new("add_member").unwrap(),
            vec![],
            vec![group_input, identity_commitment_input],
        ));

        let pt = ptb.finish();

        self.submit_transaction(pt).await;
    }

    async fn add_answer(
        &self,
        secret_bytes: String,
        message_bytes: String,
        scope_bytes: String,
        question_id: String,
        answer: String,
    ) {
        // Generate a Semaphore Groth16 proof.
        let (verifying_key, proof_points, public_inputs) = self
            .generate_groth16_proof(secret_bytes, message_bytes, scope_bytes)
            .await;

        // Generate a Programmable Transaction Block.
        let mut ptb = ProgrammableTransactionBuilder::new();

        // Prepare inputs.
        let verifying_key_input = ptb.pure(verifying_key).unwrap();
        let proof_points_input = ptb.pure(proof_points).unwrap();
        let public_inputs_input = ptb.pure(public_inputs).unwrap();
        let question_object = self
            .sui
            .read_api()
            .get_object_with_options(
                question_id.parse().unwrap(),
                SuiObjectDataOptions::default(),
            )
            .await
            .unwrap()
            .data
            .unwrap();
        let question_input = ptb
            .obj(ObjectArg::ImmOrOwnedObject(question_object.object_ref()))
            .unwrap();
        let answer_input = ptb.pure(answer).unwrap();

        ptb.command(Command::move_call(
            self.package_id,
            Identifier::new("board").unwrap(),
            Identifier::new("add_answer").unwrap(),
            vec![],
            vec![
                verifying_key_input,
                proof_points_input,
                public_inputs_input,
                question_input,
                answer_input,
            ],
        ));

        let pt = ptb.finish();

        self.submit_transaction(pt).await;
    }
}
