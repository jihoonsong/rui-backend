use dirs::home_dir;
use shared_crypto::intent::Intent;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{
    rpc_types::{SuiObjectDataOptions, SuiObjectResponseQuery, SuiTransactionBlockResponseOptions},
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

    async fn add_answer(&self, question_id: String, answer: String) {
        // Generate a Programmable Transaction Block.
        let mut ptb = ProgrammableTransactionBuilder::new();

        // Prepare inputs.
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
            vec![question_input, answer_input],
        ));

        let pt = ptb.finish();

        self.submit_transaction(pt).await;
    }
}
