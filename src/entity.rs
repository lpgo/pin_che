#[derive(Serialize, Deserialize)]
pub struct Person {
    #[serde(rename = "id")]
    pub id: String,
    pub name: String,
    pub age: i32,
}