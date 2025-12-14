/// Price per liter based on map
pub fn prices_for_map(world: &str) -> f64 {
    match world {
        // Highly Developed - Good fuel infrastructure, fuel taxes
        "Altis"
        | "Stratis"
        | "Malden"
        | "Enoch"
        | "chernarus"
        | "chernarus_summer"
        | "Chernarus_Winter"
        | "hellanmaa"
        | "hellanmaaw"
        | "tem_ruha"
        | "tem_vinjesvingenc"
        | "stubbhult"
        | "bozcaada"
        | "tem_summa"
        | "tem_summawcup"
        | "VTF_Lybor"
        | "sara"
        | "sara_dbe1"
        | "saralite"
        | "VTF_Lybor_Winter"
        | "VTF_Korsac"
        | "VTF_Korsac_Winter"
        | "vt7"
        | "Tembelan"
        | "VTF_Prilivsko"
        | "VTF_Prilivsko_Flood"
        | "pja308" => 1.83,
        // Remote Areas - Limited fuel infrastructure, higher transport costs
        "Bootcamp_ACR" | "intro" | "Tanoa" | "Woodland_ACR" | "porto" | "ProvingGrounds_PMC"
        | "Mountains_ACR" | "pulau" | "utes" | "Shapur_BAF" | "tem_suursaariv"
        | "blud_cordelia" | "pja307" | "IslaPera" | "go_map_fjord" | "tem_kujari" => 2.20,
        // Access to oil - Oil fields or refineries nearby, moderate prices
        "zargabad"
        | "Desert_E"
        | "takistan"
        | "tem_anizay"
        | "SefrouRamal"
        | "MCN_Aliabad"
        | "NorthTakistan"
        | "swu_public_salman_map"
        | "juju_javory"
        | "Farabad" => 1.30,
        _ => 2.00, // Default price for unknown maps
    }
}
