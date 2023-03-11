models = [
    ["Breacher", 240],
    ["hydration", 120],
    ["Pointman", 360],
    ["Tomahawk", 240],
]

colors = [
    "aor1",
    "aor2",
    "blk",
    "cb",
    "khk",
    "mc",
    "mcar",
    "mcb",
    "mct",
    "rgr",
    "tan",
    "why",
]

def generate_classes():
    items = []
    for model in models:
        for color in colors:
            name = "milgp_bp_" + model[0] + "_" + color
            items.append(
                [
                    name,
                    model[1],
                ]
            )
    return items
