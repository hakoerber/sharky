use sharks::{Share, Sharks};
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str;

const N: u8 = 6;
const K: u8 = 3;

struct MainError(String);

impl Error for MainError {}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for MainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for MainError {
    fn from(error: io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<&str> for MainError {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for MainError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

fn main() -> Result<(), MainError> {
    let mut args = env::args().into_iter().skip(1);

    let action = args.next().ok_or("did not find action parameter")?;

    let secret_path = PathBuf::from(
        &args
            .next()
            .ok_or("did not find secret path as first argument")?,
    );

    match action.as_str() {
        "share" => {
            let sharks = Sharks(K);

            let dealer = sharks.dealer(
                &fs::read(&secret_path).map_err(|e| format!("{}: {e}", secret_path.display()))?,
            );

            let shares: Vec<Share> = dealer.take(6).collect();

            let args: Vec<_> = args.collect::<Vec<_>>();
            if args.len() != N.into() {
                return Err(
                    format!("share length mismatch: expected {N}, got {}", args.len()).into(),
                );
            }

            for (i, share) in shares.into_iter().enumerate() {
                fs::write(&args[i], Vec::from(&share)).map_err(|e| format!("{}: {e}", &args[i]))?;
                println!("Written share {} to {}", i + 1, args[i]);
            }
        }
        "recover" => {
            // TOCTOU, waiting for nightly feature "file_create_new"
            if secret_path.exists() {
                return Err(format!(
                    "output file \"{}\" already exists, refusing",
                    secret_path.display()
                )
                .into());
            }
            let share_paths: Vec<String> = args.collect();

            if share_paths.len() > N.into() {
                return Err(format!(
                    "too many shares: expected a maximum of {N}, got {}",
                    share_paths.len()
                )
                .into());
            }

            if share_paths.len() < K.into() {
                return Err(format!(
                    "too few shares: need at least {K}, got {}",
                    share_paths.len()
                )
                .into());
            }

            let shares: Vec<Share> = share_paths
                .into_iter()
                .map(|path| -> Result<Share, MainError> {
                    Share::try_from(
                        fs::read(&path)
                            .map_err(|e| format!("{path}: {e}"))?
                            .as_slice(),
                    )
                    .map_err(|e| e.into())
                })
                .collect::<Result<_, _>>()?;

            let recovered = Sharks(K).recover(&shares)?;

            fs::write(&secret_path, recovered)
                .map_err(|e| format!("{}: {e}", secret_path.display()))?;

            println!(
                "secret recovered into {} from {} shares",
                &secret_path.display(),
                shares.len()
            );
        }
        _ => return Err(format!("unknown action \"{action}\"").into()),
    }

    Ok(())
}
