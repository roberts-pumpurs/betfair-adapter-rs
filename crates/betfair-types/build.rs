/// 1. Read the file from assets
/// 2. Parse the file from XML to Rust code
/// 3. Write the file to `env::var("OUT_DIR")` directory

fn main() {
    let aping_files =
        ["assets/AccountAPING.json", "assets/HeartbeatAPING.json", "assets/SportsAPING.json"]
            .map(|x| std::fs::read(x).expect("Couldn't read the file"))
            .map(|x| String::from_utf8(x).expect("File not UTF-8"));

    // let output = parse_file(file).await;
    // write_the_file(file, output).await;
}

// Parse the file from XML to Rust code
async fn parse_file(file: String) -> String {
    todo!()
}

async fn write_the_file(file: String, output: String) {
    todo!()
}
