use anyhow::anyhow;
use base64::prelude::*;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

pub struct Auth {
    shared_secret: Bytes,
}

impl Auth {
    pub fn new(shared_secret: Bytes) -> Self {
        Self { shared_secret }
    }

    pub fn next(&self, time: SystemTime) -> anyhow::Result<String> {
        let time = time.duration_since(UNIX_EPOCH)?.as_secs();
        let time = time / 30;

        let mut time_buffer = vec![];
        time_buffer.write_u64::<BigEndian>(time)?;

        // Evaluate hash code for `tb` by key
        let mut mac = HmacSha1::new_from_slice(&self.shared_secret[..])?;
        mac.update(time_buffer.as_slice());
        let hashcode = mac.finalize().into_bytes();

        // last 4 bytes provide initial position. hashcode should have a length of 20.
        let start = (hashcode.get(19).ok_or(anyhow!("invalid sha1 hmac digest from time"))? & 0xf) as usize;

        if start >= 16 {
            return Err(anyhow!(""));
        }

        // extract 4 bytes at start and drop first bit
        let fc32 = (&hashcode[start..start + 4]).read_u32::<BigEndian>()?;
        let fc32 = (fc32 & ((1 << 31) - 1)) as usize;

        // range of possible chars for steam auth code.
        //noinspection SpellCheckingInspection
        const CODE_CHARS: &str = "23456789BCDFGHJKMNPQRTVWXY";

        /*
        the code generation process looks like this:
            code := make([]byte, 5)
            for i := range code {
                code[i] = chars[fullCode%charsLen]
                fullCode /= charsLen
            }

        repeatedly dividing 1/x n times is the same as 1/(x^n) at each step. CODE_CHARS will always
        have a length of 26, so we in reality the divisors will be:

        1, 26, 676, 17576, 456976
         */
        let chars = CODE_CHARS.as_bytes();
        let utf8 = vec![
            chars[fc32 % CODE_CHARS.len()],
            chars[(fc32 / 26) % CODE_CHARS.len()],
            chars[(fc32 / 676) % CODE_CHARS.len()],
            chars[(fc32 / 17576) % CODE_CHARS.len()],
            chars[(fc32 / 456976) % CODE_CHARS.len()],
        ];

        Ok(String::from_utf8(utf8)?)
    }
}

pub struct MobileConf {
    identity: Bytes,
}

#[derive(Debug)]
pub struct ConfirmationKey {
    unix_time: u64,
    tag: String,
    bytes: Box<[u8]>,
}

impl ConfirmationKey {
    pub fn unix_time(&self) -> u64 {
        self.unix_time
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn tag_owned(&self) -> String {
        self.tag.clone()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_string(&self) -> String {
        BASE64_STANDARD.encode(&self.bytes)
    }
}

fn truncate_str(s: &str, count: usize) -> &str {
    match s.char_indices().nth(count) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

impl MobileConf {
    pub fn new(identity: Bytes) -> Self {
        Self { identity }
    }

    pub fn next_getlist(&self, use_time: SystemTime) -> anyhow::Result<ConfirmationKey> {
        self.next(use_time, "getlist")
    }

    pub fn next_details(&self, use_time: SystemTime) -> anyhow::Result<ConfirmationKey> {
        self.next(use_time, "details")
    }

    pub fn next_accept(&self, use_time: SystemTime) -> anyhow::Result<ConfirmationKey> {
        self.next(use_time, "accept")
    }

    pub fn next_rject(&self, use_time: SystemTime) -> anyhow::Result<ConfirmationKey> {
        self.next(use_time, "rject")
    }

    fn next(&self, use_time: SystemTime, tag: &str) -> anyhow::Result<ConfirmationKey> {
        let unix_time = use_time.duration_since(UNIX_EPOCH)?.as_secs();

        const TIME_SIZE: usize = size_of::<u64>();
        let tag = truncate_str(tag, 32);
        let data_length = tag.len();

        let mut buffer = BytesMut::new();
        buffer.reserve(TIME_SIZE + data_length);

        let mut buffer = buffer.writer();
        buffer.write_u64::<BigEndian>(unix_time)?;

        let written = buffer.write(tag.as_bytes())?;
        if written != data_length {
            return Err(anyhow!("failed to write entire tag to key buffer"));
        }

        let mut mac = HmacSha1::new_from_slice(&self.identity[..])?;
        mac.update(buffer.get_ref());

        Ok(ConfirmationKey {
            unix_time: unix_time,
            tag: tag.to_string(),
            bytes: Box::from(mac.finalize().into_bytes().as_slice()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use chrono::{DateTime, TimeZone, Utc};

    fn mock_totp() -> Auth {
        const SECRET: &str = "cnOgv/KdpLoP6Nbh0GMkXkPXALQ=";
        let secret_bytes = BASE64_STANDARD.decode(SECRET).unwrap();
        Auth::new(secret_bytes.into())
    }

    fn mock_identity() -> MobileConf {
        const IDENTITY: &str = "cnOgv/KdpLoP6Nbh0GMkXkPXALQ=";
        let identity_bytes = BASE64_STANDARD.decode(IDENTITY).unwrap();
        MobileConf::new(identity_bytes.into())
    }

    fn mock_datetime() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2025, 1, 1, 12, 30, 15).earliest().unwrap()
    }

    #[test]
    fn generates_valid_auth_code() {
        let totp = mock_totp();

        let time = Utc.with_ymd_and_hms(2025, 1, 1, 12, 30, 15).earliest().unwrap();
        let code = totp.next(time.into()).unwrap();

        assert_eq!(code, "JQNVX")
    }

    #[test]
    fn generates_valid_confirmation_keys() {
        let identity = mock_identity();
        let time = mock_datetime();

        let tag_result_pairs = vec![
            ("conf", "qpt+vhKD/ujXR1TzQHoLHB/YiBM="),
            ("", "q/rk93BVLk5dJ7fjrUpn9RmYmg0="),
            ("abcdfghiabcdfghiabcdfghiabcdfghi", "QDI10zPUHxnb8kl0z5z8Xwy41AQ="),
        ];

        for (tag, expected) in tag_result_pairs {
            let code = identity.next(time.into(), tag).unwrap();
            let code = code.to_string();

            assert_eq!(code, expected, "expected {}, got {}, for tag {}", expected, code, tag);
        }
    }

    #[test]
    fn conf_keys_are_truncated() {
        let identity = mock_identity();
        let time = mock_datetime();

        let tag_result_pairs = vec![
            ("abcdfghiabcdfghiabcdfghiabcdfghi", "QDI10zPUHxnb8kl0z5z8Xwy41AQ="),
            ("abcdfghiabcdfghiabcdfghiabcdfghia", "QDI10zPUHxnb8kl0z5z8Xwy41AQ="),
            ("abcdfghiabcdfghiabcdfghiabcdfghiab", "QDI10zPUHxnb8kl0z5z8Xwy41AQ="),
        ];

        for (tag, expected) in tag_result_pairs {
            let code = identity.next(time.into(), tag).unwrap();
            let code = code.to_string();

            assert_eq!(code, expected, "expected {}, got {}, for tag {}", expected, code, tag);
        }
    }
}
