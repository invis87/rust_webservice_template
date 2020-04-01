use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserResponse {
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTicketRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
}
