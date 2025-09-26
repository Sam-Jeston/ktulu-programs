use std::path::PathBuf;

use litesvm::LiteSVM;
use solana_pubkey::{pubkey, Pubkey};

pub fn load_dlmm_vault_program(svm: &mut LiteSVM) -> Pubkey {
    // Attempt to read program path as
    let program_path = std::env::var("DLMM_VAULT_SO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/dlmm_vault.so")
        });

    let program_bytes = std::fs::read(&program_path).unwrap_or_else(|err| {
        panic!(
            "Failed to read dlmm_vault.so at '{}': {}. Set DLMM_VAULT_SO or build the program (anchor build / cargo build-bpf).",
            program_path.display(),
            err
        )
    });

    let program_id = pubkey!("7Y1iiXP68seqhZtyQ1fEwxCYJVmJztwvXBBnZvRn3DyC");
    svm.add_program(program_id, &program_bytes).unwrap();
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
