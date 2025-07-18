use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct TencentCloudRequest {
    secret_id: String,
    secret_key: String,
    service: String,
    host: String,
    region: String,
    action: String,
    version: String,
    payload: String,
    token: String,
}

impl TencentCloudRequest {
    pub fn new(
        secret_id: String,
        secret_key: String,
        service: String,
        host: String,
        region: String,
        action: String,
        version: String,
        payload: String,
        token: String,
    ) -> Self {
        Self {
            secret_id,
            secret_key,
            service,
            host,
            region,
            action,
            version,
            payload,
            token,
        }
    }

    pub async fn send(&self) -> anyhow::Result<String> {
        let timestamp = Utc::now().timestamp();
        let date = Utc::now().format("%Y-%m-%d").to_string();

        // Step 1: Create canonical request
        let canonical_request = self.create_canonical_request()?;
        println!("Canonical request:\n{}", canonical_request);

        // Step 2: Create string to sign
        let string_to_sign = self.create_string_to_sign(&canonical_request, &date, timestamp)?;
        println!("String to sign:\n{}", string_to_sign);

        // Step 3: Calculate signature
        let signature = self.calculate_signature(&string_to_sign, &date)?;
        println!("Signature: {}", signature);

        // Step 4: Create authorization header
        let authorization = self.create_authorization(&signature, &date)?;
        println!("Authorization: {}", authorization);

        // Step 5: Send request
        let response = self.send_request(&authorization, timestamp).await?;
        Ok(response)
    }

    fn create_canonical_request(&self) -> anyhow::Result<String> {
        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_querystring = "";
        let canonical_headers = format!(
            "content-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\n",
            self.host,
            self.action.to_lowercase()
        );
        let signed_headers = "content-type;host;x-tc-action";

        let mut hasher = Sha256::new();
        hasher.update(self.payload.as_bytes());
        let hashed_request_payload = format!("{:x}", hasher.finalize());

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            http_request_method,
            canonical_uri,
            canonical_querystring,
            canonical_headers,
            signed_headers,
            hashed_request_payload
        );

        Ok(canonical_request)
    }

    fn create_string_to_sign(
        &self,
        canonical_request: &str,
        date: &str,
        timestamp: i64,
    ) -> anyhow::Result<String> {
        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{}/{}/tc3_request", date, self.service);

        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let hashed_canonical_request = format!("{:x}", hasher.finalize());

        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp, credential_scope, hashed_canonical_request
        );

        Ok(string_to_sign)
    }

    fn calculate_signature(
        &self,
        string_to_sign: &str,
        date: &str,
    ) -> anyhow::Result<String> {
        // Create secret date
        let mut mac = HmacSha256::new_from_slice(format!("TC3{}", self.secret_key).as_bytes())?;
        mac.update(date.as_bytes());
        let secret_date = hex::encode(mac.finalize().into_bytes());

        // Create secret service
        let mut mac = HmacSha256::new_from_slice(&hex::decode(secret_date)?)?;
        mac.update(self.service.as_bytes());
        let secret_service = hex::encode(mac.finalize().into_bytes());

        // Create secret signing
        let mut mac = HmacSha256::new_from_slice(&hex::decode(secret_service)?)?;
        mac.update(b"tc3_request");
        let secret_signing = hex::encode(mac.finalize().into_bytes());

        // Calculate signature
        let mut mac = HmacSha256::new_from_slice(&hex::decode(secret_signing)?)?;
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        Ok(signature)
    }

    fn create_authorization(
        &self,
        signature: &str,
        date: &str,
    ) -> anyhow::Result<String> {
        let algorithm = "TC3-HMAC-SHA256";
        let credential_scope = format!("{}/{}/tc3_request", date, self.service);
        let signed_headers = "content-type;host;x-tc-action";

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.secret_id, credential_scope, signed_headers, signature
        );

        Ok(authorization)
    }

    async fn send_request(
        &self,
        authorization: &str,
        timestamp: i64,
    ) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let url = format!("https://{}", self.host);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(authorization)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        headers.insert(
            header::HOST,
            header::HeaderValue::from_str(&self.host)?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-action"),
            header::HeaderValue::from_str(&self.action)?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-timestamp"),
            header::HeaderValue::from_str(&timestamp.to_string())?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-version"),
            header::HeaderValue::from_str(&self.version)?,
        );
        headers.insert(
            header::HeaderName::from_static("x-tc-region"),
            header::HeaderValue::from_str(&self.region)?,
        );
        if !self.token.is_empty() {
            headers.insert(
                header::HeaderName::from_static("x-tc-token"),
                header::HeaderValue::from_str(&self.token)?,
            );
        }

        let response = client
            .post(&url)
            .headers(headers)
            .body(self.payload.clone())
            .send()
            .await?;

        let response_text = response.text().await?;
        Ok(response_text)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudResponse<T> {
    #[serde(rename = "Response")]
    pub response: TencentCloudResponseDeatil<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudResponseDeatil<T> {
    #[serde(rename = "Error")]
    pub error: Option<TencentCloudResponseError>,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(flatten)]
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TencentCloudResponseError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

impl TencentCloudResponseError {
    pub fn to_string(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }
}
