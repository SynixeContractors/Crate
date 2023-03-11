
vest_jpc_config = """
class ItemInfo: VestItem {
class HitpointsProtectionInfo {
    class Chest {
        HitpointName = "HitChest";
        armor = 15;
        passThrough = 0.45;
    };
    class Diaphragm {
        HitpointName = "HitDiaphragm";
        armor = 15;
        passThrough = 0.45;
    };
    class Abdomen {
        hitpointName = "HitAbdomen";
        armor = 15;
        passThrough = 0.45;
    };
    class Body {
        hitpointName = "HitBody";
        armor = 15;
        passThrough = 0.45;
    };
};
};
"""
vest_marciras_config = """
class ItemInfo: VestItem {
class HitpointsProtectionInfo {
    class Chest {
        HitpointName = "HitChest";
        armor = 18;
        passThrough = 0.3;
    };
    class Diaphragm {
        HitpointName = "HitDiaphragm";
        armor = 18;
        passThrough = 0.3;
    };
    class Abdomen {
        hitpointName = "HitAbdomen";
        armor = 18;
        passThrough = 0.3;
    };
    class Body {
        hitpointName = "HitBody";
        armor = 18;
        passThrough = 0.3;
    };
};
};
"""
vest_mmac_config = """
class ItemInfo: VestItem {
class HitpointsProtectionInfo {
    class Chest {
        HitpointName = "HitChest";
        armor = 16;
        passThrough = 0.4;
    };
    class Diaphragm {
        HitpointName = "HitDiaphragm";
        armor = 16;
        passThrough = 0.4;
    };
    class Abdomen {
        hitpointName = "HitAbdomen";
        armor = 16;
        passThrough = 0.4;
    };
    class Body {
        hitpointName = "HitBody";
        armor = 16;
        passThrough = 0.4;
    };
};
};
"""

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
    "wht",
]

vests = [
    ["jpc_assaulter", 320, 2],
    ["jpc_assaulter_belt", 370, 3],
    ["jpc_grenadier", 320, 2],
    ["jpc_arenadier_belt", 370, 3],
    ["jpc_hgunner", 320, 2],
    ["jpc_hgunner_belt", 370, 3],
    ["jpc_light", 330, 2],
    ["jpc_marksman", 320, 2],
    ["jpc_marksman_belt", 370, 3],
    ["jpc_medic", 320, 2],
    ["jpc_medic_belt", 370, 3],
    ["jpc_teamleader", 320, 2],
    ["jpc_teamleader_belt", 370, 3],
    ["marciras_assaulter", 480, 2],
    ["marciras_assaulter_belt", 530, 3],
    ["marciras_grenadier", 480, 2],
    ["marciras_grenadier_belt", 530, 3],
    ["marciras_hgunner", 480, 2],
    ["marciras_hgunner_belt", 530, 3],
    ["marciras_light", 330, 2],
    ["marciras_marksman", 480, 2],
    ["marciras_marksman_belt", 530, 3],
    ["marciras_medic", 480, 2],
    ["marciras_medic_belt", 530, 3],
    ["marciras_teamleader", 480, 2],
    ["marciras_teamleader_belt", 530, 3],
    ["mmac_assaulter", 360, 2],
    ["mmac_assaulter_belt", 410, 3],
    ["mmac_grenadier", 360, 2],
    ["mmac_grenadier_belt", 410, 3],
    ["mmac_hgunner", 360, 2],
    ["mmac_hgunner_belt", 410, 3],
    ["mmac_light", 330, 2],
    ["mmac_marksman", 360, 2],
    ["mmac_marksman_belt", 410, 3],
    ["mmac_medic", 360, 2],
    ["mmac_medic_belt", 410, 3],
    ["mmac_teamleader", 360, 2],
    ["mmac_teamleader_belt", 410, 3],
]

def generate_classes():
    items = []
    for vest in vests:
        for x in colors:
            if x in ["tan"]:
                continue
            if "mmac" in vest[0] and x in ["mcar", "mcb", "mct"]:
                continue
            if "marciras" in vest[0]and x in ["mcar", "mcb", "mct"]:
                continue
            if vest[2] == 1:
                name = "milgp_v_" + vest[0] + "_" + x + "_" + x
                items.append(
                    [
                        name,
                        vest[1],
                    ]
                )
                continue
            for y in colors:
                name = "milgp_v_" + vest[0] + "_" + x + "_" + y
                if vest[2] == 3:
                    name = name + "_" + y
                items.append(
                    [
                        name,
                        vest[1],
                    ]
                )
    return items

if __name__ == "__main__":
    with open("CfgVehicles.hpp", "w") as out:
        for item in generate_classes():
            if item[0].startswith("milgp_v_jpc"):
                out.write(
                    """
    class {0}: ItemCore {{{1}}};
    """.format(
                item[0], vest_jpc_config
            )
                )
            elif item[0].startswith("milgp_v_marciras"):
                out.write(
                    """
    class {0}: ItemCore {{{1}}};
    """.format(
                item[0], vest_marciras_config
            )
                )
            elif item[0].startswith("milgp_v_mmac"):
                out.write(
                    """
    class {0}: ItemCore {{{1}}};
    """.format(
                item[0], vest_mmac_config
            )
                )
