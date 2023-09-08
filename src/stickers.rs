use teloxide::{
    requests::{Requester, ResponseResult},
    types::{InputFile, Message},
    Bot,
};

pub async fn send_sticker(bot: &Bot, msg: &Message, sticker_id: String) -> ResponseResult<()> {
    bot.send_sticker(msg.chat.id, InputFile::file_id(sticker_id))
        .await?;
    Ok(())
}

pub async fn send_many_stickers(
    bot: &Bot,
    msg: &Message,
    sticker_ids: Vec<String>,
) -> ResponseResult<()> {
    for sticker in sticker_ids {
        bot.send_sticker(msg.chat.id, InputFile::file_id(sticker))
            .await?;
    }
    Ok(())
}

// ! example to show what available fields are there in a struct
// use serde::Serialize;
// use serde_json::{json, Value};

// #[derive(Serialize)]
// struct MyStruct {
//     field1: i32,
//     field2: String,
// }

// fn main() {
//     let my_instance = MyStruct {
//         field1: 42,
//         field2: String::from("Hello, Rust!"),
//     };

//     // Serialize the struct into a JSON Value
//     let json_value: Value = serde_json::to_value(&my_instance).unwrap();

//     // Extract and print the field names
//     if let Value::Object(map) = json_value {
//         let field_names: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
//         println!("{:?}", field_names);
//     } else {
//         println!("Failed to extract field names.");
//     }
// }
