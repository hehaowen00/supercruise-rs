use supercruise_rs::path_trie::node::*;

fn main() {
    let mut node = Node::new(vec![]);
    node.insert("/api/hello/:name", 1);
    node.insert("/api/goodbye/:name", 2);
    node.insert("/query/*", 4);
    node.insert("/api/afternoon/:name", 2);
    println!("{:#?}", node);
}
