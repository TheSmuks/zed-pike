//! Criterion-based performance benchmarks for `pike-lsp`.
//!
//! Run with `cargo bench -p pike-lsp` (or `cargo bench --workspace`).
//! The numbers produced here back the SLOs in `docs/perf.md`.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use pike_lsp::analysis::Analysis;

const SMALL_PIKE: &str = "int main() { return 0; }\n";

const MEDIUM_PIKE: &str = r#"
class Demo {
  public int add(int a, int b) { return a + b; }
  public string greet(string name) { return "Hello, " + name; }
  protected constant PI = 3;
}

int main(int argc, array(string) argv) {
  Demo d = Demo();
  write("%d %s\n", d->add(1, 2), d->greet("Pike"));
  return 0;
}
"#;

/// Generate a synthetic source file of approximately `n_kloc` kLOC
/// of function declarations. Useful for measuring p99 hover / symbol
/// latency on a "10 kLOC" workspace.
fn make_large_pike(n_kloc: usize) -> String {
    let mut out = String::with_capacity(n_kloc * 80);
    for i in 0..(n_kloc * 10) {
        out.push_str(&format!(
            "int fn_{i:06}(int a, int b) {{ return a + b + {i}; }}\n"
        ));
    }
    out
}

fn bench_parse_small(c: &mut Criterion) {
    c.bench_function("parse_small_24B", |b| {
        b.iter(|| {
            let a = Analysis::new();
            a.open("file:///x.pike", black_box(SMALL_PIKE.to_string()));
        })
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let src = MEDIUM_PIKE.to_string();
    c.bench_function("parse_medium_240B", |b| {
        b.iter(|| {
            let a = Analysis::new();
            a.open("file:///x.pike", black_box(src.clone()));
        })
    });
}

fn bench_parse_large_10kloc(c: &mut Criterion) {
    let src = make_large_pike(10);
    let size = src.len() as u64;
    let mut group = c.benchmark_group("parse_10kloc");
    group.throughput(Throughput::Bytes(size));
    group.bench_function("open_update", |b| {
        b.iter(|| {
            let a = Analysis::new();
            a.open("file:///big.pike", black_box(src.clone()));
        })
    });
    group.finish();
}

fn bench_hover_10kloc(c: &mut Criterion) {
    let src = make_large_pike(10);
    let a = Analysis::new();
    a.open("file:///big.pike", src.clone());
    // The 50th function in the file: `fn_000050`. The identifier
    // `fn_000050` starts at approximately line 50, column 4.
    let mut group = c.benchmark_group("hover_10kloc");
    group.throughput(Throughput::Elements(1));
    group.bench_function("hover_p99", |b| {
        b.iter(|| {
            let h = a.hover("file:///big.pike", 50, 4);
            black_box(h);
        })
    });
    group.finish();
}

fn bench_symbols_10kloc(c: &mut Criterion) {
    let src = make_large_pike(10);
    let a = Analysis::new();
    a.open("file:///big.pike", src);
    let mut group = c.benchmark_group("symbols_10kloc");
    group.bench_function("document_symbols", |b| {
        b.iter(|| {
            let s = a.document_symbols("file:///big.pike");
            black_box(s);
        })
    });
    group.finish();
}

fn bench_diagnostics_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("diagnostics");
    group.bench_function("known_pp_clean", |b| {
        b.iter(|| {
            let a = Analysis::new();
            a.open("file:///x.pike", black_box(SMALL_PIKE.to_string()));
            black_box(a.diagnostics("file:///x.pike"));
        })
    });
    group.finish();
}

criterion_group!(
    parse,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large_10kloc
);
criterion_group!(
    queries,
    bench_hover_10kloc,
    bench_symbols_10kloc,
    bench_diagnostics_small
);
criterion_main!(parse, queries);
