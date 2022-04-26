use bytes::BytesMut;

pub trait Codec<I>: Encoder<I> + Decoder {}

pub trait Encoder<I> {
    type Error;

    fn encode(&mut self, item: I, dest: &mut BytesMut) -> Result<(), Self::Error>;
}

pub trait Decoder {
    type Item;
    type Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>;
}
