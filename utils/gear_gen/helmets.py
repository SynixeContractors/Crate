models = [
    "airframe_01",
    "opscore_01",
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
    "rhr",
    "tan",
    "wht",
]

hat_models = [
    "cap_01",
    "cap_02",
    "cap_backwards_01",
    "cap_backwards_02",
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
