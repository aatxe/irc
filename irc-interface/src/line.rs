//! Abstract structure for line-based codecs.
//! Most codecs will be based on parsing messages line-by-line. This trait simplifies implementing such codecs.
//! Instead of having to implement the [`Decoder`] and [`Encoder`] traits manually, it suffices to implement [`LineCodec`].

use crate::{
    Decoder, Encoder, InternalIrcMessageIncoming, InternalIrcMessageOutgoing, MessageCodec,
};
use bytes::BytesMut;
use encoding::{label::encoding_from_whatwg_label, DecoderTrap, EncoderTrap, EncodingRef};
use std::{
    fmt::{Debug, Display},
    io,
    str::FromStr,
};

/// Split received data into lines using [`LineCodecInner`] and and then run parsing functions as defined in [`LineCodec`]'s [`Encoder`] and [`Decoder`] implementations.
/// They refer the message to [`Display::to_string`] and [`FromStr::try_from`] methods.
pub struct LineCodec<Msg> {
    inner: LineSplitter,
    _phantom: std::marker::PhantomData<Msg>,
}

/// Splits received data into lines, each of which are encoded as [`String`].
pub struct LineSplitter {
    encoding: EncodingRef,
    next_index: usize,
}

/// A message that can be parsed using a line parser.
pub trait LineMessage:
    Display
    + FromStr
    + InternalIrcMessageIncoming
    + InternalIrcMessageOutgoing
    + Debug
    + Clone
    + Unpin
    + Sized
{
    type Error: From<io::Error> + From<<Self as FromStr>::Err> + Debug;
}

impl<Msg> MessageCodec for LineCodec<Msg>
where
    Msg: LineMessage,
{
    type MsgItem = Msg;
    type Error = <Msg as LineMessage>::Error;

    /// Creates a new instance of IrcCodec wrapping a LineCodec with the specific encoding.
    fn try_new(label: impl AsRef<str>) -> Result<Self, <Self as MessageCodec>::Error> {
        Ok(LineSplitter::try_new(label).map(|codec| Self {
            inner: codec,
            _phantom: std::marker::PhantomData,
        })?)
    }
}

impl<Msg> Debug for LineCodec<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LineCodec")
    }
}

impl<Msg> Encoder<Msg> for LineCodec<Msg>
where
    Msg: LineMessage,
{
    type Error = <Msg as LineMessage>::Error;

    fn encode(
        &mut self,
        msg: Msg,
        dst: &mut BytesMut,
    ) -> Result<(), <Self as Encoder<Msg>>::Error> {
        Ok(self
            .inner
            .encode(<Self as MessageCodec>::sanitize(msg.to_string()), dst)?)
    }
}

impl<Msg> Decoder for LineCodec<Msg>
where
    Msg: LineMessage,
{
    type Error = <Msg as LineMessage>::Error;
    type Item = Msg;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Msg>, <Self as Decoder>::Error> {
        if let Some(msg) = self.inner.decode(src)? {
            Ok(Some(msg.parse::<Msg>()?))
        } else {
            Ok(None)
        }
    }
}

impl LineSplitter {
    /// Creates a new instance from the specified encoding.
    fn try_new(label: impl AsRef<str>) -> Result<Self, io::Error> {
        encoding_from_whatwg_label(label.as_ref())
            .map(|enc| LineSplitter {
                encoding: enc,
                next_index: 0,
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Attempted to use unknown codec {}.", label.as_ref())[..],
                )
                .into()
            })
    }
}

impl Encoder<String> for LineSplitter {
    type Error = io::Error;

    fn encode(
        &mut self,
        msg: String,
        dst: &mut BytesMut,
    ) -> Result<(), <Self as Encoder<String>>::Error> {
        // Encode the message using the codec's encoding.
        let data: Result<Vec<u8>, io::Error> = self
            .encoding
            .encode(&msg, EncoderTrap::Replace)
            .map_err(|data| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Failed to encode {} as {}.", data, self.encoding.name())[..],
                )
            });

        // Write the encoded message to the output buffer.
        dst.extend(&data?);

        Ok(())
    }
}

impl Decoder for LineSplitter {
    type Item = String;
    type Error = io::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<<Self as Decoder>::Item>, <Self as Decoder>::Error> {
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
                )),
            }
        } else {
            // Set the search start index to the current length since we know that none of the
            // characters we've already looked at are newlines.
            self.next_index = src.len();
            Ok(None)
        }
    }
}
