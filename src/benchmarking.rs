use anyhow::Result;
use std::{
    fs::{create_dir_all, write, OpenOptions},
    io::Write,
    path::Path,
};

pub fn generate_benchmarking(tmp_dir: &Path, maybe_name: Option<&str>) -> Result<()> {
    let name = if let Some(name) = maybe_name {
        name
    } else {
        "benchmark"
    };

    let mut toml = OpenOptions::new()
        .append(true)
        .open(tmp_dir.join("Cargo.toml"))?;

    writeln!(toml, "{}", format_benchmarking(name))?;

    let bench_folder = tmp_dir.join("benches");
    create_dir_all(&bench_folder)?;
    let mut bench_file = bench_folder.join(name);
    bench_file.set_extension("rs");

    write(
        bench_file,
        "use criterion::{black_box, criterion_group, criterion_main, Criterion};\n\n\
        fn criterion_benchmark(_c: &mut Criterion) {\n\tprintln!(\"Hello, world!\");\n}\n\n\
        criterion_group!(\n\tbenches,\n\tcriterion_benchmark\n);\ncriterion_main!(benches);",
    )?;

    Ok(())
}

pub fn format_benchmarking(name: &str) -> String {
    format!(
        "
[dev-dependencies]
criterion = \"*\"

[profile.release]
debug = true

[[bench]]
name = \"{name}\"
harness = false",
    )
}
