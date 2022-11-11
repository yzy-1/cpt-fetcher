use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  error::Error,
  fs::{write, OpenOptions},
  path::PathBuf,
  process::exit,
};
use strfmt::Format;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase {
  input: String,
  output: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CCResponse {
  name: String,
  group: String,
  url: String,
  memory_limit: i64,
  time_limit: i64,
  tests: Vec<TestCase>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CPTProblemMeta {
  memory_limit: String,
  time_limit: String,
  test: CPTProblemMetaTest,
}

#[derive(Debug, Deserialize, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct CPTProblemMetaTest {
  input: Vec<String>,
  output: Vec<String>,
}

lazy_static! {
  static ref PORT: i32 = 10043;
  static ref INF_NAME: String = "{i}.in".to_string();
  static ref ANS_NAME: String = "{i}.ans".to_string();
}

fn main() {
  web_server::new().post("/", Box::new(handle)).launch(*PORT);
}

fn convert_meta(resp: &CCResponse) -> CPTProblemMeta {
  let mut tests = CPTProblemMetaTest::default();
  for i in 1..=resp.tests.len() {
    tests.input.push(
      INF_NAME
        .format(&HashMap::from([("i".to_string(), i)]))
        .unwrap(),
    );
    tests.output.push(
      ANS_NAME
        .format(&HashMap::from([("i".to_string(), i)]))
        .unwrap(),
    );
  }
  return CPTProblemMeta {
    memory_limit: format!("{} megabytes", resp.memory_limit),
    time_limit: format!("{} seconds", resp.time_limit),
    test: tests,
  };
}

fn handle(request: web_server::Request, _: web_server::Response) -> web_server::Response {
  let resp: CCResponse = serde_json::from_str(&request.get_body()).unwrap();

  eprintln!(
    "name:\t{}\ngroup:\t{}\nURL:\t{}",
    resp.name, resp.group, resp.url
  );

  let meta = convert_meta(&resp);

  write_problem(&PathBuf::from("."), &meta, &resp.tests).unwrap();

  eprintln!(
    "parsed {} testcases from {}",
    resp.tests.len().to_string().yellow(),
    resp.url.yellow()
  );

  exit(0);
}

fn write_problem(
  dir: &PathBuf,
  meta: &CPTProblemMeta,
  tests: &[TestCase],
) -> Result<(), Box<dyn Error>> {
  serde_yaml::to_writer(
    OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open(dir.join("meta.yaml"))?,
    &HashMap::from([("problem", meta)]),
  )?;

  assert_eq!(meta.test.input.len(), tests.len());
  assert_eq!(meta.test.output.len(), tests.len());

  for i in 0..tests.len() {
    eprintln!("writing input file {}", &meta.test.input[i].yellow());
    write(dir.join(&meta.test.input[i]), &tests[i].input)?;
    eprintln!("writing output file {}", &meta.test.output[i].yellow());
    write(dir.join(&meta.test.output[i]), &tests[i].output)?;
  }

  return Ok(());
}
