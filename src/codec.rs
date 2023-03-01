use bytes::{buf::BufMutExt, BufMut, BytesMut};
use integer_encoding::VarInt;
use protobuf::{Message, ProtobufError};
use tokio_util::codec::{Decoder, Encoder};

use crate::messages::abci::*;

#[derive(Debug)]
pub struct ABCICodec {
    receive_buf: BytesMut,
}

impl ABCICodec {
    pub fn new() -> ABCICodec {
        ABCICodec {
            receive_buf: Default::default(),
        }
    }
}

impl Decoder for ABCICodec {
    type Item = Request;
    type Error = ProtobufError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Request>, ProtobufError> {
        if buf.is_empty() && self.receive_buf.is_empty() {
            return Ok(None);
        }
        self.receive_buf.extend_from_slice(&buf);
        buf.clear();
        let varint: (i64, usize) = i64::decode_var(&self.receive_buf[..]);
        if varint.0 <= 0 || (varint.0 as usize + varint.1) > self.receive_buf.len() {
            return Ok(None);
        }
        let request =
            Message::parse_from_bytes(&self.receive_buf[varint.1..(varint.0 as usize + varint.1)])?;
        let _ = self.receive_buf.split_to(varint.0 as usize + varint.1);
        Ok(Some(request))
    }
}

impl Encoder<Response> for ABCICodec {
    type Error = ProtobufError;

    fn encode(&mut self, msg: Response, buf: &mut BytesMut) -> Result<(), ProtobufError> {
        let msg_len = msg.compute_size();
        let varint = i64::encode_var_vec(i64::from(msg_len));

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
        echo.set_message(String::from("Helloworld"));
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
        let st = (0..2 * 4096).map(|_| "X").collect::<String>();
        echo.set_message(st);
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
        assert_eq!(v3.get_echo().get_message(), "Helloworld");
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
        echo.set_message(String::from("Helloworld"));
        r.set_echo(echo);

        let buf = &mut BytesMut::new();

        let v = codec.encode(r, buf);
        assert!(v.is_ok());
    }
}
