use jieplag::{
    lang::Language,
    session::LoginRequest,
    submit::{Submission, SubmitRequest},
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    /// User name
    #[structopt(short, long)]
    user_name: String,

    /// Password
    #[structopt(short, long)]
    password: String,

    /// Path to template file
    #[structopt(short = "b", long)]
    template: PathBuf,

    /// Paths to source code
    code: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opts = Args::from_args();
    env_logger::init();

    let client = reqwest::blocking::Client::new();
    let body = client
        .post("http://localhost:8765/api/submit")
        .json(&SubmitRequest {
            login: Some(LoginRequest {
                user_name: opts.user_name,
                password: opts.password,
            }),
            language: Language::Cpp,
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
    println!("{:?}", body);

    Ok(())
}
