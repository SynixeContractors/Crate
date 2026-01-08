/// Transport cost factor based on map
pub fn factor_for_map(world: &str) -> f64 {
    match world {
        // Highly Developed - Good transport infrastructure
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
        | "pja308" => 0.90,
        // Remote Areas - Limited infrastructure, higher transport costs
        "Bootcamp_ACR" | "intro" | "Tanoa" | "Woodland_ACR" | "porto" | "ProvingGrounds_PMC"
        | "Mountains_ACR" | "pulau" | "utes" | "Shapur_BAF" | "tem_suursaariv"
        | "blud_cordelia" | "pja307" | "IslaPera" | "go_map_fjord" | "tem_kujari" | "zargabad"
        | "Desert_E"
        | "takistan"
        | "tem_anizay"
        | "SefrouRamal"
        | "MCN_Aliabad"
        | "NorthTakistan"
        | "swu_public_salman_map"
        | "juju_javory"
        | "Farabad" => 1.15,
        _ => 1.00, // Default price for unknown maps
    }
}
