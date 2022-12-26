pub enum HTTPRequestMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

impl PartialEq<HTTPRequestMethod> for HTTPRequestMethod {
    fn eq(&self, other: &HTTPRequestMethod) -> bool {
        match (self, other) {
            (HTTPRequestMethod::GET, HTTPRequestMethod::GET) => true,
            (HTTPRequestMethod::POST, HTTPRequestMethod::POST) => true,
            (HTTPRequestMethod::PUT, HTTPRequestMethod::PUT) => true,
            (HTTPRequestMethod::DELETE, HTTPRequestMethod::DELETE) => true,
            _ => false,
        }
    }
}
