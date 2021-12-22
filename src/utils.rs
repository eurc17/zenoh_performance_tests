use super::common::*;

pub fn get_msg_payload(args_payload_size: usize) -> String {
    let mut msg_payload;
    if args_payload_size == 8 {
        msg_payload = format!("{:08}", 1 as usize);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == args_payload_size);
    } else if args_payload_size < 8 {
        warn!("Payload size cannot be less than 8 bytes, using 8 bytes for current test.");
        msg_payload = format!("{:08}", 1 as usize);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == 8);
    } else {
        msg_payload = format!("{:08}", 1 as usize);
        let additional_size = args_payload_size - 8;
        let mut postpend_string = String::from(".");
        for _ in 1..additional_size {
            postpend_string.push_str(".");
        }
        msg_payload.push_str(&postpend_string);
        let payload_size = std::mem::size_of_val(msg_payload.as_bytes());
        assert!(payload_size == args_payload_size);
    }
    msg_payload
}
