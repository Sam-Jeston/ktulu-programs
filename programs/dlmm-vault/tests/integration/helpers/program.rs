use litesvm::LiteSVM;
use solana_pubkey::{pubkey, Pubkey};

pub fn load_dlmm_vault_program(svm: &mut LiteSVM) -> Pubkey {
    let program_bytes = include_bytes!("../../../../../target/deploy/dlmm_vault.so");
    let program_id = pubkey!("7Y1iiXP68seqhZtyQ1fEwxCYJVmJztwvXBBnZvRn3DyC");
    svm.add_program(program_id, program_bytes).unwrap();
    program_id
}

pub fn load_dlmm_program(svm: &mut LiteSVM) -> Pubkey {
    let program_bytes = include_bytes!("../../fixtures/dlmm.so");
    let program_id = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
    svm.add_program(program_id, program_bytes).unwrap();
    program_id
}

pub fn load_jupiter_program(svm: &mut LiteSVM) -> Pubkey {
    let program_bytes = include_bytes!("../../fixtures/jupiter.so");
    let program_id = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
    svm.add_program(program_id, program_bytes).unwrap();
    program_id
}

pub fn load_whirlpool_program(svm: &mut LiteSVM) -> Pubkey {
    let program_bytes = include_bytes!("../../fixtures/whirlpool.so");
    let program_id = pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
    svm.add_program(program_id, program_bytes).unwrap();
    program_id
}
