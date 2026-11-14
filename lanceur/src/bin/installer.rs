//! Installation des outils WEB·SUITE (voir installer.rs).
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(web_suite_lanceur::installer::principal(&args));
}
