use wiremock::{Match, Request};

pub struct FormEncodedBodyMatcher {
    expected: Vec<(String, String)>,
}

impl FormEncodedBodyMatcher {
    pub const fn new(expected: Vec<(String, String)>) -> Self {
        Self { expected }
    }
}

impl Match for FormEncodedBodyMatcher {
    fn matches(&self, req: &Request) -> bool {
        if let Ok(body) = String::from_utf8(req.body.clone()) {
            if let Ok(decoded_body) = serde_urlencoded::from_str::<Vec<(String, String)>>(&body) {
                return decoded_body == self.expected;
            }
        }
        false
    }
}
