use litesvm::LiteSVM;
use solana_pubkey::{pubkey, Pubkey};

pub fn load_dlmm_vault_program(svm: &mut LiteSVM) -> Pubkey {
    let program_bytes = include_bytes!("../../../../../target/deploy/dlmm_vault.so");
    let program_id = pubkey!("7Y1iiXP68seqhZtyQ1fEwxCYJVmJztwvXBBnZvRn3DyC");
    svm.add_program(program_id, program_bytes).unwrap();
    program_id
}
