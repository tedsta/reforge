#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct ChatMsg {
    pub author_name: String,
    pub content: String,
}