//! Implementation of line-delimiting codec for Tokio.

use std::io;

use bytes::BytesMut;
use encoding::label::encoding_from_whatwg_label;
use encoding::{DecoderTrap, EncoderTrap, EncodingRef};
use irc_interface::{Decoder, Encoder};

use crate::error;

/// A line-based codec parameterized by an encoding.
pub struct LineCodec {
    encoding: EncodingRef,
    next_index: usize,
}

impl LineCodec {
    /// Creates a new instance of LineCodec from the specified encoding.
    pub fn new(label: &str) -> error::Result<LineCodec> {
        encoding_from_whatwg_label(label)
            .map(|enc| LineCodec {
                encoding: enc,
                next_index: 0,
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Attempted to use unknown codec {}.", label)[..],
                )
                .into()
            })
    }
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = error::ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> error::Result<Option<String>> {
        if let Some(offset) = src[self.next_index..].iter().position(|b| *b == b'\n') {
            // Remove the next frame from the buffer.
            let line = src.split_to(self.next_index + offset + 1);

            // Set the search start index back to 0 since we found a newline.
            self.next_index = 0;

            // Decode the line using the codec's encoding.
            match self.encoding.decode(line.as_ref(), DecoderTrap::Replace) {
                Ok(data) => Ok(Some(data)),
                Err(data) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Failed to decode {} as {}.", data, self.encoding.name())[..],
                )
                .into()),
            }
        } else {
            // Set the search start index to the current length since we know that none of the
            // characters we've already looked at are newlines.
            self.next_index = src.len();
            Ok(None)
        }
    }
}

impl Encoder<String> for LineCodec {
    type Error = error::ProtocolError;

    fn encode(&mut self, msg: String, dst: &mut BytesMut) -> error::Result<()> {
        // Encode the message using the codec's encoding.
        let data: error::Result<Vec<u8>> = self
            .encoding
            .encode(&msg, EncoderTrap::Replace)
            .map_err(|data| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Failed to encode {} as {}.", data, self.encoding.name())[..],
                )
                .into()
            });

        // Write the encoded message to the output buffer.
        dst.extend(&data?);

        Ok(())
    }
}
