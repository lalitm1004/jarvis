use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::env;
use dotenv::dotenv;
use std::process::Command;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.content.starts_with(">>jarvis, run ") {
            let command = msg.content.trim_start_matches(">>jarvis, run ");
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", command])
                    .output()
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
            };

            match output {
                Ok(output) => {
                    let response = if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        format!("Error: {}", String::from_utf8_lossy(&output.stderr))
                    };

                    // Format the response as a code block
                    let formatted_response = format!("```{}\n{}```", detect_language(command), response);

                    for message in split_message(&formatted_response, 2000) {
                        if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                }
                Err(e) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("Failed to execute command: {}", e)).await {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn split_message(content: &str, max_length: usize) -> Vec<String> {
    let mut messages = Vec::new();
    let mut start = 0;
    while start < content.len() {
        let end = std::cmp::min(start + max_length, content.len());
        messages.push(content[start..end].to_string());
        start += max_length;
    }
    messages
}

fn detect_language(command: &str) -> &str {
    // Basic detection of command to apply appropriate syntax highlighting
    if command.starts_with("echo") || command.starts_with("ls") || command.starts_with("pwd") {
        "sh"
    } else if command.starts_with("lscpu") || command.starts_with("uname") {
        "bash"
    } else {
        "" // No specific language
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN not set");
    
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

