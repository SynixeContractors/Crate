#define STORE_ACTION class ACE_Actions { \
    class ACE_MainActions { \
        class GVAR(Store) { \
            displayName = "Store"; \
            condition = QUOTE([ARR_2(_player,_target)] call FUNC(canStore)); \
            statement = QUOTE([ARR_2(_player,_target)] call FUNC(store)); \
            exceptions[] = {"isNotInside"}; \
            showDisabled = 0; \
            priority = 1; \
            icon = "\a3\3den\data\displays\display3den\entitymenu\garage_ca.paa"; \
        }; \
    }; \
};

class CfgVehicles {
    class LandVehicle;
    class Car: LandVehicle {
        STORE_ACTION
    };
    class Tank: LandVehicle {
        STORE_ACTION
    };

    class Air;
    class Helicopter: Air {
        STORE_ACTION
    };
    class Plane: Air {
        STORE_ACTION
    };

    class Ship;
    class Ship_F: Ship {
        STORE_ACTION
    };

    class Cage_Large_lxWS;
    class Cage_Medium_lxWS;
    class Cage_Small_lxWS;

    class GVAR(land_large): Cage_Large_lxWS {
        displayName = "Spawn (Land, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
    };
    class GVAR(land_medium): Cage_Medium_lxWS {
        displayName = "Spawn (Land, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
    };
    class GVAR(land_small): Cage_Small_lxWS {
        displayName = "Spawn (Land, Small)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 3;
    };

    class GVAR(sea_large): Cage_Large_lxWS {
        displayName = "Spawn (Sea, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
    };
    class GVAR(sea_medium): Cage_Medium_lxWS {
        displayName = "Spawn (Sea, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
    };
    class GVAR(sea_small): Cage_Small_lxWS {
        displayName = "Spawn (Sea, Small)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 3;
    };

    class GVAR(heli_large): Cage_Large_lxWS {
        displayName = "Spawn (Heli, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
    };
    class GVAR(heli_medium): Cage_Medium_lxWS {
        displayName = "Spawn (Heli, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
    };
    class GVAR(heli_small): Cage_Small_lxWS {
        displayName = "Spawn (Heli, Small)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 3;
    };

    class GVAR(plane_large): Cage_Large_lxWS {
        displayName = "Spawn (Plane, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
    };
    class GVAR(plane_medium): Cage_Medium_lxWS {
        displayName = "Spawn (Plane, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
    };
    class GVAR(plane_small): Cage_Small_lxWS {
        displayName = "Spawn (Plane, Small)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 3;
    };

    class GVAR(thing_large): Cage_Large_lxWS {
        displayName = "Spawn (Thing, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
    };
    class GVAR(thing_medium): Cage_Medium_lxWS {
        displayName = "Spawn (Thing, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 3;
    };
    class GVAR(thing_small): Cage_Small_lxWS {
        displayName = "Spawn (Thing, Small)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 1;
    };
};
