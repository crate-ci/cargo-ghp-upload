// Copyright 2018 CAD97 <cad97@cad97.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![forbid(future_incompatible)]
#![warn(warnings)]

extern crate quicli;
use quicli::prelude::*;

extern crate fs_extra;

use std::{env, fs, str};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Debug, StructOpt)]
#[structopt(raw(bin_name = r#""cargo""#))]
enum CargoArgs {
    #[structopt(name = "ghp-upload")] GhpUpload(Args),
}

/// Upload documentation straight to GitHub Pages while maintaining branch separation and history
#[derive(Debug, StructOpt)]
struct Args {
    /// The branch used for GitHub Pages
    #[structopt(long = "deploy", default_value = "gh-pages")]
    deploy_branch: String,
    /// Branches to publish
    #[structopt(long = "branch", default_value = "master")]
    publish_branch: Vec<String>,
    /// Publish documentation for tag builds (GitHub releases)
    #[structopt(short = "r", long = "publish-tags")]
    publish_tags: bool,

    /// GitHub Personal Access token
    #[structopt(long = "token", help = "GitHub Personal Access token [default: $GH_TOKEN]")]
    token: Option<String>,
    /// Message for the git commit
    #[structopt(long = "message", default_value = "ghp-upload script")]
    message: String,
    /// The directory to publish the files from
    #[structopt(long = "directory", parse(from_os_str), default_value = "./target/doc")]
    upload_directory: PathBuf,

    #[structopt(long = "remove-index", help = "Remove `branch/index.html` if it exists")]
    clobber_index: bool,

    #[structopt(long = "verbose", short = "v", parse(from_occurrences),
                help = "Enable more verbose logging [repeatable (max 4)]")]
    verbosity: u8,
}

#[derive(Debug, Default)]
struct Context {
    branch: Option<String>,
    tag: Option<String>,
    origin: Option<String>,
    pull_request: bool,
}

fn get_context(args: &Args) -> Result<Context> {
    let mut context = Context::default();

    if env::var_os("CI").is_some() {
        info!("CI detected; running CI-exclusive features");

        if env::var_os("TRAVIS").is_some() {
            info!("Travis CI detected");
            if args.token.is_some() && env::var("TRAVIS_SECURE_ENV_VARS")? == "false" {
                // $TRAVIS_SECURE_ENV_VARS == "false" when no secure variables are available
                // e.g. during a pull request (untrusted code) or when none are set (maybe)
                // This state _should_ unequivocally mean that the secret can be stolen
                error!("***************************************************");
                error!("* WARNING * WARNING * WARNING * WARNING * WARNING *");
                error!("*                                                 *");
                error!("*   GitHub Token found but is likely NOT SECURE   *");
                error!("*    A GitHub Token in plain text in your repo    *");
                error!("*      or in Travis ENV without being hidden      *");
                error!("*  should be considered compromised and replaced  *");
                error!("*                                                 *");
                error!("* WARNING * WARNING * WARNING * WARNING * WARNING *");
                error!("***************************************************");
                error!("(If this is a false positive, open an issue for us)");
                bail!("Insecure environment found; stopping");
            }
            let tag = env::var("TRAVIS_TAG")?;
            if !tag.is_empty() {
                context.tag = Some(tag);
            }
            context.branch = Some(env::var("TRAVIS_BRANCH")?);
            context.pull_request = env::var("TRAVIS_PULL_REQUEST")? != "false";
            let repo_slug = env::var("TRAVIS_REPO_SLUG")?;
            context.origin = Some(if let Some(ref token) = args.token {
                format!("https://{}@github.com/{}.git", token, repo_slug)
            } else {
                warn!("No GitHub Personal Access Token was provided");
                warn!("Falling back to using the SSH endpoint");
                format!("git@github.com:{}.git", repo_slug)
            })
        } else {
            warn!("Unsupported CI detected; no CI features were run")
        }
    } else {
        info!("No CI detected; collecting relevant information from Git")
    }

    context.branch = context.branch.or_else(|| {
        let abbrev_ref = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()?
            .stdout;
        Some(
            str::from_utf8(&abbrev_ref)
                .expect("Invalid UTF8 from Git")
                .trim()
                .to_owned(),
        )
    });

    context.origin = context.origin.or_else(|| {
        if let Ok(upstream) = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD@{upstream}"])
            .output()
        {
            let upstream: &str = str::from_utf8(&upstream.stdout)
                .expect("Invalid UTF8 from Git")
                .split('/')
                .next()
                .unwrap();
            let origin = Command::new("git")
                .args(&["remote", "get-url"])
                .arg(upstream)
                .output()
                .ok()?
                .stdout;
            let origin = str::from_utf8(&origin)
                .expect("Invalid UTF8 from Git")
                .trim();
            if let Some(ref token) = args.token {
                let github_loc = origin.find("github.com").expect("Non-GitHub remote");
                let dot_git_loc = origin.rfind(".git").unwrap();
                let repo_slug = &origin[(github_loc + "github.com".len() + 1)..dot_git_loc];
                Some(format!("https://{}@github.com/{}.git", token, repo_slug))
            } else {
                Some(origin.to_owned())
            }
        } else {
            None
        }
    });

    Ok(context)
}

fn require_success(status: ExitStatus) -> Result<()> {
    if status.success() {
        Ok(())
    } else {
        bail!("Child process failed: {}", status)
    }
}

fn ghp_upload(branch: &str, origin: &str, args: &Args) -> Result<()> {
    let ghp_dir = Path::new("target/ghp");
    if ghp_dir.exists() {
        // If the directory exists, make sure it's up to date
        require_success(Command::new("git")
            .current_dir(ghp_dir)
            .arg("pull")
            .status()?)?;
    } else {
        // If the folder doesn't exist yet, clone it from remote
        // ASSUME: if target/ghp exists, it's ours
        let status = Command::new("git")
            .args(&["clone", "--verbose"])
            .args(&["--branch", &args.deploy_branch])
            .args(&["--depth", "1"])
            .args(&[origin, ghp_dir.to_str().unwrap()])
            .status()?;
        if !status.success() {
            // If clone fails, the remote doesn't exist
            // So create a new repository to hold the documentation branch
            require_success(Command::new("git").arg("init").arg(ghp_dir).status()?)?;
            require_success(Command::new("git")
                .current_dir(ghp_dir)
                .arg("checkout")
                .args(&["-b", &args.deploy_branch])
                .status()?)?;
        }
    }

    let ghp_branch_dir = ghp_dir.join(branch);
    fs::create_dir(&ghp_branch_dir).ok(); // Create dir if not exists
    for entry in ghp_branch_dir.read_dir()? {
        let dir = entry?;
        // Clean the directory, as we'll be copying new files
        // Ignore index.html as requested for redirect page
        if args.clobber_index || dir.file_name() != OsString::from("index.hmtl") {
            let path = dir.path();
            fs::remove_dir_all(&path).ok();
            fs::remove_file(path).ok();
        }
    }

    let upload_dir = PathBuf::from(&args.upload_directory);
    eprintln!("Copying documentation...");
    let mut last_progress = 0;
    fs_extra::copy_items_with_progress(
        &upload_dir
            .read_dir()?
            .map(|entry| entry.unwrap().path())
            .collect(),
        ghp_branch_dir,
        &fs_extra::dir::CopyOptions::new(),
        |info| {
            // Some documentation can be very large, especially with a large number of dependencies
            // Don't go silent, give updates
            if info.copied_bytes >> 20 > last_progress {
                last_progress = info.copied_bytes >> 20;
                eprintln!(
                    "~ {}/{} MiB",
                    info.copied_bytes >> 20,
                    info.total_bytes >> 20
                );
            }
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        },
    )?;

    // Track all changes
    require_success(Command::new("git")
        .current_dir(ghp_dir)
        .args(&["add", "--verbose", "--all"])
        .status()?)?;

    // Save changes
    // No changes fails, expected behavior
    let commit_status = Command::new("git")
        .current_dir(ghp_dir)
        .args(&["commit", "--verbose"])
        .args(&["-m", &args.message])
        .status()?;

    if commit_status.success() {
        require_success(Command::new("git")
            .current_dir(ghp_dir)
            .args(&["push", origin, &args.deploy_branch])
            .status()?)?;
        println!("Successfully updated documentation.");
    } else {
        println!("Documentation already up-to-date.");
    }

    Ok(())
}

fn run() -> Result<()> {
    let CargoArgs::GhpUpload(args) = CargoArgs::from_args();
    let args = Args {
        token: args.token.or_else(|| env::var("GH_TOKEN").ok()),
        ..args
    };
    println!("{}", args.clobber_index);
    bail!("");

    LoggerBuiler::new()
        .filter(
            None,
            match args.verbosity {
                0 => LogLevel::Error,
                1 => LogLevel::Warn,
                2 => LogLevel::Info,
                3 => LogLevel::Debug,
                _ => LogLevel::Trace,
            }.to_level_filter(),
        )
        .try_init()?;

    debug!("Args");
    debug!("  deploy branch: {}", args.deploy_branch);
    debug!("  publish branches: {:?}", args.publish_branch);
    debug!(
        "  token: {}",
        if args.token.is_none() {
            "None"
        } else {
            "[REDACTED]"
        }
    );
    debug!("  message: {}", args.message);
    debug!("  upload directory: {:?}", args.upload_directory);
    debug!("  clobber index: {}", args.clobber_index);
    debug!("  verbosity: {}", args.verbosity);

    let context = get_context(&args)?;

    debug!("Context");
    debug!("  branch: {:?}", context.branch);
    debug!("  tag: {:?}", context.tag);
    debug!(
        "  origin: {:?}",
        context
            .origin
            .as_ref()
            .map(|it| if let Some(ref token) = args.token {
                it.replace(token, "[REDACTED]")
            } else {
                it.clone()
            })
    );
    debug!("  pull request: {}", context.pull_request);

    let branch = context
        .branch
        .as_ref()
        .ok_or_else(|| err_msg("No branch/tag determined"))?;
    let origin = context
        .origin
        .as_ref()
        .ok_or_else(|| err_msg("No origin determined"))?;

    if context.pull_request {
        eprintln!("Skipping Pull Request build");
    } else if (args.publish_tags && context.tag.is_some()) || args.publish_branch.contains(branch) {
        ghp_upload(branch, origin, &args)?;
    } else {
        eprintln!("Skipping build; not configured to build {}", branch)
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e);
            ::std::process::exit(1);
        }
    }
}
