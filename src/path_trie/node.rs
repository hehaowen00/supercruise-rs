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

    pub fn insert(&mut self, key: &str, value: T) {
        let keys = parse_key(key).unwrap();

        println!("{:?}", keys);

        if keys.len() == 0 {
            self.data = Some(value);
            return;
        }

        if self.children.len() == 0 {
            let mut node = Node::new(keys);
            node.data = Some(value);
            self.children.push(node);
            return;
        }
    }

    pub fn get<'a, 'b>(&'a self, key: &'b str) -> Option<&(T, Params<'a, 'b>)> {
        let keys = parse_key(key).unwrap();

        None
    }
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

#[test]
fn test_parse() {
    let p = "/api/hello/:name/:age/*";
    println!("{:?}", parse_key(p));

    let p = "/api/hello/*";
    println!("{:?}", parse_key(p));

    let p = "/api/hello/*/err";
    println!("{:?}", parse_key(p));

    let p = "/query/*";
    println!("{:?}", parse_key(p));
}

#[test]
fn test_node() {
    let mut node = Node::new(Vec::new());
    node.insert("/api/hello/:name", 1);
    node.insert("/api/hello/:name/:age", 2);
    node.insert("/api/goodbye/:name/:age", 3);
    node.insert("/a/b", 4);

    let res = Node {
        path: vec!["a".to_string()],
        data: None,
        children: vec![
            Node {
                path: vec!["pi/".to_string()],
                data: None,
                children: vec![
                    Node {
                        path: vec!["hello/".to_string(), ":name".to_string()],
                        data: Some(1),
                        children: vec![Node {
                            path: vec![":age".to_string()],
                            data: Some(3),
                            children: vec![],
                        }],
                    },
                    Node {
                        path: vec![
                            "goodbye/".to_string(),
                            ":name".to_string(),
                            ":age".to_string(),
                        ],
                        data: Some(2),
                        children: vec![],
                    },
                ],
            },
            Node {
                path: vec!["/b".to_string(), "*".to_string()],
                data: Some(4),
                children: vec![],
            },
        ],
    };

    assert_eq!(node, res);
}

#[test]
fn test_node_get() {
    let trie = Node {
        path: vec!["a".to_string()],
        data: None,
        children: vec![
            Node {
                path: vec!["pi/".to_string()],
                data: None,
                children: vec![
                    Node {
                        path: vec!["hello/".to_string(), ":name".to_string()],
                        data: Some(1),
                        children: vec![Node {
                            path: vec![":age".to_string()],
                            data: Some(3),
                            children: vec![],
                        }],
                    },
                    Node {
                        path: vec![
                            "goodbye/".to_string(),
                            ":name".to_string(),
                            ":age".to_string(),
                        ],
                        data: Some(2),
                        children: vec![],
                    },
                ],
            },
            Node {
                path: vec!["/b".to_string(), "*".to_string()],
                data: Some(4),
                children: vec![],
            },
        ],
    };

    let (r, params) = trie.get("/api/hello/world").unwrap();
    assert_eq!(*r, 1);
    assert_eq!(params.get("name"), Some("world"));

    let (r, params) = trie.get("/api/goodbye/world/2").unwrap();
    assert_eq!(*r, 2);
    assert_eq!(params.get("name"), Some("world"));
    assert_eq!(params.get("age"), Some("2"));

    let (r, params) = trie.get("/api/hello/world/2").unwrap();
    assert_eq!(*r, 3);
    assert_eq!(params.get("name"), Some("world"));
    assert_eq!(params.get("age"), Some("2"));

    let (r, params) = trie.get("/a/b/string").unwrap();
    assert_eq!(*r, 4);
}
