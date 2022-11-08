mod preset;

fn main() {
  match std::env::args().nth(1).as_ref().map(|s| s.as_str()) {
    Some("desktop") => preset::desktop::run(),
    Some("laptop") => preset::laptop::run(),
    Some(preset) => eprintln!("invalid preset `{preset}`"),
    None => eprintln!("choose a preset"),
  }
}
