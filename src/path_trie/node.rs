use crate::common;

use super::params::Params;

#[derive(Debug, Eq, PartialEq)]
pub struct Node<T> {
    path: Vec<String>,
    data: Option<T>,
    children: Vec<Self>,
}

impl<T> Node<T> {
    #[inline]
    pub fn new(path: Vec<String>) -> Self {
        Self {
            path,
            data: None,
            children: Vec::new(),
        }
    }

    // pub fn insert(&mut self, keys: &[String], value: T) {
    //     println!("keys {:?}", keys);
    //
    //     if keys.len() == 0 {
    //         self.data = Some(value);
    //         return;
    //     }
    //
    //     if self.children.len() == 0 {
    //         let mut node = Node::new(keys.to_vec());
    //         node.data = Some(value);
    //         self.children.push(node);
    //         return;
    //     }
    //
    //     let nodes = &mut self.children;
    //     let mut index = 0;
    //
    //     loop {
    //         if index >= nodes.len() {
    //             break;
    //         }
    //
    //         let lhs = &nodes[index].path;
    //
    //         let cs = common_str(&lhs[0], &keys[0]);
    //         println!("cs {:?}", cs);
    //
    //         if cs == "" {
    //             index = index + 1;
    //             continue;
    //         }
    //
    //         println!("t {:?} {:?}", cs, lhs[0]);
    //
    //         if cs == &lhs[0] {
    //             // println!("count {:?} {:?} {:?}", count, &lhs, &keys);
    //
    //             if cs.len() < keys[0].len() {
    //                 let mut pk = keys.to_vec().clone();
    //                 pk[0] = pk[0][cs.len()..].to_string();
    //                 println!("{:?}", pk);
    //                 nodes[index].insert(&pk, value);
    //                 break;
    //             }
    //             let count = compare_keys(&lhs, &keys);
    //             if count == lhs.len() && count < keys.len() {
    //                 nodes[index].insert(&keys[count..], value);
    //             }
    //         } else {
    //             let mut root = Node::new(vec![cs.to_string()]);
    //             let len = cs.len();
    //
    //             let mut old = nodes.remove(index);
    //
    //             let mut s = old.path.remove(0);
    //             s = s[len..].to_string();
    //
    //             old.path.insert(0, s);
    //
    //             let mut new = Node::new(vec![keys[0][len..].to_string()]);
    //             new.path.extend_from_slice(&keys[1..]);
    //             new.data = Some(value);
    //
    //             root.children.push(old);
    //             root.children.push(new);
    //             root.children.sort_by_key(|e| e.path.clone());
    //
    //             nodes.push(root);
    //         }
    //
    //         break;
    //         // index = index + 1;
    //     }
    // }

    pub fn insert(&mut self, keys: &[String], value: T) {
        let mut nodes = &mut self.children;
        let mut rem = keys.to_vec();

        'outer: loop {
            if rem.len() == 0 {
                self.data = Some(value);
                return;
            }

            if nodes.len() == 0 {
                let mut node = Node::new(rem.to_vec());
                node.data = Some(value);
                nodes.push(node);
                return;
            }

            'inner: for node in nodes.iter_mut() {
                let lhs = node.path.clone();
                let cs = common_str(&lhs[0], &rem[0]);

                if cs == "" {
                    continue;
                }

                if cs == ":" {
                    let mut n = Node::new(rem.to_vec());
                    n.data = Some(value);
                    let old = std::mem::replace(node, n);

                    break 'outer;
                }

                if cs == &lhs[0] {
                    if cs.len() < rem[0].len() {
                        let s = rem.remove(0);
                        rem.insert(0, s[cs.len()..].to_string());
                        nodes = &mut node.children;
                        continue 'outer;
                    }

                    let count = compare_keys(&lhs, &rem);

                    if count < lhs.len() {
                        let mut new = Node::new(rem);
                        new.data = Some(value);

                        let mut old = std::mem::replace(node, new);
                        old.path.remove(0);

                        node.children.push(old);
                        break 'outer;
                    }

                    if count < rem.len() {
                        nodes = &mut node.children;
                        rem = rem[count..].to_vec();
                        continue 'outer;
                    }

                    if count == lhs.len() && count == rem.len() {
                        node.data = Some(value);
                        break 'outer;
                    }
                } else {
                    let root = Node::new(vec![cs.to_string()]);

                    let mut old = std::mem::replace(node, root);

                    let mut s = old.path.remove(0);
                    s = s[cs.len()..].to_string();
                    old.path.insert(0, s);

                    let mut new = Node::new(vec![rem[0][cs.len()..].to_string()]);
                    new.path.extend_from_slice(&rem[1..]);
                    new.data = Some(value);

                    node.children.push(old);
                    node.children.push(new);
                    node.children.sort_by_key(|e| e.path.to_owned());

                    break 'inner;
                }
            }

            // cannot borrow as mutable
            // nodes.push(Node::new(rem.to_vec()));

            break 'outer;
        }
    }

    pub fn get<'a, 'b>(&'a self, key: &'b str) -> Option<(T, Params<'a, 'b>)> {
        let mut params = Params::new();

        let s = until(&key[1..], "/");
        println!("{:?}", s);

        None
    }
}

fn common_str<'a, 'b>(a: &'a str, b: &'b str) -> &'a str {
    let min = std::cmp::min(a.len(), b.len());
    for i in 0..min {
        if a[i..i + 1] != b[i..i + 1] {
            return &a[..i];
        }
    }
    &a[..min]
}

fn until<'a, 'b>(a: &'a str, b: &'b str) -> &'a str {
    let mut i = 0;
    for i in 0..a.len() {
        if &a[i..i + 1] == b {
            return &a[..i];
        }
    }
    a
}

fn compare_keys(a: &[String], b: &[String]) -> usize {
    let min = std::cmp::min(a.len(), b.len());

    for i in 0..min {
        if &a[i][0..1] == ":" && &b[i][0..1] == ":" {
            continue;
        }

        if a[i] != b[i] {
            return i + 1;
        }
    }

    min
}

fn match_route<'a, 'b>(params: &mut Params<'a, 'b>, keys: &'a [String], s: &'b mut str) -> bool {
    let mut index = 0;
    for k in keys {
        match &k[1..] {
            "*" => {
                return true;
            }
            ":" => {}
            _ => {}
        }
    }

    false
}

#[derive(Debug)]
pub enum PathParseError<'a> {
    InsufficientLength,
    UnexpectedToken(&'a str),
}

pub fn parse_key<'a>(key: &'a str) -> Result<Vec<String>, PathParseError<'a>> {
    let mut xs = Vec::new();

    let parts = key.split("/");

    let mut buf = String::new();
    let mut end = false;

    for p in parts.filter(|s| s != &"").filter(|s| s != &"/") {
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
                    xs.push(buf.clone());
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

// #[test]
fn test_parse() {
    let res = parse_key("/api/hello/:name/:age/*").unwrap();
    assert_eq!(
        res,
        vec![
            "api/hello/".to_string(),
            ":name".to_string(),
            ":age".to_string(),
            "*".to_string()
        ]
    );

    let res = parse_key("/api/hello/*").unwrap();
    assert_eq!(res, vec!["api/hello/".to_string(), "*".to_string(),]);

    let res = parse_key("/api/hello/*/err").unwrap();
    assert_eq!(
        res,
        vec![
            "api/hello/".to_string(),
            "*".to_string(),
            "err/".to_string()
        ]
    );

    let res = parse_key("/query/*").unwrap();
    assert_eq!(res, vec!["query/".to_string(), "*".to_string()]);
}

#[test]
fn test_node() {
    let mut node = Node::new(Vec::new());

    let keys = parse_key("/api/hello/:name").unwrap();
    node.insert(&keys, 1);

    let keys = parse_key("/api/goodbye/:name/:age").unwrap();
    node.insert(&keys, 2);

    let keys = parse_key("/api/hello/:name/:age").unwrap();
    node.insert(&keys, 3);

    let keys = parse_key("/api/hello/:name/:age").unwrap();
    node.insert(&keys, 6);

    let keys = parse_key("/a/b/*").unwrap();
    node.insert(&keys, 4);

    let keys = parse_key("/api/hello").unwrap();
    node.insert(&keys, 0);

    let keys = parse_key("/:id/collections").unwrap();
    node.insert(&keys, 8);

    let res = Node {
        path: vec![],
        data: None,
        children: vec![
            Node {
                path: vec![":id".to_string(), "collections/".to_string()],
                data: Some(8),
                children: vec![],
            },
            Node {
                path: vec!["/a".to_string()],
                data: None,
                children: vec![
                    Node {
                        path: vec!["/b/".to_string(), "*".to_string()],
                        data: Some(4),
                        children: vec![],
                    },
                    Node {
                        path: vec!["pi/".to_string()],
                        data: None,
                        children: vec![
                            Node {
                                path: vec![
                                    "goodbye/".to_string(),
                                    ":name".to_string(),
                                    ":age".to_string(),
                                ],
                                data: Some(2),
                                children: vec![],
                            },
                            Node {
                                path: vec!["hello/".to_string()],
                                data: Some(0),
                                children: vec![Node {
                                    path: vec![":name".to_string()],
                                    data: Some(1),
                                    children: vec![Node {
                                        path: vec![":age".to_string()],
                                        data: Some(6),
                                        children: vec![],
                                    }],
                                }],
                            },
                        ],
                    },
                ],
            },
        ],
    };

    assert_eq!(node, res);
}

#[test]
fn test_node_get() {
    let trie = Node {
        path: vec![],
        data: None,
        children: vec![Node {
            path: vec!["/a".to_string()],
            data: None,
            children: vec![
                Node {
                    path: vec!["/b/".to_string(), "*".to_string()],
                    data: Some(6),
                    children: vec![],
                },
                Node {
                    path: vec!["pi/".to_string()],
                    data: None,
                    children: vec![
                        Node {
                            path: vec![
                                "goodbye/".to_string(),
                                ":name".to_string(),
                                ":age".to_string(),
                            ],
                            data: Some(2),
                            children: vec![],
                        },
                        Node {
                            path: vec!["hello".to_string()],
                            data: Some(0),
                            children: vec![Node {
                                path: vec![":name".to_string()],
                                data: Some(1),
                                children: vec![Node {
                                    path: vec![":age".to_string()],
                                    data: Some(3),
                                    children: vec![],
                                }],
                            }],
                        },
                    ],
                },
            ],
        }],
    };

    let (r, params) = trie.get("/api/hello/world").unwrap();
    assert_eq!(r, 1);
    assert_eq!(params.get("name"), Some("world"));

    let (r, params) = trie.get("/api/goodbye/world/2").unwrap();
    assert_eq!(r, 2);
    assert_eq!(params.get("name"), Some("world"));
    assert_eq!(params.get("age"), Some("2"));

    let (r, params) = trie.get("/api/hello/world/2").unwrap();
    assert_eq!(r, 3);
    assert_eq!(params.get("name"), Some("world"));
    assert_eq!(params.get("age"), Some("2"));

    let (r, _params) = trie.get("/a/b/string").unwrap();
    assert_eq!(r, 4);
}
