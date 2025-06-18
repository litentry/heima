// This is where the client code of 1inch will reside
// It's main object will be to provide swapping
use reqwest::{Client};

pub struct InchClient {
    client: Client,
    base_url: String,
}

// Setup primitives for request and response 
// How to use bear token? 

impl InchClient {
    // call swap request same as GoLang 
}