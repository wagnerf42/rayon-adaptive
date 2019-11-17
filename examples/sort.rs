use rayon_adaptive::merge_sort_adaptive;
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

fn main() {
    let mut input = (0..10000u32).rev().collect::<Vec<u32>>();
    //println!("before {:?}", input);
    #[cfg(feature = "logs")]
    {
        let p = ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .expect("builder failed");
        let log = p.logging_install(|| merge_sort_adaptive(&mut input)).1;
        //println!("after {:?}", input);
        log.save_svg("beast_sort.svg")
            .expect("saving svg file failed");
    }

    #[cfg(not(feature = "logs"))]
    {
        merge_sort_adaptive(&mut input);
    }
    assert_eq!(input, (0..10000u32).collect::<Vec<u32>>());
}
