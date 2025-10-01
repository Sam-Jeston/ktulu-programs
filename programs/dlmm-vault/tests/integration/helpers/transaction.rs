use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Instruction, Message, VersionedMessage};
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
