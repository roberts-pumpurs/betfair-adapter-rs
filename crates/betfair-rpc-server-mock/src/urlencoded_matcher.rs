use wiremock::{Match, Request};

pub(crate) struct FormEncodedBodyMatcher {
    expected: Vec<(String, String)>,
}

impl FormEncodedBodyMatcher {
    pub(crate) const fn new(expected: Vec<(String, String)>) -> Self {
        Self { expected }
    }
}

impl Match for FormEncodedBodyMatcher {
    fn matches(&self, req: &Request) -> bool {
        if let Ok(body) = String::from_utf8(req.body.clone())
            && let Ok(decoded_body) = serde_urlencoded::from_str::<Vec<(String, String)>>(&body) {
                return decoded_body == self.expected;
            }
        false
    }
}
