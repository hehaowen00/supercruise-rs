use crate::codec::{Decoder, Encoder};
use crate::context::Body;
use byteorder::{BigEndian, ReadBytesExt};
use bytes::{BufMut, BytesMut};
use rand::prelude::*;
use std::io::Cursor;

const FIN: u8 = 0x80;
const MASK: u8 = 0x80;
// const FRAME_LIMIT: usize = 65535;

#[derive(Debug)]
pub struct WsFrame {
    opcode: Opcode,
    pub masked: bool,
    data: BytesMut,
}

impl WsFrame {
    pub fn builder() -> WsFrameBuilder {
        WsFrameBuilder::new()
    }

    pub fn opcode(&self) -> &Opcode {
        &self.opcode
    }

    pub fn masked(&self) -> bool {
        self.masked
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn into_parts(self) -> (Opcode, bool, BytesMut) {
        (self.opcode, self.masked, self.data)
    }
}

pub struct WsFrameBuilder {
    masked: bool,
}

impl WsFrameBuilder {
    pub fn new() -> Self {
        Self { masked: false }
    }
}

impl WsFrameBuilder {
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }

    pub fn close(self) -> WsFrame {
        WsFrame {
            opcode: Opcode::CLOSE,
            masked: false,
            data: BytesMut::new(),
        }
    }

    pub fn continuation(self, fragment: impl Into<Body>) -> WsFrame {
        Self::body(Opcode::CONTINUATION, Some(fragment.into()))
    }

    pub fn binary(self, fragment: impl Into<Body>) -> WsFrame {
        Self::body(Opcode::BINARY, Some(fragment.into()))
    }

    pub fn text(self, fragment: impl Into<Body>) -> WsFrame {
        Self::body(Opcode::TEXT, Some(fragment.into()))
    }

    pub fn ping(self) -> WsFrame {
        Self::body(Opcode::PING, None)
    }

    pub fn pong(self) -> WsFrame {
        Self::body(Opcode::PONG, None)
    }

    fn body(opcode: Opcode, body: Option<Body>) -> WsFrame {
        let mut data = BytesMut::new();

        if let Some(xs) = body {
            xs.as_bytes(&mut data);
        }

        WsFrame {
            opcode,
            masked: false,
            data,
        }
    }
}

pub struct Ws {
    opcode: Option<Opcode>,
    length: usize,
    data: Option<BytesMut>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Opcode {
    CONTINUATION = 0x0,
    TEXT = 0x1,
    BINARY = 0x2,
    CLOSE = 0x8,
    PING = 0x9,
    PONG = 0xA,
}

impl From<u8> for Opcode {
    fn from(val: u8) -> Self {
        match val {
            0x0 => Opcode::CONTINUATION,
            0x1 => Opcode::TEXT,
            0x2 => Opcode::BINARY,
            0x8 => Opcode::CLOSE,
            0x9 => Opcode::PING,
            0xA => Opcode::PONG,
            _ => unreachable!(),
        }
    }
}

impl Ws {
    pub fn new() -> Self {
        Self {
            opcode: None,
            length: 0,
            data: None,
        }
    }
}

impl Decoder for Ws {
    type Item = WsFrame;
    type Error = ();

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 || src.len() < 6 {
            return Ok(None);
        }

        let mut curr = 0;

        let fin_opcode = src[curr];
        // println!("fin op {:#08b} {:#08b}", fin_opcode & 0x80, fin_opcode - 128);

        let fin = (fin_opcode & FIN) == 128;
        let opcode: Opcode = (fin_opcode & 0b0111_1111).into();

        if self.data.is_some() && opcode != Opcode::CONTINUATION {
            return Err(());
        }

        match opcode {
            Opcode::TEXT | Opcode::BINARY | Opcode::PING | Opcode::PONG | Opcode::CLOSE => {
                self.opcode = Some(opcode.clone());
            }
            _ => {}
        }

        // println!("fin {} opcode {:?}", fin, opcode);

        curr += 1;

        let mask_length = src[curr];
        curr += 1;

        let is_masked = (mask_length & MASK) == 128;
        let length = mask_length - 128;

        self.length = if length <= 125 {
            length as usize
        } else if length == 126 {
            let mut arr = Cursor::new(&src[curr..curr + 2]);
            curr += 2;
            arr.read_u16::<BigEndian>().unwrap() as usize
        } else if length == 127 {
            let mut arr = Cursor::new(&src[curr..curr + 8]);
            curr += 8;
            arr.read_u64::<BigEndian>().unwrap() as usize
        } else {
            return Err(());
        };

        // println!("masked {} length {}", is_masked, length);

        let mask_key = &src[curr..curr + 4];
        curr += 4;

        let mut decoded = match self.data.take() {
            Some(data) => data,
            None => BytesMut::new(),
        };

        if fin && src[curr..].len() != self.length {
            // println!("not enough bytes for complete message {}\n", src[curr..].len());

            return Ok(None);
        }

        if is_masked {
            for (i, b) in src[curr..].iter().enumerate() {
                decoded.put_u8(b ^ mask_key[i % 4]);
            }
        }

        // println!("message {:?}\n", unsafe { std::str::from_utf8_unchecked(&decoded) });
        // println!("{:?}", src);

        match fin {
            true => {
                let result = WsFrame {
                    opcode: self.opcode.take().unwrap(),
                    masked: is_masked,
                    data: decoded,
                };
                src.clear();
                Ok(Some(result))
            }
            false => {
                self.data = Some(decoded);
                src.clear();
                Ok(None)
            }
        }
    }
}

impl Encoder<WsFrame> for Ws {
    type Error = ();

    fn encode(&mut self, item: WsFrame, dest: &mut BytesMut) -> Result<(), Self::Error> {
        let mut rng = thread_rng();

        let mask_bit = if item.masked { MASK } else { 0 };

        match item.opcode {
            Opcode::BINARY | Opcode::TEXT => {
                let fin_opcode = FIN | item.opcode as u8;

                /*
                let mask_length = if item.data.len() <= 125 {
                    mask_bit | item.data.len() as u8
                } else {
                    126
                };
                */

                dest.put_u8(fin_opcode);
                // dest.put_u8(mask_length);

                mask_length(dest, mask_bit, item.data.len());

                let mut mask = [0; 4];

                if item.masked {
                    rng.fill_bytes(&mut mask);
                    dest.extend_from_slice(&mask);
                }

                let mut temp = item.data.clone();

                if item.masked {
                    for i in 0..temp.len() {
                        temp[i] = temp[i] ^ mask[i % 4];
                    }
                }

                dest.extend_from_slice(&temp);
            }
            Opcode::CLOSE | Opcode::PING | Opcode::PONG => {
                let fin_opcode = FIN | item.opcode as u8;

                let mask_length = 0;

                dest.put_u8(fin_opcode);
                dest.put_u8(mask_length);
            }
            _ => {}
        }

        Ok(())
    }
}

fn mask_length(dest: &mut BytesMut, mask_bit: u8, length: usize) {
    match length {
        x if x <= 125 => {
            dest.put_u8(mask_bit | length as u8);
        }
        x if x <= 65535 => {
            let len = x as u16;

            log::debug!("126 | len {}", len);

            dest.put_u8(mask_bit | 126);
            dest.extend_from_slice(&len.to_be_bytes());
        }
        x => {
            let len = x;

            log::debug!("127 | len {}", len);

            dest.put_u8(mask_bit | 127);
            dest.extend_from_slice(&len.to_be_bytes());
        }
    }
}
