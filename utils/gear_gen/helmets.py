models = [
    "airframe_01",
    "airframe_01_goggles",
    "airframe_02",
    "airframe_02_goggles",
    "airframe_03",
    "airframe_03_goggles",
    "airframe_04",
    "airframe_04_goggles",
    "airframe_05",
    "airframe_05_goggles",
    "airframe_06",
    "airframe_06_goggles",
    "opscore_01",
    "opscore_01_goggles",
    "opscore_02",
    "opscore_02_goggles",
    "opscore_03",
    "opscore_03_goggles",
    "opscore_04",
    "opscore_04_goggles",
    "opscore_05",
    "opscore_05_goggles",
    "opscore_06",
    "opscore_06_goggles",
]

base_cover = [
    "blk",
    "rgr",
    "khk",
]
base = [
    "aor1",
    "aor2",
    "cb_hexagon",
    "rgr_hexagon",
    "khk_hexagon",
    "mc",
    "wht",
]

headphones = [
    "blk",
    "cb",
    "khk",
    "rgr",
    "wht",
]

covers = [
    "blk",
    "aor1",
    "aor2",
    "cb",
    "m81",
    "mc",
    "mcal",
    "mcb",
    "mct",
    "rgr",
    "wht",
]

hats = [
    "aor1",
    "aor2",
    "blk",
    "cb",
    "gry",
    "khk",
    "m81",
    "mc",
    "rgr",
    "tan",
    "wht",
]

hat_models = [
    "cap_01",
    "cap_02",
    "cap_02_goggles",
    "cap_03",
    "cap_03_goggles",
    "cap_backwards_01",
    "cap_backwards_02",
    "cap_backwards_02_goggles",
    "cap_backwards_03",
    "cap_backwards_03_goggles",
]

def generate_classes():
    items = []
    for model in models:
        for a in base:
            for hp in headphones:
                name = "milgp_h_" + model + "_" + a + "_" + hp
                items.append(
                    [
                        name,
                        400,
                    ]
                )
        for a in base_cover:
            for hp in headphones:
                name = "milgp_h_" + model + "_" + a + "_" + hp
                items.append(
                    [
                        name,
                        400,
                    ]
                )
                for c in covers:
                    if model.startswith("opscore") and c == "aor1":
                        continue
                    name = "milgp_h_" + model + "_" + a + "_" + hp + "_" + c + "c"
                    items.append(
                        [
                            name,
                            400,
                        ]
                    )
    for model in hat_models:
        for a in hats:
            if model.endswith("01"):
                name = "milgp_h_" + model + "_" + a + "_blk"
                items.append(
                    [
                        name,
                        400,
                    ]
                )
            else:
                for hp in headphones:
                    name = "milgp_h_" + model + "_" + a + "_" + hp
                    items.append(
                        [
                            name,
                            400,
                        ]
                    )

    return items

if __name__ == "__main__":
    items = generate_classes()
    classes = []
    for item in items:
        classes.append(item[0])
    with open("helmets.txt", "w") as f:
        f.write("\n".join(classes))
    print("\n".join(classes))
