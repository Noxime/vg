fn main() {
    if std::env::var("PROFILE") == Ok("release".into()) {
        std::fs::write(".vgrelease", "").unwrap();
    }
}