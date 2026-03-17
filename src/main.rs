use std::env;

use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{
        ClientBuilder, Context, CreateMessage, EventHandler, GatewayIntents, Message, Ready, UserId,
    },
    async_trait,
};
use tracing_subscriber::FmtSubscriber;

struct Handler;

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    pub ip: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Assuming you want to send a message to a user by their ID
        let id = env::var("USER_ID").expect("Expected a user_id in the environment");
        let user_id = UserId::new(id.parse().unwrap());

        // Get the user and send them a direct message
        if let Ok(user) = ctx.http.get_user(user_id).await {
            let client = reqwest::Client::new();
            let res = client
                .get("https://api.ipify.org?format=json")
                .header(CONTENT_TYPE, "Content-Type: application/json")
                .send()
                .await;
            if let Ok(data) = res {
                let ip = data.json::<ApiResponse>().await.unwrap();
                let message = CreateMessage::new().content(ip.ip);
                if let Err(why) = user.dm(&ctx, message).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ip" {
            let client = reqwest::Client::new();
            let res = client
                .get("https://api.ipify.org?format=json")
                .header(CONTENT_TYPE, "Content-Type: application/json")
                .send()
                .await;
            if let Ok(data) = res {
                let ip = data.json::<ApiResponse>().await.unwrap();
                tracing::info!("IP: {}", ip.ip);
                if let Err(why) = msg.channel_id.say(&ctx.http, &ip.ip).await {
                    println!("Error sending message: {why:?}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(FmtSubscriber::builder().finish()).expect("OK");
    dotenv::dotenv().ok();
    let token = env::var("TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = ClientBuilder::new(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
