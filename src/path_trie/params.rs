use std::default::Default;

const SMALL: usize = 4;

#[derive(Debug)]
pub enum ParamMap<'a, 'b> {
    None,
    Small([Param<'a, 'b>; SMALL], usize),
    Large(Vec<Param<'a, 'b>>),
}

impl<'a, 'b> ParamMap<'a, 'b> {
    pub fn new() -> Self {
        Self::None
    }

    pub fn len(&self) -> usize {
        match self {
            ParamMap::None => 0,
            ParamMap::Small(_, len) => *len,
            ParamMap::Large(arr) => arr.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, k: &str) -> Option<&'b str> {
        let k = k.as_bytes();

        match self {
            ParamMap::None => None,
            ParamMap::Small(arr, len) => {
                arr.iter().take(*len).find(|e| e.key == k).map(Param::value)
            }
            ParamMap::Large(arr) => arr.iter().find(|e| e.key == k).map(Param::value),
        }
    }

    pub fn insert(&mut self, key: &'a [u8], value: &'b [u8]) {
        let param = Param::new(key, value);

        match self {
            ParamMap::None => {
                *self = ParamMap::Small(
                    [param, Param::default(), Param::default(), Param::default()],
                    1,
                );
            }
            ParamMap::Small(arr, len) => {
                if *len == SMALL {
                    let mut xs = Vec::with_capacity(*len);
                    xs.extend(arr.into_iter().map(std::mem::take));
                    *self = ParamMap::Large(xs);
                    return;
                }

                arr[*len] = Param::new(key, value);
                *len += 1;
            }
            ParamMap::Large(arr) => {
                let param = Param::new(key, value);
                arr.push(param);
            }
        }
    }
}

/*
#[derive(Debug)]
pub enum ParamIter<'a, 'b> {
    None,
    Small,
    Large,
}
*/

#[derive(Debug, Default)]
pub struct Param<'a, 'b> {
    key: &'a [u8],
    value: &'b [u8],
}

impl<'a, 'b> Param<'a, 'b> {
    pub fn new(key: &'a [u8], value: &'b [u8]) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.key) }
    }

    pub fn value(&self) -> &'b str {
        unsafe { std::str::from_utf8_unchecked(self.value) }
    }
}
