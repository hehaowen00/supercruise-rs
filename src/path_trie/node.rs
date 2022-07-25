use super::params::Params;

#[derive(Debug, Eq, PartialEq)]
pub struct Node<T> {
    path: Vec<String>,
    data: Option<T>,
    index: Option<String>,
    children: Vec<Self>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PathNode {
    Static(String),
    Param(String),
    Wildcard,
}

const SMALL: usize = 4;

enum NodeChildren<T> {
    Small([Option<T>; SMALL], usize),
    Large(Vec<T>),
}

// index is used to find the position of nodes
// wildcards use * and params use :<id>
// a wildcard and param cannot exist in the same node logically
// only one of either

impl<T> Node<T> {
    #[inline]
    pub fn new(path: Vec<String>) -> Self {
        Self {
            path,
            data: None,
            index: None,
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: T) {
        let keys = parse_key(key).unwrap();

        if keys.len() == 0 {
            self.data = Some(value);
            return;
        }

        let mut index = self.index.get_or_insert_with(String::new);

        match &keys[0][0..1] {
            ":" => {
                if index.contains(':') {
                    panic!();
                }
                index.push(':');
            }
            "*" => {
                index.push('*');
                if index.contains('*') {
                    panic!();
                }
            }
            c => {
                index.push_str(c);
            }
        }

        if self.children.len() == 0 {
            let mut node = Node::new(keys);
            node.data = Some(value);
            self.children.push(node);
            return;
        }

        for child in &mut self.children {
            let path = &child.path;
        }
    }

    pub fn get<'a, 'b>(&self, key: &str) -> Option<&(T, Params<'a, 'b>)> {
        let keys = parse_key(key).unwrap();

        None
    }
}

impl PathNode {
    pub fn concat_static(&self, rhs: &Self) -> Result<Self, ()> {
        match (self, rhs) {
            (PathNode::Static(a), PathNode::Static(b)) => {
                Ok(PathNode::Static(format!("{}{}", a, b)))
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
enum PathParseError<'a> {
    InsufficientLength,
    UnexpectedToken(&'a str),
}

fn parse_key<'a>(key: &'a str) -> Result<Vec<String>, PathParseError<'a>> {
    let mut xs = Vec::new();

    let parts = key.split("/");

    let mut buf = String::new();
    let mut end = false;

    for p in parts.filter(|s| s != &"") {
        if end {
            return Err(PathParseError::UnexpectedToken(p));
        }

        if p == "" {
            continue;
        }

        match &p[0..1] {
            ":" => {
                if !buf.is_empty() {
                    xs.push(buf.clone());
                    buf.clear();
                }
                if p.len() == 1 {
                    return Err(PathParseError::InsufficientLength);
                }
                xs.push(p.to_string());
            }
            "*" => {
                if !buf.is_empty() {
                    xs.push(p.to_string());
                    buf.clear();
                }
                xs.push(p.to_string());
                end = true;
            }
            _ => {
                buf.push_str(p.trim_start_matches('/').trim_end_matches('/'));
                buf.push('/');
            }
        }
    }

    if !buf.is_empty() {
        xs.push(buf.to_string());
    }

    Ok(xs)
}

fn longest_subseq<'a>(lhs: &'a str, rhs: &'a str) -> &'a str {
    let min = std::cmp::min(lhs.len(), rhs.len());

    if min == 0 {
        return "";
    }

    for i in 0..min {
        if &lhs[i..i + 1] != &rhs[i..i + 1] {
            return &lhs[..i];
        }
    }

    return &lhs[..min];
}

fn compare_keys<'a, 'b>(params: &mut Params<'a, 'b>, a: &'a [String], b: &'b str) -> usize {
    let mut i = 0;

    for el in a {
        if &b[i..i + 1] == "/" {
            i = i + 1;
        }

        if i >= b.len() {
            return i;
        }

        match &el[0..1] {
            "*" => {
                return b.len();
            }
            ":" => {
                let r = find_substr(&b[i..], '/');
                if r == 0 {
                    params.insert(&el[1..], &b[i..b.len()]);
                    return b.len();
                }
                params.insert(&el[1..], &b[i..i + r]);
            }
            _ => {
                if (&b[i..]).starts_with(el) {
                    i = i + el.len();
                } else {
                    return i;
                }
            }
        }
    }

    return i;
}

fn find_substr(a: &str, ch: char) -> usize {
    for (i, c) in a.char_indices() {
        if ch == c {
            return i;
        }
    }
    return 0;
}

// #[test]
// fn test_subseq() {
//     let a = "/a/b/c/";
//     let b = "/a/b/d/";
//
//     let subseq = longest_subseq(a, b);
//     assert_eq!(subseq, "/a/b/");
// }
//
// #[test]
// fn test_parse() {
//     let p = "/api/hello/:name/:age/*";
//     println!("{:?}", parse_key(p));
//
//     let p = "/api/hello/*";
//     println!("{:?}", parse_key(p));
//
//     let p = "/api/hello/*/err";
//     println!("{:?}", parse_key(p));
// }
//
#[test]
fn test_node() {
    let mut node = Node::new(Vec::new());
    node.insert("/api/hello/:name", 1);
    node.insert("/api/hello/:name/:age", 2);
    // println!("{:#?}", node);

    let s = "/api/hello/:name/";
    let key = parse_key(s).unwrap();

    let t = "/api/hello/world/";

    let mut params = Params::new();
    let len = compare_keys(&mut params, &key, t);

    println!("len {}, {}", len, &t[..len]);
    println!("params {:?}", params);

    // test 2

    let s = "/api/hello/*";
    let key = parse_key(s).unwrap();

    let t = "/api/hello/world/";

    let mut params = Params::new();
    let len = compare_keys(&mut params, &key, t);

    println!("len {}, {}", len, &t[..len]);
    println!("params {:?}", params);
}
