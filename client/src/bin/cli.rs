use api::{
    def::{LoginRequest, Submission, SubmitRequest},
    env::ENV,
};
use clap::Parser;
use core::lang::Language;
use dotenv::dotenv;
use encoding::{DecoderTrap, Encoding};
use log::{info, warn};
use std::{ffi::OsString, path::Path, path::PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
struct Args {
    /// User name
    #[arg(short, long)]
    user_name: String,

    /// Password
    #[arg(short, long)]
    password: String,

    /// Language
    #[arg(short, long)]
    language: String,

    /// Path to template file
    #[arg(short = 'b', long)]
    template: Option<PathBuf>,

    /// Paths to source code
    code: Vec<PathBuf>,
}

fn collect(language: &Language, path: &Path) -> String {
    let comment = match &language {
        Language::Cpp => "//",
        Language::Rust => "//",
        Language::Python => "#",
        Language::Verilog => "//",
        Language::SQL => "--",
        Language::JavaScript => "//",
        Language::Lua => "--",
    };
    let extensions = match &language {
        Language::Cpp => ["cpp", "h"].to_vec(),
        Language::Rust => ["rs"].to_vec(),
        Language::Python => ["py"].to_vec(),
        Language::Verilog => ["v"].to_vec(),
        Language::SQL => ["sql"].to_vec(),
        Language::JavaScript => ["js"].to_vec(),
        Language::Lua => ["lua"].to_vec(),
    };

    if std::path::Path::new(path).is_file() {
        // one file
        std::fs::read_to_string(path).unwrap()
    } else {
        // find all sources and concat
        let mut source_code = String::new();
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            for ext in &extensions {
                if entry.path().extension() == Some(&OsString::from(ext)) {
                    source_code += &format!("{} {} \n", comment, entry.path().display());
                    let content = std::fs::read(entry.path()).unwrap();
                    match String::from_utf8(content.clone()) {
                        Ok(content) => {
                            source_code += &content;
                        }
                        Err(err) => {
                            warn!(
                                "Failed to parse utf8 for {}: {}, trying to decoding as GB18030",
                                entry.path().display(),
                                err
                            );

                            match encoding::all::GB18030.decode(&content, DecoderTrap::Strict) {
                                Ok(content) => {
                                    source_code += &content;
                                    info!(
                                        "Succeeded to parse gb18030 for {}: {}",
                                        entry.path().display(),
                                        err
                                    );
                                }
                                Err(err) => {
                                    warn!(
                                        "Failed to parse gb18030 for {}: {}",
                                        entry.path().display(),
                                        err
                                    );
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        source_code
    }
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let opts = Args::parse();
    env_logger::init();

    let language = match opts.language.as_str() {
        "c++" | "cpp" | "cc" => Language::Cpp,
        "python" | "py" => Language::Python,
        "rust" => Language::Rust,
        _ => unimplemented!("Language: {}", opts.language),
    };

    let client = reqwest::blocking::Client::new();
    let template = opts
        .template
        .as_ref()
        .map(|template| collect(&language, template));
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
                .map(|code| Submission {
                    name: format!("{}", code.display()),
                    code: collect(&language, code),
                })
                .collect::<Vec<_>>(),
        })
        .send()?
        .text()?;
    println!("{}", body);

    Ok(())
}
