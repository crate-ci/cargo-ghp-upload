#![forbid(future_incompatible)]
#![warn(warnings)]

#[macro_use]
extern crate quicli;
use quicli::prelude::*;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(long = "deploy", help = "Deploy to the given branch", default_value = "gh-pages")]
    deploy_branch: String,
    #[structopt(long = "branch", help = "Publish for this branch", default_value = "master")]
    publish_branch: Vec<String>,

    #[structopt(long = "token", help = "GitHub token to use [default: $GH_TOKEN]")]
    token: Option<String>,
    #[structopt(long = "message", help = "Use this message for the git commit",
                default_value = "ghp-upload script")]
    message: String,
    #[structopt(long = "directory", help = "The directory to upload from", parse(from_os_str),
                default_value = "./target/doc")]
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
            context.tag = env::var("TRAVIS_TAG").ok();
            if context.tag.is_none() {
                context.branch = Some(env::var("TRAVIS_BRANCH")?);
            }
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

fn ghp_upload(_args: Args, _context: Context) -> Result<()> {
    unimplemented!()
}

main!(|args: Args, log_level: verbosity| {
    let args = Args {
        token: args.token.or(env::var("GH_TOKEN").ok()),
        ..args
    };
    debug!("Args");
    debug!("  deploy branch: {}", args.deploy_branch);
    debug!("  publish branches: {:?}", args.publish_branch);
    debug!("  token: {}", if args.token.is_none() { "None" } else { "[REDACTED]" });
    debug!("  message: {}", args.message);
    debug!("  upload directory: {:?}", args.upload_directory);
    debug!("  clobber index: {}", args.clobber_index);
    debug!("  verbosity: {}", args.verbosity);
    let context = get_context(&args)?;
    debug!("Context");
    debug!("  branch: {:?}", context.branch);
    debug!("  tag: {:?}", context.tag);
    debug!("  origin: {:?}", context.origin.as_ref().map(|it|
        if let Some(ref token) = args.token { it.replace(token, "[REDACTED]") } else { it.clone() }
    ));
    debug!("  pull request: {}", context.pull_request);
    ghp_upload(args, context)?;
});
