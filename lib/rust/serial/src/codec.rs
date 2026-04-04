use std::{io, marker::PhantomData};

use bytes::BufMut;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Default)]
pub(crate) struct RxCodec<Rx: prost::Message> {
    _phantom: PhantomData<Rx>,
}

impl<Rx: prost::Message + Default> Decoder for RxCodec<Rx> {
    type Item = Rx;

    type Error = io::Error;

    fn decode(&mut self, src: &mut prost::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let delimiter = src.as_ref().iter().position(|b| *b == 0u8);

        if let Some(n) = delimiter {
            let mut data = src.split_to(n + 1);

            if let Ok(len) = cobs::decode_in_place(&mut data) {
                if let Ok(msg) = Rx::decode(&data[..len]) {
                    tracing::trace!("decoded a valid message");
                    Ok(Some(msg))
                } else {
                    tracing::debug!("dropping message - invalid protobuf");
                    Ok(None)
                }
            } else {
                tracing::debug!("dropping message - invalid COBS");
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Default)]
pub(crate) struct TxCodec<Tx: prost::Message> {
    _phantom: PhantomData<Tx>,
}

impl<Tx: prost::Message + Default> Encoder<Tx> for TxCodec<Tx> {
    type Error = io::Error;

    fn encode(&mut self, item: Tx, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let proto = item.encode_to_vec();
        let mut cobs_buf = vec![0u8; cobs::max_encoding_length(proto.len())];
        let n = cobs::encode(&proto, &mut cobs_buf);
        dst.extend_from_slice(&cobs_buf[..n]);
        dst.put_u8(0u8); // cobs delimiter
        Ok(())
    }
}
