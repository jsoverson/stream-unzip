# stream-unzip

This library is a minimal unzip implementation designed for streaming data.

## Important note

Zip files contain the central directory at the end of the file. This library
decodes Zip entries as they are read and does not reference the central directory.

This works for many zip files but there may be edge cases.

## Usage

```rust
let path = "path/to/file.zip";
let mut file = tokio::fs::File::open(path).await.unwrap();
let mut buff: [u8; 1024] = [0; 1024];

let mut zip_reader = ZipReader::default();
while let Ok(num) = file.read(&mut buff).await {
    if num == 0 {
        break;
    }
    zip_reader.update(buff[..num].to_vec().into());

    // Entries can be drained from the reader as they
    // are completed.
    let entries = zip_reader.drain_entries();
    for entry in entries {
        println!("entry: {}", entry.name());
        // write to disk or whatever you need.
    }

}
// Or read the whole file and deal with the entries
// at the end.
zip_reader.finish();
let entries = zip_reader.drain_entries();
```

## Running the example

```sh
cargo run --example <zip file> <output directory>
```

## Contributing

There are known zip files that this library can not yet decode. They are found in the `testdata/todo` directory. They will be addressed by the author when the need arises but you are free to contribute fixes for them at any time. If there are other issues with decompressing zip files, please include a minimal zip file that reproduces the issue.

## License

This project is licensed under

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
