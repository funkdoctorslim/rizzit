use rizzit::parse;
use std::time::Instant;

fn main() {
    let sample_filenames = [
        "The.Matrix.1999.1080p.BluRay.x264-FGT.mkv",
        "[SubsPlease] Kono Subarashii Sekai ni Shukufuku wo! 3 - 01 (1080p) [F109283].mkv",
        "Cyberpunk.Edgerunners.S01E02.Like.a.Boy.1080p.BluRay.Remux.Dual-Audio.DTS-HD.MA.5.1.H.264-CRUCiBLE.mkv",
        "Interstellar.2014.2160p.UHD.BluRay.x265.10bit.HDR.Atmos.7.1-TERMINAL.mkv",
        "www.Torrenting.com - Marvels.Agents.of.S.H.I.E.L.D.S04E15.1080p.WEB-DL.DD5.1.H264-RARBG.mkv",
        "Interstellar20142160pHEVCx265-RARBG.mkv",
        "Love.Death.And.Robots.S02E01.Automated.Customer.Service.2160p.Web.mkv",
    ];

    let iterations = 100_000;
    println!("Benchmarking rizzit parser over {} iterations...", iterations * sample_filenames.len());

    let start = Instant::now();
    for _ in 0..iterations {
        for name in &sample_filenames {
            let _ = std::hint::black_box(parse(name));
        }
    }
    let elapsed = start.elapsed();
    let total_parsed = iterations * sample_filenames.len();
    let per_sec = (total_parsed as f64) / elapsed.as_secs_f64();
    let ns_per_parse = (elapsed.as_nanos() as f64) / (total_parsed as f64);

    println!("Completed in {:.2?}", elapsed);
    println!("Throughput: {:.0} filenames/sec", per_sec);
    println!("Latency: {:.2} ns/filename", ns_per_parse);
}
