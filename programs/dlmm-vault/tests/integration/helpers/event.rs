use anchor_lang::solana_program::hash::hashv;
use base64::decode;

pub fn find_event(logs: &[String], event_type: &[u8]) -> Vec<u8> {
    let data_line = logs
        .iter()
        .find(|l| l.starts_with("Program data: "))
        .expect("event not found")
        .trim_start_matches("Program data: ")
        .to_string();

    let raw = decode(data_line).expect("base64 decode");
    let (disc, body) = raw.split_at(8);

    let want_disc = &hashv(&[b"event:", event_type]).to_bytes()[..8];
    assert_eq!(disc, want_disc, "unexpected event type");

    body.to_vec()
}
