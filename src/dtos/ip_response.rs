use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct IPResponse {
    pub country_code: String,
}
