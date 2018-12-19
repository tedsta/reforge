#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub author_name: String,
    pub content: String,
}