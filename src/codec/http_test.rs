#[test]
fn http_test() {
    use http::Uri;
    let uri = "/api/todos?a=b".parse::<Uri>().unwrap();
    println!("{:?}", uri.path());
}
