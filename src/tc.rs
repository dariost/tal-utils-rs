use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::iter::Iterator;
use std::time::Instant;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn fetch_env(name: &str) -> Result<String> {
    env::var(name).map_err(|e| format!("Cannot get environment variable {}: {}", name, e).into())
}

pub fn run_tc<I, G, C, T, U, S>(timeout: f64, init_fn: I, gen_fn: G, check_fn: C) -> Result<()>
where
    S: IntoIterator<Item = T>,
    I: FnOnce(&str) -> Result<S>,
    G: Fn(T) -> Result<U>,
    C: Fn(U) -> Result<(bool, Option<String>)>,
{
    let subtask = fetch_env("TAL_size")?;
    let output_dir = fetch_env("TAL_META_OUTPUT_FILES")?;
    let mut fout = File::create(format!("{output_dir}/result.txt"))?;
    let mut tc_ok = 0;
    let mut tc_n = 0;
    let iter = init_fn(&subtask)?.into_iter();
    let total_tc_n = match iter.size_hint() {
        (n, Some(m)) if n == m => n,
        _ => return Err("Cannot get the number of test cases".into()),
    };
    println!("{}", total_tc_n);
    stdout().flush()?;
    for tc_param in iter {
        tc_n += 1;
        let tc = gen_fn(tc_param)?;
        stdout().flush()?;
        let start = Instant::now();
        let (ok, msg) = match check_fn(tc) {
            Ok(x) => x,
            Err(e) => {
                writeln!(fout, "Case #{tc_n:03}: RE")?;
                eprintln!("Check error: {}", e);
                continue;
            }
        };
        if Instant::now().duration_since(start).as_secs_f64() > timeout {
            writeln!(fout, "Case #{tc_n:03}: TLE")?;
        } else if ok {
            writeln!(fout, "Case #{tc_n:03}: AC")?;
            tc_ok += 1;
        } else {
            writeln!(fout, "Case #{tc_n:03}: WA")?;
        }
        if let Some(msg) = msg {
            writeln!(fout)?;
            writeln!(fout, "{}", msg)?;
            writeln!(fout)?;
        }
    }
    writeln!(fout)?;
    writeln!(fout, "Score: {}/{}", tc_ok, tc_n)?;
    Ok(())
}
