pragma circom 2.1.5;

include "node_modules/circomlib/circuits/babyjub.circom";
include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/comparators.circom";

template Semaphore(MAX_MEMBERS) {
    // Input signals.
    // The input signals are all private except `members`, `message` and `scope`.
    // The secret is the scalar generated from the EdDSA private key.
    // Using the secret scalar instead of the private key allows this circuit
    // to skip steps 1, 2, 3 in the generation of the public key defined here:
    // https://www.rfc-editor.org/rfc/rfc8032#section-5.1.5, making the circuit
    // more efficient and simple.
    // See the Semaphore identity package to know more about how the identity is generated:
    // https://github.com/semaphore-protocol/semaphore/tree/main/packages/identity.
    signal input secret;
    signal input members[MAX_MEMBERS];
    signal input message;
    signal input scope;

    // Output signals.
    // The output signals are all public.
    // signal output merkleRoot, nullifier;
    signal output nullifier;

    // The secret scalar must be in the prime subgroup order 'l'.
    var l = 2736030358979909402780800718157159386076813972158567259200215660948447373041;

    component isLessThan = LessThan(251);
    isLessThan.in <== [secret, l];
    isLessThan.out === 1;

    // Identity generation.
    // The circuit derives the EdDSA public key from a secret using
    // Baby Jubjub (https://eips.ethereum.org/EIPS/eip-2494),
    // which is basically nothing more than a point with two coordinates.
    // It then calculates the hash of the public key, which is used
    // as the commitment, i.e. the public value of the Semaphore identity.
    var Ax, Ay;
    (Ax, Ay) = BabyPbk()(secret);

    var identityCommitment = Poseidon(2)([Ax, Ay]);

    // Inclusion proof.
    component isEqual[MAX_MEMBERS];
    for (var i = 0; i < MAX_MEMBERS; i++) {
        isEqual[i] = IsEqual();
        isEqual[i].in[0] <== identityCommitment;
        isEqual[i].in[1] <== members[i];
    }
    component orAll = MultiOR(MAX_MEMBERS);
    for (var i = 0; i < MAX_MEMBERS; i++) {
        orAll.in[i] <== isEqual[i].out;
    }
    orAll.out === 1;

    // Nullifier generation.
    // The nullifier is a value that essentially identifies the proof generated in a specific scope
    // and by a specific identity, so that externally anyone can check if another proof with the same
    // nullifier has already been generated. This mechanism can be particularly useful in cases
    // where one wants to prevent double-spending or double-voting, for example.
    nullifier <== Poseidon(2)([scope, secret]);

    // The message is not really used within the circuit.
    // The square applied to it is a way to force Circom's compiler to add a constraint and
    // prevent its value from being changed by an attacker.
    // More information here: https://geometry.xyz/notebook/groth16-malleability.
    signal dummySquare <== message * message;
}

template MultiOR(n) {
    signal input in[n];
    signal output out;

    signal sums[n];
    sums[0] <== in[0];
    for (var i = 1; i < n; i++) {
        sums[i] <== sums[i-1] + in[i];
    }

    component is_zero = IsZero();
    is_zero.in <== sums[n-1];
    out <== 1 - is_zero.out;
}

component main = Semaphore(50);
