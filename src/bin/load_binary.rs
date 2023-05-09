// Open3dならロードできるがPymeshlabではロードできない

use dotenv::dotenv;
use std::env;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let input_path = env::var("OUTPUT_PATH").unwrap();
    let point_num: usize = 444;

    let header = make_header(point_num);

    // 上書きをする場合、OpenOptionsで開く必用がある
    // そのまま先頭に追記するのが楽そう
    let mut writer = BufWriter::new(OpenOptions::new().write(true).open(&input_path)?);
    writer.write_all(header.as_bytes())?;
    writer.flush()?;

    Ok(())
}

fn make_header(point_num: usize) -> String {
    let point_digits_size: usize = 20 - format!("{:}", point_num).to_string().len();
    let adjust_comment = &"xxxxxxxxxxxxxxxxxxxx"[0..point_digits_size];
    let header:String = format!(
        "ply\nformat binary_little_endian 1.0\ncomment adjust str {}\nelement vertex {}\nproperty int x\nproperty int y\nproperty int z\nend_header\n",
        adjust_comment,
        point_num
    );
    header
}
