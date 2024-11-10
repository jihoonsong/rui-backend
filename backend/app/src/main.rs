use ark_bn254::{Bn254, Fr};
use ark_circom::{CircomBuilder, CircomConfig};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_std::rand::thread_rng;
use num::BigUint;
use std::vec;

#[tokio::main]
async fn main() {
    // Load the WASM and R1CS for witness and proof generation
    let cfg = CircomConfig::<Fr>::new(
        "./circuit/main_js/semaphore.wasm",
        "./circuit/semaphore.r1cs",
    )
    .unwrap();
    let mut builder = CircomBuilder::new(cfg);

    // Private input.
    let secret_bytes =
        b"296112461635921789684155629793707096438749297915627185464238387744996954561";
    let mut identity_commitment_bytes_vec: Vec<Vec<u8>> = vec![b"0".to_vec(); 50];
    identity_commitment_bytes_vec[0] =
        b"11237622825477336339577122413451117718539783476837539122310492284566644730311".to_vec();
    identity_commitment_bytes_vec[1] =
        b"9332663527862709610616009715800254142772436825222910251631161087138559093425".to_vec();
    identity_commitment_bytes_vec[2] =
        b"17571968525740531145908143522134746202008892770988337169171069637989921082428".to_vec();
    identity_commitment_bytes_vec[3] =
        b"13255821893820536903335282929376140649646180444238593676033702344407594536519".to_vec();
    let message_bytes =
        b"312829776796408387545637016147278514583116203736587368460269838669765409292";
    let scope_bytes =
        b"377198358630688860307588270710436263233488522579783182156144235454763907569";

    println!(
        "identity_commitment_bytes_vec[0]: {}",
        hex::encode(&identity_commitment_bytes_vec[0])
    );

    // Private inputs: A factorisation of a number
    builder.push_input("secret", BigUint::parse_bytes(secret_bytes, 10).unwrap());
    identity_commitment_bytes_vec
        .iter()
        .for_each(|identity_commitment_bytes| {
            builder.push_input(
                "members",
                BigUint::parse_bytes(identity_commitment_bytes, 10).unwrap(),
            );
        });
    builder.push_input("message", BigUint::parse_bytes(message_bytes, 10).unwrap());
    builder.push_input("scope", BigUint::parse_bytes(scope_bytes, 10).unwrap());

    let circuit = builder.setup();

    // Generate a random proving key. WARNING: This is not secure. A proving key generated from a ceremony should be used in production.
    let mut rng = thread_rng();
    let pk =
        Groth16::<Bn254>::generate_random_parameters_with_reduction(circuit, &mut rng).unwrap();

    let circuit = builder.build().unwrap();
    let public_inputs = circuit.get_public_inputs().unwrap();

    // Create proof
    let proof = Groth16::<Bn254>::prove(&pk, circuit, &mut rng).unwrap();

    // Verify proof
    let pvk = Groth16::<Bn254>::process_vk(&pk.vk).unwrap();
    let verified =
        Groth16::<Bn254>::verify_with_processed_vk(&pvk, &public_inputs, &proof).unwrap();
    assert!(verified);

    // Print verifying key
    let mut pk_bytes = Vec::new();
    pk.vk.serialize_compressed(&mut pk_bytes).unwrap();
    println!("Verifying key: {}", hex::encode(pk_bytes));

    // Print proof
    let mut proof_serialized = Vec::new();
    proof.serialize_compressed(&mut proof_serialized).unwrap();
    println!("Proof: {}", hex::encode(proof_serialized));

    // Print public inputs. Note that they are concatenated.
    let mut public_inputs_serialized = Vec::new();
    public_inputs.iter().for_each(|input| {
        input
            .serialize_compressed(&mut public_inputs_serialized)
            .unwrap();
    });
    println!("Public inputs: {}", hex::encode(public_inputs_serialized));
}
