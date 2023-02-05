class CUP_AH6_BASE;
class CUP_AH6_ARMED_BASE: CUP_AH6_BASE {
    class ACE_Actions {
        class GVAR(minigun) {
            position = "[0,0.25,0]";
            displayName = "Minigun";
            condition = "true";
            statement = "";
            distance = 2;
        };
        class GVAR(dagrLeft) {
            selection = "gau19_end_1";
            displayName = "DAGR";
            condition = QUOTE([ARR_3(_target, ""PylonRack_12Rnd_PG_missiles"", 1)] call FUNC(pylon_isType));
            statement = "";
            distance = 2;
            class GVAR(unload) {
                displayName = "Unload DAGR";
                condition = QUOTE(_target ammoOnPylon 1 > 0);
                statement = QUOTE([ARR_3(_target, 1, QQGVAR(DAGR))] call FUNC(pylon_unload));
            };
            class GVAR(load) {
                displayName = "Load DAGR";
                condition = QUOTE(_target ammoOnPylon 1 < 12 && {[ARR_2(_target, QQGVAR(DAGR))] call FUNC(pylon_canLoad)});
                statement = QUOTE([ARR_3(_target, QQGVAR(DAGR), 1)] call FUNC(pylon_load));
            };
        };
        class GVAR(dagrRight) {
            selection = "gau19_end_2";
            displayName = "DAGR";
            condition = QUOTE([ARR_3(_target, ""PylonRack_12Rnd_PG_missiles"", 2)] call FUNC(pylon_isType));
            statement = "";
            distance = 2;
            class GVAR(unload) {
                displayName = "Unload DAGR";
                condition = QUOTE(_target ammoOnPylon 2 > 0);
                statement = QUOTE([ARR_3(_target, 2, QQGVAR(DAGR))] call FUNC(pylon_unload));
            };
            class GVAR(load) {
                displayName = "Load DAGR";
                condition = QUOTE(_target ammoOnPylon 2 < 12 && {[ARR_2(_target, QQGVAR(DAGR))] call FUNC(pylon_canLoad)});
                statement = QUOTE([ARR_3(_target, QQGVAR(DAGR), 2)] call FUNC(pylon_load));
            };
        };
    };
};
