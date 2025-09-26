use anchor_lang::solana_program::hash::hashv;
use base64::decode;

pub fn find_event(logs: &[String], event_type: &[u8]) -> Vec<u8> {
    let data_lines = logs
        .iter()
        .filter(|l| l.starts_with("Program data: "))
        .map(|l| l.trim_start_matches("Program data: "))
        .collect::<Vec<_>>();

    let mut body_vec = vec![];

    for data_line in data_lines {
        let raw = decode(data_line).expect("base64 decode");
        let (disc, body) = raw.split_at(8);

        let want_disc = &hashv(&[b"event:", event_type]).to_bytes()[..8];

        if disc == want_disc {
            body_vec.extend_from_slice(body);
        }
    }

    if body_vec.is_empty() {
        panic!("event not found");
    }

    body_vec
}
