use supercruise_rs::path_trie::node::*;

fn main() {
    let mut node = Node::new(vec![]);

    let keys = parse_key("/api/hello/:name").unwrap();
    node.insert(&keys, 1);

    let keys = parse_key("/api/goodbye/:name/:age").unwrap();
    node.insert(&keys, 2);

    let keys = parse_key("/a/b/*").unwrap();
    node.insert(&keys, 4);

    let keys = parse_key("/api/hello/:name/:age").unwrap();
    node.insert(&keys, 3);

    let keys = parse_key("/api/hello/:name/:age").unwrap();
    node.insert(&keys, 6);

    let keys = parse_key("/api/hello").unwrap();
    node.insert(&keys, 0);

    let keys = parse_key("/:id/collections").unwrap();
    println!("{:?}", keys);
    node.insert(&keys, 8);

    println!("{:#?}", node);

    let res = node.get("/api/hello/world");
    println!("{:?}", res);
}
