use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
};

pub fn read<D: bincode::Decode<()>>(path: &str) -> Option<D> {
    println!("trying to read {}...", path);
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                return None;
            }
            _ => panic!("failed to open file"),
        },
    };
    println!("reading {}...", path);
    let mut file = BufReader::new(file);
    let deserialized: D =
        bincode::decode_from_std_read(&mut file, bincode::config::standard()).unwrap();
    Some(deserialized)
}

pub fn write<D: bincode::Encode>(path: &str, data: D) {
    let mut file = BufWriter::new(File::create(path).unwrap());
    bincode::encode_into_std_write(data, &mut file, bincode::config::standard())
        .expect("failed to write mappings");
}

pub fn read_or_create<T, D: bincode::Encode + bincode::Decode<()>>(
    path: &str,
    create: impl Fn() -> T,
    encode: impl Fn(&T) -> D,
    decode: impl Fn(&D) -> T,
) -> T {
    match read(path) {
        Some(data) => decode(&data),
        None => {
            let data = create();
            write(path, encode(&data));
            data
        }
    }
}
