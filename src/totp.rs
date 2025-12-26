use anyhow::anyhow;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::BytesMut;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

pub struct State<'a> {
    shared_secret: &'a [u8],
}

impl<'a> State<'a> {
    pub fn new(secret: &'a [u8]) -> Self {
        Self { shared_secret: secret }
    }

    pub fn next(&self, time: SystemTime) -> anyhow::Result<String> {
        let time = time.duration_since(UNIX_EPOCH)?.as_secs();
        let time = time / 30;

        let mut time_buffer = vec![];
        time_buffer.write_u64::<BigEndian>(time)?;

        // Evaluate hash code for `tb` by key
        let mut mac = HmacSha1::new_from_slice(self.shared_secret)?;
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

pub struct Confirm<'a> {
    identity: &'a [u8],
}

impl<'a> Confirm<'a> {
    pub fn new(secret: &'a [u8]) -> Self {
        Self { identity: secret }
    }

    pub fn generate_key(&self, use_time: SystemTime, tag: &[u8]) -> anyhow::Result<Vec<u8>> {
        let unix_time = use_time.duration_since(UNIX_EPOCH)?.as_secs();

        const TIME_SIZE: usize = size_of::<u64>();
        let data_length: usize = tag.len().min(32);

        let mut buffer = BytesMut::with_capacity(TIME_SIZE + data_length);
        (&mut buffer[0..TIME_SIZE]).write_u64::<BigEndian>(unix_time)?;

        let written = (&mut buffer[TIME_SIZE..]).write(tag)?;
        if written != data_length {
            return Err(anyhow!("failed to write entire tag to key buffer"));
        }

        let mut mac = HmacSha1::new_from_slice(self.identity)?;
        mac.update(&buffer);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use chrono::{TimeZone, Utc};

    #[test]
    fn generates_valid_auth_code() {
        const SECRET: &str = "cnOgv/KdpLoP6Nbh0GMkXkPXALQ=";
        let secret_bytes = BASE64_STANDARD.decode(SECRET).unwrap();
        let state = State::new(secret_bytes.as_slice());

        let time = Utc.with_ymd_and_hms(2025, 1, 1, 12, 30, 15).earliest().unwrap();
        let code = state.next(time.into()).unwrap();

        assert_eq!(code, "JQNVX")
    }
}
