use risc0_zkvm::{default_prover, sha::Digest, ExecutorEnv, Receipt};

pub fn prove<T: serde::ser::Serialize>(input_vec: Vec<T>, elf: &[u8]) -> (u64, Receipt) {
    let mut builder = ExecutorEnv::builder();

    for input in input_vec {
        builder
            .write(&input)
            .unwrap();
    }

    let env = builder
        .build()
        .unwrap();

    let prover = default_prover();

    let receipt = prover.prove(env, elf).unwrap().receipt;

    let digest = receipt.journal.decode().unwrap();
    (digest, receipt)
}

pub fn verify(receipt: Receipt, image_id: impl Into<Digest>) {
    receipt
    .verify(image_id)
    .expect("receipt verification failed");
}
