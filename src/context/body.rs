use bytes::BytesMut;

#[derive(Debug)]
pub struct Body {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    None,
    Bytes(Vec<u8>),
}

impl Body {
    pub fn empty() -> Self {
        Self { kind: Kind::None }
    }

    pub fn from(bytes: &BytesMut) -> Self {
        let data = bytes.to_vec();

        Self {
            kind: Kind::Bytes(data),
        }
    }

    pub fn as_bytes(&self, dest: &mut BytesMut) {
        match &self.kind {
            Kind::None => return,
            Kind::Bytes(xs) => dest.extend(xs.iter()),
        }
    }

    pub fn len(&self) -> usize {
        match &self.kind {
            Kind::None => 0,
            Kind::Bytes(xs) => xs.len(),
        }
    }

    pub fn bytes(&self, dest: &mut BytesMut) -> usize {
        match &self.kind {
            Kind::None => 0,
            Kind::Bytes(xs) => {
                dest.extend_from_slice(&xs);
                xs.len()
            }
        }
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self { kind: Kind::None }
    }
}

impl From<&'static str> for Body {
    fn from(s: &'static str) -> Self {
        Self {
            kind: Kind::Bytes(s.as_bytes().to_vec()),
        }
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self {
            kind: Kind::Bytes(s.as_bytes().to_vec()),
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
