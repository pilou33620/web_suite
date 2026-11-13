//! Serveur MCP de WEB·SUITE sur stdio (lanceur/src/mcp/mod.rs) : Claude Code le lance lui-même.
//!
//!     claude mcp add web_suite -- <web_suite>/bin/mcp_web_suite

fn main() {
    // stdout ne porte que le protocole : tout le reste (journal, panique) part sur stderr
    let fin = web_suite_lanceur::mcp::servir(std::io::stdin().lock(), std::io::stdout().lock());
    std::process::exit(if fin.is_ok() { 0 } else { 1 });
}
