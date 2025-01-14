use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use data_pile::Database;
use rand::RngCore;

fn put_get(c: &mut Criterion) {
    const BATCH_SIZE: usize = 64;
    const MAX_RECORD_LEN: u64 = 1 << 20; // 1MB
    const RECORD_COUNT: usize = 1024;

    let mut group = c.benchmark_group("pile");

    group.throughput(Throughput::Elements(RECORD_COUNT as u64));
    group.bench_function(BenchmarkId::new("put", RECORD_COUNT), |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                (0..RECORD_COUNT)
                    .map(|_| {
                        let record_len = (rng.next_u64() % MAX_RECORD_LEN) as usize + 1;
                        let mut record = vec![0u8; record_len];
                        rng.fill_bytes(&mut record);

                        record
                    })
                    .collect()
            },
            |data: Vec<Vec<u8>>| {
                let tmp = tempfile::tempdir().unwrap();
                let db = Database::file(tmp.path()).unwrap();
                data.iter().for_each(|data| {
                    db.put(data).unwrap();
                });
            },
            BatchSize::PerIteration,
        );
    });

    group.throughput(Throughput::Elements(RECORD_COUNT as u64));
    group.bench_function(BenchmarkId::new("append", RECORD_COUNT), |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();

                (0..RECORD_COUNT)
                    .map(|_| {
                        let record_len = (rng.next_u64() % MAX_RECORD_LEN) as usize + 1;
                        let mut record = vec![0u8; record_len];
                        rng.fill_bytes(&mut record);

                        record
                    })
                    .collect()
            },
            |data: Vec<Vec<u8>>| {
                let tmp = tempfile::tempdir().unwrap();
                let db = Database::file(tmp.path()).unwrap();
                data.chunks(BATCH_SIZE).for_each(|chunk| {
                    let records: Vec<_> = chunk.iter().map(|data| data.as_ref()).collect();
                    db.append(&records).unwrap();
                });
            },
            BatchSize::PerIteration,
        );
    });

    group.throughput(Throughput::Elements(1));
    group.bench_function("read random records", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();

                let tmp = tempfile::tempdir().unwrap();
                let db = Database::file(tmp.path()).unwrap();

                let records: Vec<_> = (0..RECORD_COUNT)
                    .map(|_| {
                        let record_len = (rng.next_u64() % MAX_RECORD_LEN) as usize + 1;
                        let mut record = vec![0u8; record_len];
                        rng.fill_bytes(&mut record);

                        record
                    }).collect();
                let records: Vec<_> = records.iter().map(|data| data.as_ref()).collect();
                db.append(&records).unwrap();


                db
            },
            |db| {
                let mut rng = rand::thread_rng();

                let i = (rng.next_u64() as usize) % RECORD_COUNT;
                let _record = db.get_by_seqno(i).unwrap();
            },
            BatchSize::LargeInput,
        );
    });

    group.throughput(Throughput::Elements(RECORD_COUNT as u64));
    group.bench_function("read consecutive records", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();

                let tmp = tempfile::tempdir().unwrap();
                let db = Database::file(tmp.path()).unwrap();

                let records: Vec<_> = (0..RECORD_COUNT)
                    .map(|_| {
                        let record_len = (rng.next_u64() % MAX_RECORD_LEN) as usize + 1;
                        let mut record = vec![0u8; record_len];
                        rng.fill_bytes(&mut record);

                        record
                    }).collect();
                let records: Vec<_> = records.iter().map(|data| data.as_ref()).collect();
                db.append(&records).unwrap();

                db
            },
            |db| {
                let _maybe_record = db.iter_from_seqno(0).unwrap().for_each(|e| {
                    black_box(e);
                });
            },
            BatchSize::LargeInput,
        );
    });
}

criterion_group!(benches, put_get);
criterion_main!(benches);
