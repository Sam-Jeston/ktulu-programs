use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{v0, AddressLookupTableAccount, Instruction, Message, VersionedMessage};
use solana_pubkey::Pubkey;
use solana_transaction::versioned::VersionedTransaction;

pub fn prepare_tx(
    svm: &mut LiteSVM,
    payer: &Pubkey,
    signers: &[&Keypair],
    instructions: &[Instruction],
) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(instructions, Some(payer), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap()
}

pub fn prepare_v0_tx(
    svm: &mut LiteSVM,
    payer: &Pubkey,
    signers: &[&Keypair],
    address_lookup_table_accounts: &[AddressLookupTableAccount],
    instructions: &[Instruction],
) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = v0::Message::try_compile(
        payer,
        instructions,
        address_lookup_table_accounts,
        blockhash,
    )
    .unwrap();
    VersionedTransaction::try_new(VersionedMessage::V0(msg), signers).unwrap()
}
