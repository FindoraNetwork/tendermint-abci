use bytes::{buf::BufMutExt, BufMut, BytesMut};
use integer_encoding::VarInt;
use protobuf::{Error, Message};
use tokio_util::codec::{Decoder, Encoder};

use crate::messages::abci::*;

#[derive(Debug)]
pub struct ABCICodec;

impl ABCICodec {
    pub fn new() -> ABCICodec {
        ABCICodec
    }
}

impl Decoder for ABCICodec {
    type Item = Request;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Request>, Error> {
        let length = buf.len();
        if length == 0 {
            return Ok(None);
        }
        let varint: (i64, usize) = i64::decode_var(&buf[..]);
        if varint.0 as usize + varint.1 > length {
            return Ok(None);
        }
        let request = Message::parse_from_bytes(&buf[varint.1..(varint.0 as usize + varint.1)])?;
        let _ = buf.split_to(varint.0 as usize + varint.1);
        Ok(Some(request))
    }
}

impl Encoder<Response> for ABCICodec {
    type Error = Error;

    fn encode(&mut self, msg: Response, buf: &mut BytesMut) -> Result<(), Error> {
        let msg_len = msg.compute_size();
        let varint = i64::encode_var_vec(msg_len as i64);

        let remaining = buf.remaining_mut();
        let needed = msg_len as usize + varint.len();
        if remaining < needed {
            buf.reserve(needed);
        }

        buf.put(varint.as_ref());
        msg.write_to_writer(&mut buf.writer())?;
        trace!("Encode response! {:?}", &buf[..]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    fn setup_echo_request_buf() -> Result<BytesMut, Box<dyn Error>> {
        let mut buf = BytesMut::new();

        let mut r = Request::new();
        let mut echo = RequestEcho::new();
        echo.message = String::from("Helloworld");
        r.set_echo(echo);

        let msg_len = r.compute_size();
        let varint = i64::encode_var_vec(msg_len as i64);
        buf.put(varint.as_ref());
        r.write_to_writer(&mut (&mut buf).writer())?;

        trace!("Encode response! {:?}", &buf[..]);

        Ok(buf)
    }

    fn setup_echo_large_request_buf() -> Result<BytesMut, Box<dyn Error>> {
        let mut buf = BytesMut::new();

        let mut r = Request::new();
        let mut echo = RequestEcho::new();
        echo.message = (0..2 * 4096).map(|_| "X").collect::<String>();
        r.set_echo(echo);

        let msg_len = r.compute_size();
        let varint = i64::encode_var_vec(msg_len as i64);

        let remaining = buf.remaining_mut();
        let needed = msg_len as usize + varint.len();
        if remaining < needed {
            buf.reserve(needed);
        }

        buf.put(varint.as_ref());
        r.write_to_writer(&mut (&mut buf).writer())?;

        trace!("Encode response! {:?}", &buf[..]);

        Ok(buf)
    }

    #[test]
    fn should_decode() {
        let mut codec = ABCICodec::new();
        let mut buf = setup_echo_request_buf().unwrap();
        let r = codec.decode(&mut buf);
        assert!(r.is_ok());
        let v1 = r.ok();
        assert!(v1.is_some());
        let v2 = v1.unwrap();
        assert!(v2.is_some());
        let v3 = v2.unwrap();
        assert!(v3.has_echo());
        assert_eq!(v3.echo().message, String::from("Helloworld"));
    }

    #[test]
    fn should_decode_large_request() {
        let mut codec = ABCICodec::new();
        let mut buf = setup_echo_large_request_buf().unwrap();
        let r = codec.decode(&mut buf);
        assert!(r.is_ok());
        let v1 = r.ok();
        assert!(v1.is_some());
        let v2 = v1.unwrap();
        assert!(v2.is_some());
        let v3 = v2.unwrap();
        assert!(v3.has_echo());
    }

    #[test]
    fn should_encode() {
        let mut codec = ABCICodec::new();

        let mut r = Response::new();
        let mut echo = ResponseEcho::new();
        echo.message = String::from("Helloworld");
        r.set_echo(echo);

        let buf = &mut BytesMut::new();

        let v = codec.encode(r, buf);
        assert!(v.is_ok());
    }
}
