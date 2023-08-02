use api::{
    def::{LoginRequest, Submission, SubmitRequest},
    env::ENV,
};
use core::lang::Language;
use dotenv::dotenv;
use std::{ffi::OsString, path::PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

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
    template: Option<PathBuf>,

    /// Paths to source code
    code: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let opts = Args::from_args();
    env_logger::init();

    let language = match opts.language.as_str() {
        "c++" | "cpp" | "cc" => Language::Cpp,
        "python" | "py" => Language::Python,
        "rust" => Language::Rust,
        _ => unimplemented!("Language: {}", opts.language),
    };

    let extensions = match &language {
        Language::Cpp => ["cpp", "h"].to_vec(),
        Language::Rust => ["rs"].to_vec(),
        Language::Python => ["py"].to_vec(),
        Language::Verilog => ["v"].to_vec(),
    };

    let comment = match &language {
        Language::Cpp => "//",
        Language::Rust => "//",
        Language::Python => "#",
        Language::Verilog => "//",
    };

    let client = reqwest::blocking::Client::new();
    let template = match &opts.template {
        Some(template) => Some(std::fs::read_to_string(template)?),
        None => None,
    };
    let body = client
        .post(format!("{}/api/submit", ENV.public_url))
        .json(&SubmitRequest {
            login: Some(LoginRequest {
                user_name: opts.user_name,
                password: opts.password,
            }),
            language,
            template,
            submissions: opts
                .code
                .iter()
                .map(|code| {
                    if std::path::Path::new(code).is_file() {
                        // one file
                        Submission {
                            name: format!("{}", code.display()),
                            code: std::fs::read_to_string(&code).unwrap(),
                        }
                    } else {
                        // find all sources and concat
                        let mut source_code = String::new();
                        for entry in WalkDir::new(code).into_iter().filter_map(|e| e.ok()) {
                            for ext in &extensions {
                                if entry.path().extension() == Some(&OsString::from(ext)) {
                                    source_code +=
                                        &format!("{} {} \n", comment, entry.path().display());
                                    source_code += &std::fs::read_to_string(&entry.path()).unwrap();
                                    break;
                                }
                            }
                        }
                        Submission {
                            name: format!("{}", code.display()),
                            code: source_code,
                        }
                    }
                })
                .collect::<Vec<_>>(),
        })
        .send()?
        .text()?;
    println!("{}", body);

    Ok(())
}
