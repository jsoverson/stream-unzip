use stream_unzip::{ZipEntry, ZipReader};
use tokio::io::AsyncReadExt;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let outdir = std::env::args().nth(2).unwrap();
    println!("Extracting file {} to {}...", path, outdir);
    let mut file = tokio::fs::File::open(path).await.unwrap();
    let mut buff: [u8; 1024] = [0; 1024];

    let mut zip_reader = ZipReader::default();
    while let Ok(num) = file.read(&mut buff).await {
        if num == 0 {
            println!("done");
            break;
        }
        zip_reader.update(buff[..num].to_vec().into());

        // This is where the entries that have been read
        // can be drained from the reader.
        let entries = zip_reader.drain_entries();
        write_entries(&outdir, entries).await;
    }
    zip_reader.finish();

    // Alternately, you can read the entire file and deal
    // with each entry in one go.
    write_entries(&outdir, zip_reader.drain_entries()).await;

    Ok(())
}

async fn write_entries(outdir: &str, entries: Vec<ZipEntry>) {
    for entry in entries {
        let inflated = entry.inflate().unwrap();
        tokio::fs::write(format!("{}/{}", outdir, inflated.name()), inflated.data())
            .await
            .unwrap();
    }
}
