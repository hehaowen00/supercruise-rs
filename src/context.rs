use bytes::BytesMut;

#[derive(Debug)]
pub struct Body {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    None,
    Bytes(Vec<u8>),
    Stream(),
}

impl Body {
    pub fn as_bytes(&self, dest: &mut BytesMut) {
        match &self.kind {
            Kind::None => return,
            Kind::Bytes(xs) => dest.extend(xs.iter()),
            _ => return,
        }
    }

    pub fn len(&self) -> usize {
        match &self.kind {
            Kind::None => 0,
            Kind::Bytes(xs) => xs.len(),
            _ => unreachable!()
        }
    }

    pub fn bytes(&self, dest: &mut BytesMut) {
        match &self.kind {
            Kind::None => {
            }
            Kind::Bytes(xs) => {
                dest.extend_from_slice(&xs);
            }
            _ => unreachable!(),
        }
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self {
            kind: Kind::None,
        }
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self {
            kind: Kind::Bytes(s.as_bytes().to_vec())
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(xs: Vec<u8>) -> Self {
        Self {
            kind: Kind::Bytes(xs),
        }
    }
}

