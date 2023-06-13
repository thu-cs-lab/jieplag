use dotenv::dotenv;
use std::path::PathBuf;
use structopt::StructOpt;

use api::{
    env::ENV,
    def::{LoginRequest, SubmitRequest, Submission}
};
use core::lang::Language;


#[derive(StructOpt)]
struct Args {
    /// User name
    #[structopt(short, long)]
    user_name: String,

    /// Password
    #[structopt(short, long)]
    password: String,

    /// Language
    #[structopt(short, long)]
    language: String,

    /// Path to template file
    #[structopt(short = "b", long)]
    template: PathBuf,

    /// Paths to source code
    code: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let opts = Args::from_args();
    env_logger::init();

    let client = reqwest::blocking::Client::new();
    let body = client
        .post(format!("{}/api/submit", ENV.public_url))
        .json(&SubmitRequest {
            login: Some(LoginRequest {
                user_name: opts.user_name,
                password: opts.password,
            }),
            language: match opts.language.as_str() {
                "c++" | "cpp" | "cc" => Language::Cpp,
                "rust" => Language::Rust,
                _ => unimplemented!("Language: {}", opts.language),
            },
            template: Some(std::fs::read_to_string(&opts.template)?),
            submissions: opts
                .code
                .iter()
                .map(|code| Submission {
                    name: format!("{}", code.display()),
                    code: std::fs::read_to_string(&code).unwrap(),
                })
                .collect::<Vec<_>>(),
        })
        .send()?
        .text()?;
    println!("{}", body);

    Ok(())
}
