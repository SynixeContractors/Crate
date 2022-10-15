#[derive(Serialize, Deserialize)]
pub struct Wrapper<T> {
    #[serde(rename = "i")]
    inner: T,
    #[serde(rename = "m")]
    metadata: HashMap<String, String>,
}

impl<T> Wrapper<T> {
    pub fn new(token: Option<Token>, inner: T) -> Self {
        Self {
            token,
            inner,
            metadata: HashMap::new(),
        }
    }

    pub const fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }

    pub const fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn into_parts(self) -> (T, Option<Token>, HashMap<String, String>) {
        (self.inner, self.token, self.metadata)
    }
}
