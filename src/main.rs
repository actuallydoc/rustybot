use serenity::{
    async_trait,
    framework::{
        standard::{
            macros::{command, group},
            CommandResult,
        },
        StandardFramework,
    },
    model::prelude::{Message, MessageCreateEvent, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    utils::MessageBuilder,
    Client,
};
use sqlx::mysql::MySqlPool;
use sqlx::Executor;
const DATABASE_NAME: &str = "";
const DATABASE_USER: &str = "";
const DATABASE_PASSWORD: &str = "";

#[group]
struct General;

struct Handler {
    pool: MySqlPool,
}
struct User<'a> {
    id: &'a u64,
    username: &'a String,
    discriminator: &'a u16,
    avatar: &'a String,
}
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "!ping" {
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);
                    return;
                }
            };
        }
        if msg.content == "!login" {
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);
                    return;
                }
            };
            let user = User {
                id: &msg.author.id.0,
                username: &msg.author.name,
                discriminator: &msg.author.discriminator,
                avatar: &msg.author.avatar_url().unwrap_or("".to_string()),
            };

            let mut conn = self.pool.acquire().await.unwrap();

            conn.execute("BEGIN").await.unwrap(); // unprepared, simple query
            conn.execute(
                sqlx::query(
                    "INSERT Into User (discord_id, username, discriminator, avatar) VALUES (?, ?, ?, ?)",
                )
                .bind(&user.id)
                .bind(&user.username)
                .bind(&user.discriminator)
                .bind(&user.avatar),
            )
            .await
            .unwrap();
            conn.execute("COMMIT").await.unwrap();
            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" successfully logged in!")
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let app = App::new();
    app.await.run().await;
}

struct App {
    client: Option<Client>,
}

trait DatabaseConnection {
    fn get_connection(&self) -> &MySqlPool;
}

const TOKEN: &str = "";
impl App {
    async fn new() -> Self {
        let mut handler = Handler {
            pool: MySqlPool::connect(
                &format!(
                    "mariadb://{}:{}@localhost/{}",
                    DATABASE_USER, DATABASE_PASSWORD, DATABASE_NAME
                )
                .as_str(),
            )
            .await
            .unwrap(),
        };
        let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
        let framework = StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&GENERAL_GROUP);
        let mut client = Client::builder(TOKEN, intents)
            .event_handler(handler)
            .framework(framework)
            .await
            .expect("Err creating client");

        App {
            client: Some(client),
        }
    }
    async fn run(&mut self) {
        let mut client = self.client.take().unwrap();
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    }
}

impl DatabaseConnection for Handler {
    fn get_connection(&self) -> &MySqlPool {
        &self.pool
    }
}
