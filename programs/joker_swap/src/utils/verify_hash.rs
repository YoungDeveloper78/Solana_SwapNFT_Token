pub fn verify(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let computed_hash = process_proof(proof, leaf);
    computed_hash == root
}

fn process_proof(proof: Vec<[u8; 32]>, leaf: [u8; 32]) -> [u8; 32] {
    let mut computed_hash = leaf;
    for proof_element in proof.iter() {
        if computed_hash <= *proof_element {
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&computed_hash, proof_element]).0;
        } else {
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[proof_element, &computed_hash]).0;
        }
    }
    computed_hash
}
