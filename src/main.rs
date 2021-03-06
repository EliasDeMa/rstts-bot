use dotenv::dotenv;
use hyper::{body::Bytes, StatusCode};
use image::Rgb;
use once_cell::sync::Lazy;
use sus::sus1;
use std::{collections::HashMap, convert::TryInto, error::Error, fs::{self, File}, sync::Arc};

use serenity::{Result as SerenityResult, async_trait, client::{Client, Context, EventHandler}, framework::standard::CommandError, utils::MessageBuilder, framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group, hook},
        },
    }, model::{channel::Message, id::UserId}, prelude::*};

use hyper::{body, Body, Client as HyperClient, Method, Request};
use std::{
    self, 
    io::prelude::*,
    env
};
use uwuifier::uwuify_str_sse;

use songbird::{
    input::{
        self,
        cached::Memory,
    },
    SerenityInit,
};


mod sus;

const C: Rgb<u8> = Rgb([197, 17, 17]);
const C2: Rgb<u8> = Rgb([122, 8, 56]);

static VOICES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("altman".to_string(), "sam-altman".to_string());
    m.insert("arnold".to_string(), "arnold-schwarzenegger".to_string());
    m.insert("attenborough".to_string(), "david-attenborough".to_string());
    m.insert("ayoade".to_string(), "richard-ayoade".to_string());
    m.insert("bart".to_string(), "bart-simpson".to_string());
    m.insert("ben_stein".to_string(), "ben-stein".to_string());
    m.insert("betty_white".to_string(), "betty-white".to_string());
    m.insert("bill_clinton".to_string(), "bill-clinton".to_string());
    m.insert("bill_gates".to_string(), "bill-gates".to_string());
    m.insert("bill_nye".to_string(), "bill-nye".to_string());
    m.insert("bob_barker".to_string(), "bob-barker".to_string());
    m.insert("boss".to_string(), "the-boss".to_string());
    m.insert("brimley".to_string(), "wilford-brimley".to_string());
    m.insert("broomstick".to_string(), "boomstick".to_string());
    m.insert("bush".to_string(), "george-w-bush".to_string());
    m.insert("carter".to_string(), "jimmy-carter".to_string());
    m.insert("christopher_lee".to_string(), "christopher-lee".to_string());
    m.insert("cooper".to_string(), "anderson-cooper".to_string());
    m.insert("craig_ferguson".to_string(), "craig-ferguson".to_string());
    m.insert("cramer".to_string(), "jim-cramer".to_string());
    m.insert("cranston".to_string(), "bryan-cranston".to_string());
    m.insert("crypt_keeper".to_string(), "crypt-keeper".to_string());
    m.insert("darth_vader".to_string(), "james-earl-jones".to_string());
    m.insert("david_cross".to_string(), "david-cross".to_string());
    m.insert("degrasse".to_string(), "neil-degrasse-tyson".to_string());
    m.insert("dench".to_string(), "judi-dench".to_string());
    m.insert("devito".to_string(), "danny-devito".to_string());
    m.insert("dr_phil".to_string(), "dr-phil-mcgraw".to_string());
    m.insert("earl_jones".to_string(), "james-earl-jones".to_string());
    m.insert("fred_rogers".to_string(), "fred-rogers".to_string());
    m.insert("gottfried".to_string(), "gilbert-gottfried".to_string());
    m.insert("goku".to_string(), "goku".to_string());
    m.insert("hillary_clinton".to_string(), "hillary-clinton".to_string());
    m.insert("homer".to_string(), "homer-simpson".to_string());
    m.insert("krabs".to_string(), "mr-krabs".to_string());
    m.insert("larry_king".to_string(), "larry-king".to_string());
    m.insert("lisa".to_string(), "lisa-simpson".to_string());
    m.insert("luckey".to_string(), "palmer-luckey".to_string());
    m.insert("mcconnell".to_string(), "mitch-mcconnell".to_string());
    m.insert("nimoy".to_string(), "leonard-nimoy".to_string());
    m.insert("nixon".to_string(), "richard-nixon".to_string());
    m.insert("obama".to_string(), "barack-obama".to_string());
    m.insert("oliver".to_string(), "john-oliver".to_string());
    m.insert("palin".to_string(), "sarah-palin".to_string());
    m.insert("paul_graham".to_string(), "paul-graham".to_string());
    m.insert("paula_deen".to_string(), "paula-deen".to_string());
    m.insert("penguinz0".to_string(), "moistcr1tikal".to_string());
    m.insert("reagan".to_string(), "ronald-reagan".to_string());
    m.insert("rickman".to_string(), "alan-rickman".to_string());
    m.insert("rosen".to_string(), "michael-rosen".to_string());
    m.insert("saruman".to_string(), "christopher-lee".to_string());
    m.insert("scout".to_string(), "scout".to_string());
    m.insert("shapiro".to_string(), "ben-shapiro".to_string());
    m.insert("shohreh".to_string(), "shohreh-aghdashloo".to_string());
    m.insert("simmons".to_string(), "j-k-simmons".to_string());
    m.insert("snake".to_string(), "solid-snake".to_string());
    m.insert("snape".to_string(), "severus-snape".to_string());
    m.insert("sonic".to_string(), "sonic".to_string());
    m.insert("spongebob".to_string(), "spongebob-squarepants".to_string());
    m.insert("squidward".to_string(), "squidward".to_string());
    m.insert("takei".to_string(), "george-takei".to_string());
    m.insert("thiel".to_string(), "peter-thiel".to_string());
    m.insert("trevor".to_string(), "trevor-philips".to_string());
    m.insert("trump".to_string(), "donald-trump".to_string());
    m.insert("tucker".to_string(), "tucker-carlson".to_string());
    m.insert("tupac".to_string(), "tupac-shakur".to_string());
    m.insert("vegeta".to_string(), "vegeta".to_string());
    m.insert("wiseau".to_string(), "tommy-wiseau".to_string());
    m.insert("wizard".to_string(), "wizard".to_string());
    m.insert("yugi".to_string(), "yami-yugi".to_string());
    m.insert("zuckerberg".to_string(), "mark-zuckerberg".to_string());

    m
});

#[group]
#[commands(join, say, leave, voices, uwu, sus)]
struct General;
struct Handler;

#[async_trait]
impl EventHandler for Handler { }

struct FileLock;

impl TypeMapKey for FileLock {
    type Value = Arc<Mutex<HashMap<UserId, String>>>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let token = {
        let mut token = String::new();
        for (key, value) in env::vars() {
            if key == "DISCORD_TOKEN" {
                token = value;
            }
        }

        token
    };
    
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("tts_")) // set the bot's prefix to "tts_"
        .after(clear) 
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        
        // The FileLock Value has the following type:
        // Arc<RwLock<HashMap<UserId, String>>>
        // So, we have to insert the same type to it.
        data.insert::<FileLock>(Arc::new(Mutex::new(HashMap::new())));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
async fn sus(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(file) = msg.attachments.first() {
        let name = file.filename.clone();
        let downloaded = file.download().await?;
        let mut file = File::create(&name).unwrap();

        file.write_all(&downloaded).unwrap();
        let gif = sus1(&name).unwrap();

        let files = vec![gif.as_str()];

        msg.channel_id.send_files(ctx, files, |m| m.content("created gif")).await?;
        fs::remove_file(&name);
        fs::remove_file(&gif);
            
    } else {
        check_msg(msg.channel_id.say(ctx, "Please give a file").await);
    }
    
    Ok(())
}

#[command]
async fn voices(ctx: &Context, msg: &Message) -> CommandResult {
    let mut voices = String::new();

    for (k, v) in VOICES.iter() {
        voices += &format!("{}:   {}\n", k, v);
    }

    let response = MessageBuilder::new()
        .push_bold_line("The available voices and their keywords are:")
        .push(voices)
        .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[hook]
async fn clear(ctx: &Context, msg: &Message, cmd_name: &str, _error: Result<(), CommandError>) {
    if cmd_name == "say" {
        let sources_lock = ctx.data
            .read()
            .await
            .get::<FileLock>()
            .cloned()
            .expect("FileLock was installed at startup");

        let mut sources = sources_lock.lock().await;
        
        if let Some(file_name) = sources.get(&msg.author.id) { 
            match fs::remove_file(file_name) {
                Ok(_) => {
                    sources.remove(&msg.author.id);
                },
                Err(_) => {
                    sources.remove(&msg.author.id);
                }
            }              
        }     
    }
}

#[command]
async fn uwu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    check_msg(msg.channel_id.say(ctx, uwuify_str_sse(args.rest())).await);
    
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let filename = format!("{}.wav", uuid::Uuid::new_v4().to_string());

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let sources_lock = ctx.data
            .read()
            .await
            .get::<FileLock>()
            .cloned()
            .expect("FileLock was installed at startup");

        let mut sources = sources_lock.lock().await;
        sources.insert(msg.author.id, filename.clone());
        let name = args.single::<String>()?;
        let sentence = args.rest();

        if let Some(passing_name) = VOICES.get(&name) {
            let (code, audio) = get_wav_file(passing_name, sentence).await.unwrap();
            
            if code == StatusCode::OK {
                let mut buffer = File::create(filename.clone())?;
                buffer.write_all(&audio)?;

                let audio = Memory::new(
                    input::ffmpeg(filename.clone())
                        .await
                        .expect("File should be in root folder."),
                    ).expect("These parameters are well-defined.");
                let _ = audio.raw.spawn_loader();
                
                let _song = handler.play_source(audio.try_into().unwrap());
                check_msg(msg.channel_id.say(&ctx.http, "Playing!").await);
            } else {
                println!("{}, via {}", code, name);
                check_msg(msg.channel_id.say(&ctx.http, "Couldn't retrieve file from server!").await);
            }
            
        } else {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("{} is not an eligible voice.", name))
                    .await,
            );
        }
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

async fn get_wav_file(speaker: &str, text: &str) -> Result<(StatusCode, Bytes), Box<dyn Error>> {
    let req = Request::builder()
        .method(Method::POST)
        .uri("http://mumble.stream/speak")
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(Body::from(format!("{{\"speaker\": \"{}\", \"text\": \"{}\"}}", speaker, text)))?;

    let client = HyperClient::new();
    let res = client.request(req).await?;
    let code = res.status();
    let body_bytes = body::to_bytes(res.into_body()).await?;

    Ok((code, body_bytes))
} 

