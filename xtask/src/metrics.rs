use std::{
    collections::BTreeMap,
    env,
    fmt::{self, Write as _},
    io::Write as _,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, format_err, Result};

use crate::not_bash::{fs2, pushd, pushenv, rm_rf, run};

type Unit = String;

pub struct MetricsCmd {
    pub dry_run: bool,
}

impl MetricsCmd {
    pub fn run(self) -> Result<()> {
        let mut metrics = Metrics::new()?;
        if !self.dry_run {
            rm_rf("./target/release")?;
        }
        if !Path::new("./target/rustc-perf").exists() {
            fs2::create_dir_all("./target/rustc-perf")?;
            run!("git clone https://github.com/rust-lang/rustc-perf.git ./target/rustc-perf")?;
        }
        {
            let _d = pushd("./target/rustc-perf");
            run!("git reset --hard 1d9288b0da7febf2599917da1b57dc241a1af033")?;
        }

        let _env = pushenv("RA_METRICS", "1");

        metrics.measure_build()?;
        metrics.measure_analysis_stats_self()?;
        metrics.measure_analysis_stats("ripgrep")?;
        metrics.measure_analysis_stats("webrender")?;

        if !self.dry_run {
            let _d = pushd("target");
            let metrics_token = env::var("METRICS_TOKEN").unwrap();
            let repo = format!("https://{}@github.com/rust-analyzer/metrics.git", metrics_token);
            run!("git clone --depth 1 {}", repo)?;
            let _d = pushd("metrics");

            let mut file = std::fs::OpenOptions::new().append(true).open("metrics.json")?;
            writeln!(file, "{}", metrics.json())?;
            run!("git add .")?;
            run!("git -c user.name=Bot -c user.email=dummy@example.com commit --message 📈")?;
            run!("git push origin master")?;
        }
        eprintln!("{:#?}", metrics);
        Ok(())
    }
}

impl Metrics {
    fn measure_build(&mut self) -> Result<()> {
        eprintln!("\nMeasuring build");
        run!("cargo fetch")?;

        let time = Instant::now();
        run!("cargo build --release --package rust-analyzer --bin rust-analyzer")?;
        let time = time.elapsed();
        self.report("build", time.as_millis() as u64, "ms".into());
        Ok(())
    }
    fn measure_analysis_stats_self(&mut self) -> Result<()> {
        self.measure_analysis_stats_path("self", &".")
    }
    fn measure_analysis_stats(&mut self, bench: &str) -> Result<()> {
        self.measure_analysis_stats_path(
            bench,
            &format!("./target/rustc-perf/collector/benchmarks/{}", bench),
        )
    }
    fn measure_analysis_stats_path(&mut self, name: &str, path: &str) -> Result<()> {
        eprintln!("\nMeasuring analysis-stats/{}", name);
        let output = run!("./target/release/rust-analyzer analysis-stats --quiet {}", path)?;
        for (metric, value, unit) in parse_metrics(&output) {
            self.report(&format!("analysis-stats/{}/{}", name, metric), value, unit.into());
        }
        Ok(())
    }
}

fn parse_metrics(output: &str) -> Vec<(&str, u64, &str)> {
    output
        .lines()
        .filter_map(|it| {
            let entry = it.split(':').collect::<Vec<_>>();
            match entry.as_slice() {
                ["METRIC", name, value, unit] => Some((*name, value.parse().unwrap(), *unit)),
                _ => None,
            }
        })
        .collect()
}

#[derive(Debug)]
struct Metrics {
    host: Host,
    timestamp: SystemTime,
    revision: String,
    metrics: BTreeMap<String, (u64, Unit)>,
}

#[derive(Debug)]
struct Host {
    os: String,
    cpu: String,
    mem: String,
}

impl Metrics {
    fn new() -> Result<Metrics> {
        let host = Host::new()?;
        let timestamp = SystemTime::now();
        let revision = run!("git rev-parse HEAD")?;
        Ok(Metrics { host, timestamp, revision, metrics: BTreeMap::new() })
    }

    fn report(&mut self, name: &str, value: u64, unit: Unit) {
        self.metrics.insert(name.into(), (value, unit));
    }

    fn json(&self) -> Json {
        let mut json = Json::default();
        self.to_json(&mut json);
        json
    }
    fn to_json(&self, json: &mut Json) {
        json.begin_object();
        {
            json.field("host");
            self.host.to_json(json);

            json.field("timestamp");
            let timestamp = self.timestamp.duration_since(UNIX_EPOCH).unwrap();
            json.number(timestamp.as_secs() as f64);

            json.field("revision");
            json.string(&self.revision);

            json.field("metrics");
            json.begin_object();
            {
                for (k, (value, unit)) in &self.metrics {
                    json.field(k);
                    json.begin_array();
                    {
                        json.number(*value as f64);
                        json.string(unit);
                    }
                    json.end_array();
                }
            }
            json.end_object()
        }
        json.end_object();
    }
}

impl Host {
    fn new() -> Result<Host> {
        if cfg!(not(target_os = "linux")) {
            bail!("can only collect metrics on Linux ");
        }

        let os = read_field("/etc/os-release", "PRETTY_NAME=")?.trim_matches('"').to_string();

        let cpu =
            read_field("/proc/cpuinfo", "model name")?.trim_start_matches(':').trim().to_string();

        let mem = read_field("/proc/meminfo", "MemTotal:")?;

        return Ok(Host { os, cpu, mem });

        fn read_field<'a>(path: &str, field: &str) -> Result<String> {
            let text = fs2::read_to_string(path)?;

            let line = text
                .lines()
                .find(|it| it.starts_with(field))
                .ok_or_else(|| format_err!("can't parse {}", path))?;
            Ok(line[field.len()..].trim().to_string())
        }
    }
    fn to_json(&self, json: &mut Json) {
        json.begin_object();
        {
            json.field("os");
            json.string(&self.os);

            json.field("cpu");
            json.string(&self.cpu);

            json.field("mem");
            json.string(&self.mem);
        }
        json.end_object();
    }
}

struct State {
    obj: bool,
    first: bool,
}

#[derive(Default)]
struct Json {
    stack: Vec<State>,
    buf: String,
}

impl Json {
    fn begin_object(&mut self) {
        self.stack.push(State { obj: true, first: true });
        self.buf.push('{');
    }
    fn end_object(&mut self) {
        self.stack.pop();
        self.buf.push('}')
    }
    fn begin_array(&mut self) {
        self.stack.push(State { obj: false, first: true });
        self.buf.push('[');
    }
    fn end_array(&mut self) {
        self.stack.pop();
        self.buf.push(']')
    }
    fn field(&mut self, name: &str) {
        self.object_comma();
        self.string_token(name);
        self.buf.push(':');
    }
    fn string(&mut self, value: &str) {
        self.array_comma();
        self.string_token(value);
    }
    fn string_token(&mut self, value: &str) {
        self.buf.push('"');
        self.buf.extend(value.escape_default());
        self.buf.push('"');
    }
    fn number(&mut self, value: f64) {
        self.array_comma();
        write!(self.buf, "{}", value).unwrap();
    }

    fn array_comma(&mut self) {
        let state = self.stack.last_mut().unwrap();
        if state.obj {
            return;
        }
        if !state.first {
            self.buf.push(',');
        }
        state.first = false;
    }

    fn object_comma(&mut self) {
        let state = self.stack.last_mut().unwrap();
        if !state.first {
            self.buf.push(',');
        }
        state.first = false;
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.buf)
    }
}
