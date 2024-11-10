use chrono::{DateTime, Utc};
use jwt_simple::prelude::*;
use tonic::service::Interceptor;

const _JWT_DURATION: u64 = 30;
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Clone)]
pub struct DecodingKey(Ed25519PublicKey);

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, jwt_simple::Error> {
        let key = Ed25519PublicKey::from_pem(pem)?;
        Ok(Self(key))
    }
    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, jwt_simple::Error> {
        let options = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };

        let claims = self.0.verify_token::<User>(token, Some(options))?;
        Ok(claims.custom)
    }
}

impl Interceptor for DecodingKey {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let token = req
            .metadata()
            .get("authorization")
            .and_then(|t| t.to_str().ok());
        let user = match token {
            Some(token) => self
                .verify(token)
                .map_err(|e| tonic::Status::unauthenticated(e.to_string())),
            _ => return Err(tonic::Status::unauthenticated("No valid auth token")),
        }?;

        req.extensions_mut().insert(user);
        Ok(req)
    }
}
