use phisher_phinder_rust::persistence::{connect, find_random_run};
use phisher_phinder_rust::ui::display_run;
use std::path::Path;

fn main() {
    let db_path = std::env::var("PP_DB_PATH").unwrap();

    let conn = connect(Path::new(&db_path)).unwrap();

    let run = find_random_run(&conn).unwrap();

    println!("{}", display_run(&run).unwrap());
}
