use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use duration_string::DurationString;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    fs::{remove_dir_all, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

fn duration_parser(s: &str) -> Result<Duration> {
    Ok(Duration::from_std(
        DurationString::from_str(s)
            .map_err(|e| anyhow!("{e:?}"))?
            .into(),
    )?)
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    datadir: PathBuf,
    #[arg(long = "unit")]
    units: Vec<String>,
    #[arg(long, value_parser=duration_parser)]
    time_window: Duration,
}

lazy_static! {
    static ref GAS_RE: Regex = Regex::new(
        r#"^\[(?P<date>[^ ]*) ERROR ya_erc20_driver::erc20::wallet\] Error sending transaction: GenericError \{ inner: "RPC error: Error \{ code: ServerError\(-32000\), message: \\"insufficient funds for gas \* price \+ value\\", data: None \}" \}$"#,
    ).unwrap();
}

fn is_situation_bad(datadir: &Path, time_window: Duration) -> Result<bool> {
    let time_limit = Utc::now() - time_window;
    Ok(
        BufReader::new(File::open(datadir.join("yagna_rCURRENT.log"))?)
            .lines()
            .any(|line| {
                (|| -> Result<bool> {
                    let line = line?;
                    // If regex doesn't match it's not an error.
                    let caps = match GAS_RE.captures(&line) {
                        Some(caps) => caps,
                        None => return Ok(false),
                    };
                    let date = caps["date"].parse::<DateTime<Utc>>().with_context(|| {
                        format!("Failed to parse date from \"{}\"", &caps["date"])
                    })?;
                    Ok(date >= time_limit)
                })()
                .map_or_else(
                    |e| {
                        eprintln!("Parsing line failed: {e:?}");
                        false
                    },
                    |v| v,
                )
            }),
    )
}

fn is_unit_active(unit: &str) -> Result<bool> {
    Ok(Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", unit])
        .status()
        .context("Failed to execute subcommand")?
        .success())
}

fn systemctl(command: &str, unit: &str) -> Result<()> {
    if !Command::new("systemctl")
        .args(["--user", command, unit])
        .status()
        .context("Failed to execute subcommand")?
        .success()
    {
        bail!("failed to {command} \"{unit}\"");
    }
    Ok(())
}

fn stop_units(units: Vec<String>) -> Result<Vec<String>> {
    let mut units_to_restart_later: Vec<String> = Vec::new();

    for unit in units {
        if let Some(name) = unit.strip_suffix(".timer") {
            let service = format!("{name}.service");
            println!("Stopping \"{service}\"");
            systemctl("stop", &service)?;
        }

        if is_unit_active(&unit)? {
            println!("Stopping \"{unit}\"");
            systemctl("stop", &unit)?;
            units_to_restart_later.push(unit);
        } else {
            println!("Unit \"{unit}\" already stopped");
        }
    }

    Ok(units_to_restart_later)
}

fn start_units(units: Vec<String>) -> Result<()> {
    for unit in units {
        println!("Starting \"{unit}\"");
        systemctl("start", &unit)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if !is_situation_bad(&cli.datadir, cli.time_window)? {
        println!("Nothing to do");
        return Ok(());
    }

    println!("Stopping units");
    let units_to_restart = stop_units(cli.units)?;

    println!("Wiping datadir");
    remove_dir_all(&cli.datadir)?;

    println!("Restarting units");
    start_units(units_to_restart)?;

    Ok(())
}
