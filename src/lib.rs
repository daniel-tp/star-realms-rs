pub use self::error::{Error, Result};
use reqwest::Client;
use serde::Deserialize;

mod error; //TODO: Rename

/// A single logged in instance of a logged in Star Realms user
#[derive(Debug, Clone)]
pub struct StarRealms {
    token: Token,
    core_version: usize,
    client: Client,
}


impl StarRealms {
    /// Create a new instance of StarRealms using a user's Username and Password to login.
    /// Password is not retained internally and is sent via HTTPS connection to official Star Realms servers
    pub async fn new(username: &str, password: &str) -> Result<StarRealms> {
        let mut sr = StarRealms {
            token: Token::default(),
            core_version: 45,
            client: reqwest::Client::new(),
        };
        sr.new_token(username, password).await?;
        sr.core_version().await?;
        Ok(sr)
    }

    /// Create a new instance of StarRealms using a user's token. The required token is Token2 from the token response from the server.
    /// As we don't get a token, we also don't have other data available that is usually provided when retrieving a token, such as purchases.
    pub async fn new_with_token2_str(token: &str) -> Result<StarRealms> {
        let mut sr = StarRealms {
            token: Token::default(),
            core_version: 45,
            client: reqwest::Client::new(),
        };
        sr.token.token2 = token.to_string();
        sr.core_version().await?;
        Ok(sr)
    }

    /// Create a new instance of StarRealms using a previously made Token.
    pub async fn new_with_token(token: Token) -> Result<StarRealms> {
        let mut sr = StarRealms {
            token: token,
            core_version: 45,
            client: reqwest::Client::new(),
        };
        sr.core_version().await?;
        Ok(sr)
    }

    /// Gets a login token using the username and password.
    /// This token doesn't seem to expire
    async fn new_token(&mut self, username: &str, password: &str) -> Result<()> {
        let params = [("username", username), ("password", password)];
        let res = self
            .client
            .post("https://srprodv2.whitewizardgames.com/Account/Login")
            .form(&params)
            .send()
            .await?;
        if res.status() != 200 {
            return Err(Error::InvalidAPIResponse(res.status().to_string()));
        }
        self.token = res.json().await?;
        Ok(())
    }

    /// Get the latest core version via trial and error
    /// Incorrect core version causes empty or invalid responses for other calls
    async fn core_version(&mut self) -> Result<()> {
        for core_version in 44..100 {
            let res = self
                .client
                .get("https://srprodv2.whitewizardgames.com/NewGame/ListActivitySortable")
                .header("Auth", &self.token.token2)
                .header("coreversion", core_version)
                .send()
                .await?;
            if res.status() == 200 {
                self.core_version = core_version;
                return Ok(());
            }
        }
        Err(Error::UnknownCoreVersion())
    }

    /// Get the latest user activity, including current player data
    pub async fn activity(&self) -> Result<Activity> {
        let res = self
            .client
            .get("https://srprodv2.whitewizardgames.com/NewGame/ListActivitySortable")
            .header("Auth", &self.token.token2)
            .header("coreversion", self.core_version)
            .send()
            .await?;
        if res.status() != 200 {
            return Err(Error::InvalidAPIResponse(res.status().to_string()));
        }
        Ok(res.json().await?)
    }

    /// Get the Star Realms user's Token struct
    pub fn token(&self) -> &Token {
        &self.token
    }
}

//TODO: More rust friendly names?
#[derive(Default, Deserialize, Debug, Clone)]
pub struct Token {
    name: String,
    id: usize,
    token1: String,
    token2: String,
    purchases: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Activity {
    pub acceptedterms: bool,
    pub avatar: String,
    pub rankstars: i64,
    pub ranktotalstars: i64,
    pub level: i64,
    pub arenatrophystars: i64,
    pub hasfreearena: bool,
    pub pendingrewards: ::serde_json::Value, //TODO: Find what this is
    pub queues: Vec<::serde_json::Value>,    //TODO: Find what this is
    pub challenges: Vec<Challenge>,
    pub activegames: Vec<Game>,
    pub finishedgames: Vec<Game>,
    pub result: String,
}

//TODO: Merge ActiveGame and FinishedGame under "Game"
#[derive(Debug, Deserialize)]
pub struct Game {
    pub gameid: i64,
    pub timing: String,
    pub mmdata: String,     //TODO: Change this into a struct
    pub clientdata: String, //TODO: Change this into a struct
    pub opponentname: String,
    #[serde(default)]
    pub actionneeded: bool,
    #[serde(default)]
    pub endreason: i64, //TODO: Figure out what these are
    #[serde(default)]
    pub won: bool,
    pub lastupdatedtime: String, //TODO: Change to chrono time?
    pub isleaguegame: bool,
    pub istournamentgame: bool,
}

#[derive(Debug, Deserialize)]
pub struct Challenge {
    pub challengeid: i64,
    pub challengername: String,
    pub challengercommander: String,
    pub opponentname: String,
    pub mmdata: String,
    pub status: String, //TODO: Change to enum?
    pub statusdescription: String,
    pub lastupdatedtime: String,
    pub timing: String, //TODO: Change to enum?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
        dotenv::dotenv().ok();
    }

    #[tokio::test]
    async fn new_star_realms_test() -> Result<()> {
        init();
        StarRealms::new(
            env::var("SR_USERNAME").unwrap().as_str(),
            env::var("SR_PASSWORD").unwrap().as_str(),
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn incorrect_login_test() {
        init();
        StarRealms::new("fakeuser123", "fakepass123").await.unwrap();
    }

    #[tokio::test]
    async fn list_activity_test() -> Result<()> {
        init();
        let sr = StarRealms::new(
            env::var("SR_USERNAME").unwrap().as_str(),
            env::var("SR_PASSWORD").unwrap().as_str(),
        )
        .await?;
        sr.activity().await?;
        Ok(())
    }
}
