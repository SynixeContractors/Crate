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

#define TEXTURE_LAND "#(argb,8,8,3)color(0.581,0.441,0.293,1,co)"
#define TEXTURE_HELI "#(argb,8,8,3)color(0.293,0.441,0.581,1,co)"
#define TEXTURE_THING "#(argb,8,8,3)color(0.293,0.581,0.441,1,co)"
#define TEXTURE_SEA "#(argb,8,8,3)color(0.441,0.293,0.581,1,co)"
#define TEXTURE_PLANE "#(argb,8,8,3)color(0.581,0.293,0.441,1,co)"

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

    class VR_Area_01_circle_4_grey_F;
    class VR_Area_01_square_4x4_grey_F;
    class VR_Area_01_square_2x2_grey_F;

    class GVAR(land_large): VR_Area_01_circle_4_grey_F {
        displayName = "Spawn (Land, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
        hiddenSelectionsTextures[] = {TEXTURE_LAND};
    };
    class GVAR(land_medium): VR_Area_01_square_4x4_grey_F {
        displayName = "Spawn (Land, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
        hiddenSelectionsTextures[] = {TEXTURE_LAND};
    };
    class GVAR(sea_large): VR_Area_01_circle_4_grey_F {
        displayName = "Spawn (Sea, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
        hiddenSelectionsTextures[] = {TEXTURE_SEA};
    };
    class GVAR(sea_medium): VR_Area_01_square_4x4_grey_F {
        displayName = "Spawn (Sea, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
        hiddenSelectionsTextures[] = {TEXTURE_SEA};
    };

    class GVAR(heli_large): VR_Area_01_circle_4_grey_F {
        displayName = "Spawn (Heli, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
        hiddenSelectionsTextures[] = {TEXTURE_HELI};
    };

    class GVAR(plane_large): VR_Area_01_circle_4_grey_F {
        displayName = "Spawn (Plane, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
        hiddenSelectionsTextures[] = {TEXTURE_PLANE};
    };

    class GVAR(thing_large): VR_Area_01_circle_4_grey_F {
        displayName = "Spawn (Thing, Large)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 9;
        hiddenSelectionsTextures[] = {TEXTURE_THING};
    };
    class GVAR(thing_medium): VR_Area_01_square_4x4_grey_F {
        displayName = "Spawn (Thing, Medium)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 5;
        hiddenSelectionsTextures[] = {TEXTURE_THING};
    };
    class GVAR(thing_small): VR_Area_01_square_2x2_grey_F {
        displayName = "Spawn (Thing, Small)";
        editorCategory = "SynixeContractors";
        GVAR(size) = 3;
        hiddenSelectionsTextures[] = {TEXTURE_THING};
    };
};
