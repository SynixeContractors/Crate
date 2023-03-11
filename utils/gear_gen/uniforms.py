fleece_base = [
    "fleece_{0}_g3_field_pants",
]

fleece_colors = [
    "grey",
    "khk",
    "mc",
    "rgr",
]

pcu_base = [
    "pcu_{0}_g3_field_pants",
]

pcu_colors = [
    "aor1",
    "aor2",
    "grey",
    "mc",
    "tan",
]

base = [
    "g3_field_set_{0}",
    "g3_field_set_rolled_{0}",
]

base_colors = [
    "3cd",
    "aor1",
    "aor2",
    "atacsau",
    "atacsfg",
    "blk",
    "flecktarn",
    "khk",
    "m81",
    "mc",
    "mcalpine",
    "mcarid",
    "mctropic",
    "rgr",
    "tropentarn",
]

tshirts = [
    "tshirt_g3_field_pants_{0}",
]

tshirts_colors = [
    "3CD",
    "aor1",
    "aor2",
    "atacsau",
    "atacsfg",
    "khk",
    "m81",
    "mc",
    "mcarid",
    "mctropic",
    "rgr",
]

def generate_classes():
    items = []
    for b in base:
        for x in base_colors:
            name = "milgp_u_" + b.format(x)
            items.append(
                [
                    name,
                    130,
                ]
            )
            for y in base_colors:
                name = "milgp_u_" + b.format(x) + "_" + y
                items.append(
                    [
                        name,
                        130,
                    ]
                )
    for b in pcu_base:
        for x in pcu_colors:
            for y in base_colors:
                name = "milgp_u_" + b.format(x) + "_" + y
                items.append(
                    [
                        name,
                        130,
                    ]
                )
    for b in fleece_base:
        for x in fleece_colors:
            for y in base_colors:
                name = "milgp_u_" + b.format(x) + "_" + y
                items.append(
                    [
                        name,
                        130,
                    ]
                )
    for b in tshirts:
        for x in tshirts_colors:
            name = "milgp_u_" + b.format(x)
            items.append(
                [
                    name,
                    130,
                ]
            )
    return items
