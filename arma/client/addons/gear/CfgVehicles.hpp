class CfgVehicles {
    class MapBoard_altis_F;
    class ACE_bodyBagObject: MapBoard_altis_F {
        class ACE_Actions {
            class ACE_MainActions {
                class GVAR(store) {
                    displayName = "Store";
                    condition = QUOTE([_target] call FUNC(bodybag_canStore));
                    statement = QUOTE([_target] call FUNC(bodybag_store));
                    exceptions[] = {"isNotInside"};
                };
            };
        };
    };
};
