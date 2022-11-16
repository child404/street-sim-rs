#![allow(dead_code, unused_imports)]
mod candidate;
mod street_sim;
mod text_sim;

use candidate::Candidate;
use street_sim_rs::{Plz, SimAlgo};

use std::panic;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
use street_sim::{Street, StreetConfig};

use crate::text_sim::Config;

const CHUNK_SIZE: usize = 1000;

#[inline]
fn join_to_row<T>(
    index: &str,
    street: &str,
    place: &str,
    mstreet: (Option<Candidate>, Option<T>),
) -> String
where
    T: ToString,
{
    [
        index,
        street,
        place,
        &mstreet.0.unwrap_or_default().text,
        &match mstreet.1 {
            Some(location) => location.to_string(),
            None => "".to_string(),
        },
    ]
    .join("\t")
}

fn main() {
    // TODO: compare "Siders/Sierre" to places.txt
    let streets = [
        "Mühlematt 7\\11 (Emmenbrücke)",
        "Mühlematt 7-11 (Emmenbrücke)",
        "Meierhofstr. 3",
        "Meierhofstr. 1",
        "Meierhofstr. 5",
        "Hauptstr. 5",
        "Hohenrainstr. 10-14",
        "Bernstr. 8",
        "Neuheim 8+10",
        "Hohenrainstrasse 10+12+14",
        "St. Cergue 11",
        "Allée du Communet / Allée Leotherius 3,5,7 / Allée Waldo 4,6,8 (Eikenott)",
        "Avenue de la Gare 3 / Rue du Rhône 2 (Le Kluser)",
        "Rue des Terreaux 15,17,19,21,23,25,27,29 (Métropole 2000 I + II)",
        "Dorfstrasse 4c / Gerbergasse 1,3,4,5,7,11,13,15 (Gerber-Areal)",
        "Route de St-Légier 15a–d",
        "Rue du Riant-Coteau 120,122,124 / Chemin des Fleurs 6",
        "Avenue Edouard-Müller 17",
        "Route de Buyère 2 (Veillon-Areal)",
        "Avenue Bel-Air 52,54",
        "Avenue des Baumes 17–19",
        "Chemin Bosquets de Paudille 15, 16, 17, 18, 19, 20, 21, 22, 23, 25",
        "chfsdfdf de saint-cierges 3,fas23dfsfsdf",
        "Ziegeleistrasse 52,52a,54,54a,56,56a,58,58a",
        "Günzenenstrasse 5,5a–c",
        "Route de Pré-Bois 20 (ICC-International Center Cointrin)",
        "Route de l'Aéroport 10",
        "Rue de la Prulay 30,32,34,36,38",
        "Avenue des Morgines 18, Chemin L-Hubert 13",
        "Route de Chancy 71 (Lancy Centre) Route de Chancy 77",
        "Chemin des Troënes 13,15",
        "Rue Ami-Lullin 4",
        "Rue de Lyon 114–120 / Rue de Bourgogne 19–31 (Quartet)",
        "Avenue Ernest-Pictet 20,22,26a",
        "Rue Pestalozzi 23+23bis",
        "Wright-Strasse 37,39,41,43 (Wilhelm & Bertha, Glattpark)",
        "Avenue des Champs-Montants 12a–c",
        "Rue de la Pierre-Ã -Mazel 10 (Stade de la Maladière)",
        "Rue de la Pierre-à-Mazel 10 (Stade de la Maladière)",
        "Rue Fritz-Courvoisier 34c–d",
        "Rue du Clos 1,3,5 (Parc du Château)",
        "Avenue du Bietschhorn 21a–d,23a–b (Les Aquarelles)",
    ];
    for street in streets {
        let street = Street::new(street, None).unwrap();
        println!("{:?}", &street.value);
        let mat = street_sim::find_matches::<Plz>(&street, StreetConfig::<Plz>::default());
        println!("{:?}", mat);
    }
    // let reader = BufReader::new(File::open("./test_data/real_data.csv").expect("file exists"));
    // let street_matcher = StreetMatcher::default();
    // let mut file_out = File::options()
    //     .create(true)
    //     .write(true)
    //     .append(true)
    //     .open("./test_data/real_data_output.csv")
    //     .unwrap();
    // let _: Vec<_> = reader
    //     .lines()
    //     .flatten()
    //     .collect::<Vec<String>>()
    //     .chunks(CHUNK_SIZE)
    //     .map(|chunk| {
    //         let lines = chunk
    //             .iter()
    //             .flat_map(|line| {
    //                 if let [index, street, place] =
    //                     line.split('\t').take(3).collect::<Vec<&str>>()[..]
    //                 {
    //                     panic::catch_unwind(|| street_matcher.match_by_place(street, Some(place)))
    //                         .ok()
    //                         .map(|mat| join_to_row(index, street, place, mat))
    //                 } else {
    //                     None
    //                 }
    //             })
    //             .collect::<Vec<String>>()
    //             .join("\n");
    //         if lines.is_empty() {
    //             println!("Lines are empty, skipping...");
    //             return;
    //         }
    //         match writeln!(file_out, "{}", lines) {
    //             Ok(_) => {
    //                 println!("Successfully written chunk to file");
    //             }
    //             Err(err) => {
    //                 eprintln!("Couldn't write to file: {}", err);
    //             }
    //         }
    //     })
    //     .collect();
}
