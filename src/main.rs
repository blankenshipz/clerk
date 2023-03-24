use std::error::Error;
use std::fs::File;
use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use walkdir::WalkDir;
use pdf_extract;
use regex::Regex;
use clap::Parser;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// TODO move to CLI arg
const DATA_SET_DIRECTORY: &str =".";

#[derive(Serialize, Deserialize)]
struct Config {
    categories: HashMap<String, Vec<String>>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Maximum length of content to read from files for matching.
    #[arg(short, long, default_value_t = 10_000)]
    max_read_length: usize,
    /// Excluded File Type
    #[arg(short, long, default_values = vec!["zip", "xlsx", "yml"])]
    exclude_file_type: Vec<String>,
    /// Location of Configuration file that defines file categories
    #[arg(short, long, default_value = "clerk.yml")]
    config_file: String,
}

struct Object {
    path: String,
    content: String
}

// TODO add debug logging for prompt
fn prompt(config: &Config, file: Object) -> String {
    let categories: String = config.categories.keys().cloned().collect::<Vec<String>>().join(", ");
    let mut categories_string: String = "".to_string();

    for (category_name, values) in config.categories.clone().into_iter() {
         let category_names_string = format!("Analyze the path and content of the object below and label it with a {} value from the list", category_name);
         categories_string = categories_string + &category_names_string;

         categories_string = categories_string + &format!("\n\n{}: [\n", category_name);

        for category in values {
            categories_string = categories_string + "\"" + &category + "\",\n"
        }
        categories_string = categories_string + "]\n\n";
    }

    let mut object_string: String = "Object: {\n".to_string();

    // path
    object_string.push_str("  path: \"");
    object_string.push_str(&file.path);
    object_string.push_str("\",\n");

    // println!("{}", &file.content);

    // content
    object_string.push_str("  content: \"");
    object_string.push_str(&file.content);
    object_string.push_str("\",\n");

    // end of object
    object_string.push_str("}\n");

    let mut prompt: String = "".to_string();

    prompt.push_str(&categories_string);
    prompt.push_str("####\n");
    prompt.push_str(&object_string);
    prompt.push_str("####\n\n");
    prompt.push_str("Use known values from the lists above when assigning labels if a label cannot be determined assign Unknown. Return a JSON string with values for ");
    prompt.push_str(&categories);
    prompt.push_str(" and path.");

    // println!("{}", prompt);

    return prompt
}

fn get_config(config_file: &String) -> Config {
    // TODO Remove DATA_SET_DIRECTORY!! This is just for easy testing
    let file = File::open(DATA_SET_DIRECTORY.to_owned() + "/" + config_file).expect(&format!("Missing Config {}", config_file));
    let config: Config = serde_yaml::from_reader(file).expect("Could not marshall config.");

    return config
}

async fn categorize_file(client: &Client, config: &Config, file: Object) -> serde_json::Value {
    let prompt = prompt(config, file);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        // TODO Allow CLI arg for model; support multi-modal for GPT-4
        .model("gpt-4")
        .temperature(0.0)
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(prompt)
                .build()
                .unwrap(),
        ])
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    let answer = response.choices.first().unwrap();

    // Ensure the respnose is proper json
    // TODO add error handling and nice retry semantics maybe try altering the prompt
    return serde_json::from_str(&answer.message.content).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let client = Client::new();

    // Use parsed args
    let excluded_file_types = args.exclude_file_type;
    let max_document_length = args.max_read_length;

    let config: Config = get_config(&args.config_file);

    let files_in_dirs = WalkDir::new(DATA_SET_DIRECTORY).into_iter().filter_map(|e| e.ok());

    // TODO parallelize API calls
    for file in files_in_dirs {
        let name = file.path().display().to_string();
        let extension = name.split(".").into_iter().last().unwrap();

        let mut file_content: String = "".to_string();

        if excluded_file_types.contains(&extension.to_string()) || file.file_type().is_dir(){
            continue;
        }

        // TODO; extract logic for different types of files and how they're read - add support for
        // other modalities with GPT-4. Fix bug with empty content with pdf-extract
        if name.ends_with(".pdf") {
          match pdf_extract::extract_text(&name) {
              Ok(result) => {
                  // Remove random whitespace
                  let re = Regex::new(r"\s+").unwrap();
                  file_content = re.replace_all(&result, " ").to_string();

                  if file_content.len() > max_document_length {
                      file_content = file_content[..max_document_length].to_string();
                  }

              },
              Err(_e) => println!("Error reading {}", name)
          }
        }

        let value = categorize_file(
            &client,
            &config,
            Object{
                path: name.to_string(),
                content: file_content
            }
        ).await;

        println!("{}", serde_json::to_string(&value)?);
    }
    Ok(())
}
