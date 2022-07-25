const SMALL: usize = 4;

#[derive(Debug)]
pub enum Params<'a, 'b> {
    Small([Param<'a, 'b>; SMALL], usize),
    Large(Vec<Param<'a, 'b>>),
}

impl<'a, 'b> Params<'a, 'b> {
    pub fn new() -> Self {
        Self::Small([Default::default(); 4], 0)
    }

    pub fn insert(&mut self, key: &'a str, value: &'b str) {
        match self {
            Self::Small(params, count) => {
                if *count == SMALL {
                    let mut xs = Vec::with_capacity(SMALL + 1);
                    xs.extend_from_slice(params);
                    xs.push(Param::new(key, value));

                    *self = Self::Large(xs);
                    return;
                }

                params[*count] = Param::new(key, value);
                *count += 1;
            }
            Self::Large(params) => {
                params.push(Param::new(key, value));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&'b str> {
        match self {
            Self::Small(params, count) => {
                for i in 0..*count {
                    if params[i].key == key {
                        return Some(params[i].value);
                    }
                }
            }
            Self::Large(params) => {
                for i in params {
                    if i.key == key {
                        return Some(i.value);
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Param<'a, 'b> {
    key: &'a str,
    value: &'b str,
}

impl<'a, 'b> Param<'a, 'b> {
    #[inline]
    pub fn new(key: &'a str, value: &'b str) -> Self {
        Self { key, value }
    }

    #[inline]
    pub fn key(&self) -> &'a str {
        self.key
    }

    #[inline]
    pub fn value(&self) -> &'b str {
        self.value
    }
}
// const SMALL: usize = 4;
//
// #[derive(Debug)]
// pub enum Params<'a, 'b> {
//     Small {
//         keys: [&'a str; SMALL],
//         values: [&'b str; SMALL],
//         count: usize,
//     },
//     Large(Vec<Param<'a, 'b>>),
// }
//
// impl<'a, 'b> Params<'a, 'b> {
//     pub fn new() -> Self {
//         Self::Small {
//             keys: [Default::default(); SMALL],
//             values: [Default::default(); SMALL],
//             count: 0,
//         }
//     }
//
//     pub fn insert(&mut self, key: &'a str, value: &'b str) {
//         match self {
//             Self::Small {
//                 keys,
//                 values,
//                 count,
//             } => {
//                 if *count == SMALL {
//                     let mut xs = Vec::with_capacity(SMALL + 1);
//                     for i in 0..SMALL {
//                         xs.push(Param::new(keys[i], values[i]));
//                     }
//                     xs.push(Param::new(key, value));
//
//                     *self = Self::Large(xs);
//                     return;
//                 }
//
//                 keys[*count] = key;
//                 values[*count] = value;
//                 *count += 1;
//             }
//             Self::Large(params) => {
//                 params.push(Param::new(key, value));
//             }
//         }
//     }
//
//     pub fn get(&self, key: &str) -> Option<&'b str> {
//         match self {
//             Self::Small {
//                 keys,
//                 values,
//                 count,
//             } => {
//                 for i in 0..*count {
//                     if keys[i] == key {
//                         return Some(values[i]);
//                     }
//                 }
//             }
//             Self::Large(params) => {
//                 for i in params {
//                     if i.key == key {
//                         return Some(i.value);
//                     }
//                 }
//             }
//         }
//         None
//     }
// }
//
// #[derive(Debug, Clone, Copy, Default)]
// pub struct Param<'a, 'b> {
//     key: &'a str,
//     value: &'b str,
// }
//
// impl<'a, 'b> Param<'a, 'b> {
//     #[inline]
//     pub fn new(key: &'a str, value: &'b str) -> Self {
//         Self { key, value }
//     }
//
//     #[inline]
//     pub fn key(&self) -> &'a str {
//         self.key
//     }
//
//     #[inline]
//     pub fn value(&self) -> &'b str {
//         self.value
//     }
// }
