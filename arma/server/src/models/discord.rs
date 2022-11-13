use arma_rs::IntoArma;

pub struct MemberInfo {
    pub steam: String,
    pub discord_id: String,
    pub roles: Vec<String>,
}

impl IntoArma for MemberInfo {
    fn to_arma(&self) -> arma_rs::Value {
        arma_rs::Value::Array(vec![
            arma_rs::Value::String(self.steam.clone()),
            arma_rs::Value::String(self.discord_id.clone()),
            arma_rs::Value::Array(
                self.roles
                    .clone()
                    .into_iter()
                    .map(arma_rs::Value::String)
                    .collect(),
            ),
        ])
    }
}

pub struct GuildInfo {
    pub roles: Vec<(String, String)>,
}

impl IntoArma for GuildInfo {
    fn to_arma(&self) -> arma_rs::Value {
        arma_rs::Value::Array(
            self.roles
                .clone()
                .into_iter()
                .map(|(id, name)| {
                    arma_rs::Value::Array(vec![
                        arma_rs::Value::String(id),
                        arma_rs::Value::String(name),
                    ])
                })
                .collect(),
        )
    }
}
