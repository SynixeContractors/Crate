class ThingX;
class ADDON: ThingX {
    ace_dragging_canCarry = 1;
    ace_dragging_canDrag = 1;
};

class GVAR(DAGR): ADDON {
    scope = 2;
    scopeCurator = 2;
    mass = 140;
    displayName = "DAGR";
    model = "\A3\Weapons_F\Ammo\Rocket_01_fly_F";
    class ACE_Actions {
        class ACE_MainActions {
            displayName = "DAGR";
            position = "[0,-1.5,0]";
            condition = "true";
            distance = 2;
        };
    };
    ace_dragging_carryDirection = 90;
    ace_dragging_carryPosition[] = {1.5, 1, 1};
    ace_dragging_dragDirection = 180;
    ace_dragging_dragPosition[] = {0, 0, 0};
};
