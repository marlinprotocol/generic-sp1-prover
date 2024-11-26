use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
struct JsonResponse {
    message: String,
    data: Option<Value>,
}

struct ResponseOptions {
    data: Option<Value>,
    message: String,
    status_code: StatusCode,
}

struct ResponseHandler {
    options: ResponseOptions,
}

impl ResponseHandler {
    fn new(options: ResponseOptions) -> Self {
        ResponseHandler { options }
    }

    fn create_json_response(self) -> JsonResponse {
        JsonResponse {
            message: self.options.message,
            data: self.options.data,
        }
    }

    fn create_http_response(self) -> HttpResponse {
        let status_code = self.options.status_code;
        let json_resp = self.create_json_response();
        HttpResponse::build(status_code).json(json_resp)
    }
}

//Generate response
pub fn response(
    message: &str,
    status_code: StatusCode,
    data: Option<serde_json::Value>,
) -> HttpResponse {
    let options = ResponseOptions {
        data,
        message: message.to_string(),
        status_code,
    };

    let response_handler = ResponseHandler::new(options);
    response_handler.create_http_response()
}

use bindings::shared_types::Ask;

#[derive(Serialize, Debug, Deserialize)]
pub struct GenerateProofInputs {
    pub ask: Ask,
    pub private_input: Vec<u8>,
    pub ask_id: u64,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct InputPayload {
    pub public: String,
    pub secrets: Option<String>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct EncryptedInputPayload {
    pub acl: String,
    pub encrypted_secrets: String,
    pub me_decryption_url: String,
    pub market_id: String,
}

pub mod secret_inputs_helpers;

#[derive(Serialize, Debug, Deserialize)]
pub struct AskPayload {
    pub ask_id: u64,
    pub encrypted_secret: String,
    pub acl: String,
}
